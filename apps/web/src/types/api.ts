export type AgentActivityState =
  | 'thinking'
  | 'reading'
  | 'analysing'
  | 'debugging'
  | 'creating'
  | 'idle'

export interface HealthInfo {
  status: string
  service: string
  version: string
  demo: boolean
  uptime_secs: number
  port: number
}

export interface LiveEvent {
  kind: string
  state: AgentActivityState
  provider_id: string
  target: string
  detail: string
  ts: string
}

export interface MissionView {
  id: string
  title: string
  mode: string
  effort: string
  sensitivity: string
  provider_id: string
  status: string
  state: AgentActivityState
  worktree: string
  repo_root?: string | null
  gate_passed: boolean | null
  updated_at: string
}

export interface ActiveChatView {
  source: string
  session: string
  title: string
  model: string
  working: boolean
  last_ts: number
}

export interface WorkspaceInfo {
  name: string
  path: string
  is_git_repo: boolean
  sources: string[]
  active_chats: ActiveChatView[]
  last_ts: number
}

export interface MissionDiff {
  mission_id: string
  summary: string
  diff: string
}

export interface ArtifactPreview {
  id: string
  kind: string
  name: string
  mime_type: string
  data_uri: string
  width: number | null
  height: number | null
  created_at: string
}

export interface GeneratedImageResp {
  backend_used: string
  mime_type: string
  width: number
  height: number
  data_uri: string
}

export interface CheckItem {
  id: string
  name: string
  category: string
  status: 'ok' | 'warning' | 'unavailable'
  version: string | null
  location: string | null
  latency_ms: number
  warnings: string[]
}

export interface DoctorReport {
  generated_at: string
  os: string
  arch: string
  checks: CheckItem[]
  summary: {
    ok: number
    warning: number
    unavailable: number
  }
}

export interface ProviderManifest {
  id: string
  name: string
  tier: 'builtin' | 'custom'
  kind: 'cli' | 'http_open_ai' | 'mcp'
  privacy: 'local' | 'paid' | 'free' | 'anonymous'
  capabilities: string[]
  enabled: boolean
}

export interface SkillEntry {
  name: string
  description: string
  path: string
  source: string
}

export interface TelemetrySnapshot {
  total_tokens: number
  active_agents: number
  gate_pass_rate: number
  avg_latency_ms: number
  heatmap: { day: string; count: number }[]
  by_provider: {
    provider_id: string
    name: string
    calls: number
    success_rate: number
    p95_ms: number
    errors_429: number
    errors_403: number
    tokens: number
  }[]
  logs: {
    ts: string
    provider_id: string
    level: string
    message: string
  }[]
}

export interface ClaudeHubBridge {
  connected: boolean
  version: string | null
  url: string
  level: number | null
  level_label: string | null
  today_fresh: number | null
}

export interface PairingTicket {
  code: string
  qr_uri: string
  qr_svg: string
  expires_at: string
}

export interface NetworkInfo {
  local_url: string
  public_url: string | null
  lan_ip: string | null
  remote_enabled: boolean
}

export interface TotpStatus {
  enrolled: boolean
  remote_enabled: boolean
  bind: string
}

export interface TotpSetup {
  otpauth_uri: string
  qr_svg: string
  secret_base32: string
  enrolled: boolean
}

export interface DeviceView {
  id: string
  label: string
  created_at: string
  last_seen_at: string
  revoked: boolean
}

export interface CachedSnapshot {
  savedAt: string
  missions: MissionView[]
  telemetry: TelemetrySnapshot | null
  claudehub: ClaudeHubBridge | null
}
