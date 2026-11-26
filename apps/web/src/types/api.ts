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
  gate_passed: boolean | null
  updated_at: string
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
  expires_at: string
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
