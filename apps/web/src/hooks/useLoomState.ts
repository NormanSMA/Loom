import { useCallback, useEffect, useState } from 'react'
import { notifyLoomEvent, playLoomSound } from '../lib/sound'
import type {
  ArtifactPreview,
  CachedSnapshot,
  ClaudeHubBridge,
  DeviceView,
  DoctorReport,
  GeneratedImageResp,
  HealthInfo,
  LiveEvent,
  MissionDiff,
  MissionView,
  NetworkInfo,
  PairingTicket,
  ProviderManifest,
  SkillEntry,
  TelemetrySnapshot,
  TotpSetup,
  TotpStatus,
  WorkspaceInfo,
} from '../types/api'

const SNAPSHOT_KEY = 'loom.offline.snapshot.v1'
const SESSION_KEY = 'loom.session.v1'

function detectDeviceLabel(): string {
  const ua = typeof navigator !== 'undefined' ? navigator.userAgent : ''
  if (/iPhone/i.test(ua)) return 'iPhone'
  if (/iPad/i.test(ua)) return 'iPad'
  if (/Android/i.test(ua)) return 'Android'
  return 'Escritorio'
}

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
    let message = `HTTP ${resp.status}`
    try {
      const errBody = (await resp.json()) as { error?: unknown }
      if (typeof errBody.error === 'string' && errBody.error.trim()) {
        message = errBody.error
      }
    } catch {
      /* ignore non-JSON error */
    }
    throw new Error(message)
  }
  return (await resp.json()) as T
}

export function useLoomState(options: { enableAudio?: boolean } = {}) {
  const enableAudio = options.enableAudio ?? true
  const initial = loadCached()
  const [online, setOnline] = useState(true)
  const [simulatedOffline, setSimulatedOffline] = useState(false)
  const [health, setHealth] = useState<HealthInfo | null>(null)
  const [doctor, setDoctor] = useState<DoctorReport | null>(null)
  const [providers, setProviders] = useState<ProviderManifest[]>([])
  const [workspaces, setWorkspaces] = useState<WorkspaceInfo[]>([])
  const [skills, setSkills] = useState<SkillEntry[]>([])
  const [missions, setMissions] = useState<MissionView[]>(initial?.missions ?? [])
  const [telemetry, setTelemetry] = useState<TelemetrySnapshot | null>(initial?.telemetry ?? null)
  const [claudehub, setClaudehub] = useState<ClaudeHubBridge | null>(initial?.claudehub ?? null)
  const [devices, setDevices] = useState<DeviceView[]>([])
  const [network, setNetwork] = useState<NetworkInfo | null>(null)
  const [pairedDevice, setPairedDevice] = useState<DeviceView | null>(null)
  const [pairingNotice, setPairingNotice] = useState<'ok' | 'error' | null>(null)
  const [generatedImage, setGeneratedImage] = useState<GeneratedImageResp | null>(null)
  const [imageError, setImageError] = useState<string | null>(null)
  const [totpStatus, setTotpStatus] = useState<TotpStatus>({
    enrolled: false,
    remote_enabled: false,
    bind: '127.0.0.1',
  })
  const [liveEvent, setLiveEvent] = useState<LiveEvent | null>(null)
  const [lastSync, setLastSync] = useState<string | null>(initial?.savedAt ?? null)

  const refresh = useCallback(async () => {
    try {
      const [h, d, p, s, m, tel, ch, dev, totp, net, ws] = await Promise.all([
        fetchJson<HealthInfo>('/api/health'),
        fetchJson<DoctorReport>('/api/doctor'),
        fetchJson<ProviderManifest[]>('/api/providers'),
        fetchJson<SkillEntry[]>('/api/skills'),
        fetchJson<MissionView[]>('/api/missions'),
        fetchJson<TelemetrySnapshot>('/api/telemetry'),
        fetchJson<ClaudeHubBridge>('/api/claudehub'),
        fetchJson<DeviceView[]>('/api/devices'),
        fetchJson<Partial<TotpStatus>>('/api/auth/totp/status').catch((): Partial<TotpStatus> => ({})),
        fetchJson<Partial<NetworkInfo>>('/api/network').catch((): Partial<NetworkInfo> => ({})),
        fetchJson<WorkspaceInfo[]>('/api/workspaces').catch((): WorkspaceInfo[] => []),
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
      setWorkspaces(Array.isArray(ws) ? ws : [])
      if (typeof net?.local_url === 'string') {
        setNetwork({
          local_url: net.local_url,
          public_url: net.public_url ?? null,
          lan_ip: net.lan_ip ?? null,
          remote_enabled: Boolean(net.remote_enabled),
        })
      }
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
    const code = new URLSearchParams(window.location.search).get('pair')?.trim()
    if (!code) return
    let active = true
    const runExchange = async () => {
      try {
        const res = await fetchJson<{ device: DeviceView; session_token: string }>('/api/auth/exchange', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ code, label: detectDeviceLabel() }),
        })
        if (!active) return
        try {
          window.localStorage.setItem(
            SESSION_KEY,
            JSON.stringify({ token: res.session_token, device: res.device }),
          )
        } catch {
          /* ignore storage unavailability */
        }
        setPairedDevice(res.device)
        setPairingNotice('ok')
        playLoomSound('complete')
        void refresh()
      } catch {
        if (active) setPairingNotice('error')
      } finally {
        if (active) {
          window.history.replaceState({}, '', window.location.pathname)
        }
      }
    }
    void runExchange()
    return () => {
      active = false
    }
  }, [refresh])

  useEffect(() => {
    if (!online || simulatedOffline) return
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    let ws: WebSocket | null = null
    try {
      ws = new WebSocket(`${proto}//${window.location.host}/api/ws`)
      let prevLiveState = ''
      ws.onmessage = (ev) => {
        try {
          const parsed = JSON.parse(String(ev.data)) as LiveEvent
          if (
            parsed.kind === 'agent_activity' ||
            parsed.kind === 'mission_created' ||
            parsed.kind === 'mission_completed' ||
            parsed.kind === 'mission_cancelled'
          ) {
            setLiveEvent(parsed)
          }
          if (enableAudio) {
            if (parsed.kind === 'mission_completed') {
              notifyLoomEvent(
                'Misión completada',
                parsed.target ?? parsed.detail ?? 'Loom',
                'complete',
              )
            } else if (parsed.kind === 'mission_cancelled') {
              playLoomSound('alert')
            } else {
              const nextState: string = parsed.state ?? ''
              if (
                nextState &&
                nextState !== prevLiveState &&
                (nextState === 'reading' ||
                  nextState === 'thinking' ||
                  nextState === 'analysing' ||
                  nextState === 'debugging')
              ) {
                prevLiveState = nextState
                playLoomSound('stateChange')
              }
            }
          }
        } catch {
          /* ignore non-event payloads */
        }
      }
    } catch {
      /* ignore ws errors in test jsdom */
    }
    return () => ws?.close()
  }, [online, simulatedOffline, enableAudio])

  const generateImage = async (payload: {
    prompt: string
    style_context?: string
    mission_id?: string
  }) => {
    setImageError(null)
    try {
      const resp = await fetchJson<GeneratedImageResp>('/api/images/generate', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(payload),
      })
      setGeneratedImage(resp)
      return resp
    } catch (err) {
      setImageError(err instanceof Error ? err.message : 'No se pudo generar la imagen')
      return null
    }
  }

  const createMission = async (payload: {
    title: string
    mode: string
    effort: string
    sensitivity: string
    repo_root?: string
  }) => {
    const created = await fetchJson<MissionView>('/api/missions', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload),
    })
    playLoomSound('dispatch')
    setMissions((prev) => [created, ...prev])
    if (payload.mode === 'image') {
      await generateImage({ prompt: payload.title, mission_id: created.id })
    }
  }

  const fetchMissionDiff = (id: string) =>
    fetchJson<MissionDiff>(`/api/missions/${encodeURIComponent(id)}/diff`)

  const fetchMissionArtifacts = (id: string) =>
    fetchJson<ArtifactPreview[]>(`/api/missions/${encodeURIComponent(id)}/artifacts`)

  const decideMission = async (id: string, decision: 'approve' | 'reject') => {
    const updated = await fetchJson<MissionView>(
      `/api/missions/${encodeURIComponent(id)}/decision`,
      {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ decision }),
      },
    )
    setMissions((prev) => prev.map((m) => (m.id === id ? updated : m)))
    if (decision === 'approve') {
      notifyLoomEvent('Misión aprobada', updated.title, 'complete')
    } else {
      playLoomSound('alert')
    }
    return updated
  }

  const createPairing = (baseUrl?: string) =>
    fetchJson<PairingTicket>('/api/auth/pair', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ base_url: baseUrl ?? network?.public_url ?? window.location.origin }),
    })

  const revokeDevice = async (id: string) => {
    await fetchJson('/api/devices/' + encodeURIComponent(id), { method: 'DELETE' })
    setDevices((prev) => prev.map((d) => (d.id === id ? { ...d, revoked: true } : d)))
  }

  const emergencyStop = async () => {
    const resp = await fetchJson<{ stopped?: number }>('/api/emergency-stop', { method: 'POST' })
    const count = typeof resp?.stopped === 'number' ? resp.stopped : 0
    if (enableAudio) {
      notifyLoomEvent(
        'Parada total activada',
        count > 0 ? `${count} agente(s) detenidos` : 'Sin agentes activos',
        'alert',
      )
    }
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
    workspaces,
    skills,
    missions,
    telemetry,
    claudehub,
    devices,
    network,
    pairedDevice,
    pairingNotice,
    dismissPairingNotice: () => setPairingNotice(null),
    generatedImage,
    imageError,
    dismissGeneratedImage: () => {
      setGeneratedImage(null)
      setImageError(null)
    },
    totpStatus,
    liveEvent,
    lastSync,
    refresh,
    createMission,
    fetchMissionDiff,
    fetchMissionArtifacts,
    decideMission,
    generateImage,
    createPairing,
    revokeDevice,
    setupTotp,
    verifyTotp,
    emergencyStop,
  }
}
