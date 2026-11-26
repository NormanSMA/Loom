use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::sync::RwLock;
use totp_rs::{Algorithm, TOTP};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingTicket {
    pub code: String,
    pub qr_uri: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceView {
    pub id: String,
    pub label: String,
    pub created_at: String,
    pub last_seen_at: String,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExchangeResult {
    pub device: DeviceView,
    pub session_token: String,
}

#[derive(Debug, Clone)]
struct StoredDevice {
    view: DeviceView,
    token_hash: String,
}

#[derive(Debug, Clone)]
struct PendingTicket {
    expires_at: DateTime<Utc>,
    used: bool,
}

pub struct AuthManager {
    pending: RwLock<HashMap<String, PendingTicket>>,
    devices: RwLock<Vec<StoredDevice>>,
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
            devices: RwLock::new(Vec::new()),
        }
    }

    pub async fn create_pairing(&self, base_url: &str) -> PairingTicket {
        self.create_pairing_with_ttl(base_url, Duration::seconds(120))
            .await
    }

    pub async fn create_pairing_with_ttl(&self, base_url: &str, ttl: Duration) -> PairingTicket {
        let code = format!("lm_{}", Uuid::new_v4().simple());
        let exp = Utc::now() + ttl;
        let qr_uri = format!("{}/?pair={}", base_url.trim_end_matches('/'), code);

        let mut map = self.pending.write().await;
        let now = Utc::now();
        map.retain(|_, v| !v.used && v.expires_at > now);
        map.insert(
            code.clone(),
            PendingTicket {
                expires_at: exp,
                used: false,
            },
        );

        PairingTicket {
            code,
            qr_uri,
            expires_at: exp.to_rfc3339(),
        }
    }

    pub async fn exchange_pairing(&self, code: &str, label: &str) -> Option<ExchangeResult> {
        let now = Utc::now();
        {
            let mut map = self.pending.write().await;
            let ticket = map.get_mut(code.trim())?;
            if ticket.used || ticket.expires_at <= now {
                return None;
            }
            ticket.used = true;
        }

        let token = format!("loom_tok_{}", Uuid::new_v4().simple());
        let token_hash = sha256_hex(token.as_bytes());
        let now_str = now.to_rfc3339();
        let clean_label = if label.trim().is_empty() {
            "Dispositivo movil".to_string()
        } else {
            label.trim().to_string()
        };

        let view = DeviceView {
            id: Uuid::new_v4().to_string(),
            label: clean_label,
            created_at: now_str.clone(),
            last_seen_at: now_str,
            revoked: false,
        };

        self.devices.write().await.push(StoredDevice {
            view: view.clone(),
            token_hash,
        });

        Some(ExchangeResult {
            device: view,
            session_token: token,
        })
    }

    pub async fn list_devices(&self) -> Vec<DeviceView> {
        self.devices
            .read()
            .await
            .iter()
            .map(|d| d.view.clone())
            .collect()
    }

    pub async fn revoke_device(&self, id: &str) -> bool {
        let mut devices = self.devices.write().await;
        if let Some(dev) = devices.iter_mut().find(|d| d.view.id == id) {
            dev.view.revoked = true;
            return true;
        }
        false
    }

    pub async fn validate_token(&self, token: &str) -> bool {
        let candidate = sha256_hex(token.as_bytes());
        let devices = self.devices.read().await;
        devices
            .iter()
            .any(|d| !d.view.revoked && d.token_hash == candidate)
    }
}

pub fn is_allowed_host(host_header: Option<&str>, remote_enabled: bool) -> bool {
    if remote_enabled {
        return true;
    }
    let Some(raw) = host_header else {
        return false;
    };
    let host = strip_port(raw.trim()).to_ascii_lowercase();
    matches!(host.as_str(), "127.0.0.1" | "localhost" | "[::1]" | "::1")
}

fn strip_port(host: &str) -> &str {
    if host.starts_with('[')
        && let Some(end) = host.find(']')
    {
        return &host[..=end];
    }
    if let Some((name, _port)) = host.rsplit_once(':')
        && !name.contains(':')
    {
        return name;
    }
    host
}

pub fn hash_password(password: &str) -> Result<String, String> {
    let mut salt_bytes = [0_u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|e| e.to_string())?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn verify_totp(secret_bytes: &[u8], code: &str) -> bool {
    let Ok(totp) = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes.to_vec(),
        Some("Loom".to_string()),
        "owner@loom.local".to_string(),
    ) else {
        return false;
    };
    totp.check_current(code.trim()).unwrap_or(false)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{b:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_guard_blocks_non_loopback_when_local_only() {
        assert!(is_allowed_host(Some("127.0.0.1:4320"), false));
        assert!(is_allowed_host(Some("localhost:4321"), false));
        assert!(is_allowed_host(Some("[::1]:4320"), false));
        assert!(!is_allowed_host(Some("evil.example:4320"), false));
        assert!(!is_allowed_host(None, false));
        assert!(is_allowed_host(Some("loom.example.com"), true));
    }

    #[test]
    fn hashes_passwords_and_verifies_totp() {
        let hash = hash_password("clave-segura").unwrap();
        assert!(verify_password("clave-segura", &hash));
        assert!(!verify_password("otra", &hash));

        let secret = b"0123456789abcdef0123";
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_vec(),
            Some("Loom".to_string()),
            "owner@loom.local".to_string(),
        )
        .unwrap();
        let code = totp.generate_current().unwrap();
        assert!(verify_totp(secret, &code));
        assert!(!verify_totp(secret, "000000"));
    }

    #[tokio::test]
    async fn pairing_is_single_use_and_device_revocation_invalidates_token() {
        let mgr = AuthManager::new();
        let ticket = mgr.create_pairing("http://127.0.0.1:4320").await;
        let ex = mgr
            .exchange_pairing(&ticket.code, "iPhone 16")
            .await
            .expect("first exchange succeeds");
        assert!(mgr.validate_token(&ex.session_token).await);

        assert!(mgr.exchange_pairing(&ticket.code, "Reuso").await.is_none());

        let expired = mgr
            .create_pairing_with_ttl("http://127.0.0.1:4320", Duration::seconds(-5))
            .await;
        assert!(mgr.exchange_pairing(&expired.code, "Tarde").await.is_none());

        assert!(mgr.revoke_device(&ex.device.id).await);
        assert!(!mgr.validate_token(&ex.session_token).await);
    }
}
