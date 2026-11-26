import { useCallback, useEffect, useState } from 'react'
import type {
  CachedSnapshot,
  ClaudeHubBridge,
  DeviceView,
  DoctorReport,
  HealthInfo,
  LiveEvent,
  MissionView,
  PairingTicket,
  ProviderManifest,
  SkillEntry,
  TelemetrySnapshot,
  TotpSetup,
  TotpStatus,
} from '../types/api'

const SNAPSHOT_KEY = 'loom.offline.snapshot.v1'

function loadCached(): CachedSnapshot | null {
  try {
    const raw = window.localStorage.getItem(SNAPSHOT_KEY)
    if (!raw) return null
    return JSON.parse(raw) as CachedSnapshot
  } catch {
    return null
  }
}

function saveCached(snap: CachedSnapshot) {
  try {
    window.localStorage.setItem(SNAPSHOT_KEY, JSON.stringify(snap))
  } catch {
    /* ignore storage quota */
  }
}

async function fetchJson<T>(url: string, init?: RequestInit): Promise<T> {
  const resp = await fetch(url, init)
  if (!resp.ok) {
    throw new Error(`HTTP ${resp.status}`)
  }
  return (await resp.json()) as T
}

export function useLoomState() {
  const initial = loadCached()
  const [online, setOnline] = useState(true)
  const [simulatedOffline, setSimulatedOffline] = useState(false)
  const [health, setHealth] = useState<HealthInfo | null>(null)
  const [doctor, setDoctor] = useState<DoctorReport | null>(null)
  const [providers, setProviders] = useState<ProviderManifest[]>([])
  const [skills, setSkills] = useState<SkillEntry[]>([])
  const [missions, setMissions] = useState<MissionView[]>(initial?.missions ?? [])
  const [telemetry, setTelemetry] = useState<TelemetrySnapshot | null>(initial?.telemetry ?? null)
  const [claudehub, setClaudehub] = useState<ClaudeHubBridge | null>(initial?.claudehub ?? null)
  const [devices, setDevices] = useState<DeviceView[]>([])
  const [totpStatus, setTotpStatus] = useState<TotpStatus>({
    enrolled: false,
    remote_enabled: false,
    bind: '127.0.0.1',
  })
  const [liveEvent, setLiveEvent] = useState<LiveEvent | null>(null)
  const [lastSync, setLastSync] = useState<string | null>(initial?.savedAt ?? null)

  const refresh = useCallback(async () => {
    try {
      const [h, d, p, s, m, tel, ch, dev, totp] = await Promise.all([
        fetchJson<HealthInfo>('/api/health'),
        fetchJson<DoctorReport>('/api/doctor'),
        fetchJson<ProviderManifest[]>('/api/providers'),
        fetchJson<SkillEntry[]>('/api/skills'),
        fetchJson<MissionView[]>('/api/missions'),
        fetchJson<TelemetrySnapshot>('/api/telemetry'),
        fetchJson<ClaudeHubBridge>('/api/claudehub'),
        fetchJson<DeviceView[]>('/api/devices'),
        fetchJson<Partial<TotpStatus>>('/api/auth/totp/status').catch((): Partial<TotpStatus> => ({})),
      ])
      const now = new Date().toISOString()
      setHealth(h)
      setDoctor(d)
      setProviders(p)
      setSkills(s)
      setMissions(m)
      setTelemetry(tel)
      setClaudehub(ch)
      setDevices(dev)
      if (typeof totp?.enrolled === 'boolean') {
        setTotpStatus({
          enrolled: totp.enrolled,
          remote_enabled: Boolean(totp.remote_enabled),
          bind: typeof totp.bind === 'string' && totp.bind ? totp.bind : '127.0.0.1',
        })
      }
      setOnline(true)
      setLastSync(now)
      saveCached({ savedAt: now, missions: m, telemetry: tel, claudehub: ch })
    } catch {
      setOnline(false)
    }
  }, [])

  useEffect(() => {
    void refresh()
    const timer = window.setInterval(() => void refresh(), 25_000)
    return () => window.clearInterval(timer)
  }, [refresh])

  useEffect(() => {
    if (!online || simulatedOffline) return
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    let ws: WebSocket | null = null
    try {
      ws = new WebSocket(`${proto}//${window.location.host}/api/ws`)
      ws.onmessage = (ev) => {
        try {
          const parsed = JSON.parse(String(ev.data)) as LiveEvent
          if (parsed.kind === 'agent_activity' || parsed.kind === 'mission_created') {
            setLiveEvent(parsed)
          }
        } catch {
          /* ignore non-event payloads */
        }
      }
    } catch {
      /* ignore ws errors in test jsdom */
    }
    return () => ws?.close()
  }, [online, simulatedOffline])

  const createMission = async (payload: {
    title: string
    mode: string
    effort: string
    sensitivity: string
  }) => {
    const created = await fetchJson<MissionView>('/api/missions', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload),
    })
    setMissions((prev) => [created, ...prev])
  }

  const createPairing = () =>
    fetchJson<PairingTicket>('/api/auth/pair', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ base_url: window.location.origin }),
    })

  const revokeDevice = async (id: string) => {
    await fetchJson('/api/devices/' + encodeURIComponent(id), { method: 'DELETE' })
    setDevices((prev) => prev.map((d) => (d.id === id ? { ...d, revoked: true } : d)))
  }

  const emergencyStop = async () => {
    await fetchJson('/api/emergency-stop', { method: 'POST' })
    void refresh()
  }

  const setupTotp = () =>
    fetchJson<TotpSetup>('/api/auth/totp/setup', { method: 'POST' })

  const verifyTotp = async (code: string) => {
    const resp = await fetchJson<{ verified: boolean; enrolled: boolean }>('/api/auth/totp/verify', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ code }),
    })
    setTotpStatus((prev) => ({ ...prev, enrolled: resp.enrolled }))
    return resp.verified
  }

  return {
    isOffline: !online || simulatedOffline,
    simulatedOffline,
    setSimulatedOffline,
    health,
    doctor,
    providers,
    skills,
    missions,
    telemetry,
    claudehub,
    devices,
    totpStatus,
    liveEvent,
    lastSync,
    refresh,
    createMission,
    createPairing,
    revokeDevice,
    setupTotp,
    verifyTotp,
    emergencyStop,
  }
}
