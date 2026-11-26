export interface SoundPreferences {
  enabled: boolean
  notifications: boolean
  volume: number
}

export type LoomSoundCue =
  | 'click'
  | 'dispatch'
  | 'stateChange'
  | 'complete'
  | 'alert'
  | 'test'

export type LoomNotificationKind = 'complete' | 'alert' | 'info'

export interface LoomAndroidBridge {
  notify(title: string, body: string, kind: string): void
  playCue(cue: string): void
  haptic(kind: string): void
  getDeviceInfo(): string
}

declare global {
  interface Window {
    LoomAndroidBridge?: LoomAndroidBridge
    webkitAudioContext?: typeof AudioContext
  }
}

const STORAGE_KEY = 'loom.sound.v1'
const PREFS_EVENT = 'loom:sound-prefs'

const DEFAULT_PREFERENCES: SoundPreferences = {
  enabled: true,
  notifications: true,
  volume: 0.45,
}

let sharedAudioContext: AudioContext | null = null

function clampVolume(value: number): number {
  if (!Number.isFinite(value)) return DEFAULT_PREFERENCES.volume
  return Math.min(1, Math.max(0, value))
}

export function getSoundPreferences(): SoundPreferences {
  if (typeof window === 'undefined') return { ...DEFAULT_PREFERENCES }
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) return { ...DEFAULT_PREFERENCES }
    const parsed = JSON.parse(raw) as Partial<SoundPreferences>
    return {
      enabled:
        typeof parsed.enabled === 'boolean'
          ? parsed.enabled
          : DEFAULT_PREFERENCES.enabled,
      notifications:
        typeof parsed.notifications === 'boolean'
          ? parsed.notifications
          : DEFAULT_PREFERENCES.notifications,
      volume:
        typeof parsed.volume === 'number'
          ? clampVolume(parsed.volume)
          : DEFAULT_PREFERENCES.volume,
    }
  } catch {
    return { ...DEFAULT_PREFERENCES }
  }
}

export function setSoundPreferences(
  partial: Partial<SoundPreferences>,
): SoundPreferences {
  const current = getSoundPreferences()
  const next: SoundPreferences = {
    enabled:
      typeof partial.enabled === 'boolean' ? partial.enabled : current.enabled,
    notifications:
      typeof partial.notifications === 'boolean'
        ? partial.notifications
        : current.notifications,
    volume:
      typeof partial.volume === 'number'
        ? clampVolume(partial.volume)
        : current.volume,
  }
  if (typeof window !== 'undefined') {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next))
    } catch {
      /* ignore storage errors */
    }
    try {
      window.dispatchEvent(new CustomEvent(PREFS_EVENT, { detail: next }))
    } catch {
      /* ignore event dispatch errors */
    }
  }
  return next
}

export function subscribeSoundPreferences(
  cb: (prefs: SoundPreferences) => void,
): () => void {
  if (typeof window === 'undefined') return () => {}

  const onPrefsEvent = () => {
    cb(getSoundPreferences())
  }
  const onStorage = (event: StorageEvent) => {
    if (event.key === null || event.key === STORAGE_KEY) {
      cb(getSoundPreferences())
    }
  }

  window.addEventListener(PREFS_EVENT, onPrefsEvent)
  window.addEventListener('storage', onStorage)
  return () => {
    window.removeEventListener(PREFS_EVENT, onPrefsEvent)
    window.removeEventListener('storage', onStorage)
  }
}

function getAudioContext(fromUserGesture: boolean): AudioContext | null {
  if (typeof window === 'undefined') return null
  const Ctor = window.AudioContext ?? window.webkitAudioContext
  if (typeof Ctor !== 'function') return null
  try {
    if (!sharedAudioContext || sharedAudioContext.state === 'closed') {
      sharedAudioContext = new Ctor()
    }
    if (sharedAudioContext.state === 'suspended') {
      void sharedAudioContext.resume().catch(() => {
        /* ignore autoplay policy rejection */
      })
      if (!fromUserGesture) {
        return null
      }
    }
    return sharedAudioContext
  } catch {
    return null
  }
}

function playTone(
  ctx: AudioContext,
  startTime: number,
  durationSec: number,
  startFreq: number,
  endFreq: number,
  peakGain: number,
  type: OscillatorType = 'sine',
): void {
  const osc = ctx.createOscillator()
  const gain = ctx.createGain()
  const endTime = startTime + durationSec

  osc.type = type
  osc.frequency.setValueAtTime(startFreq, startTime)
  if (endFreq !== startFreq) {
    osc.frequency.exponentialRampToValueAtTime(Math.max(20, endFreq), endTime)
  }

  gain.gain.setValueAtTime(0.0001, startTime)
  gain.gain.linearRampToValueAtTime(Math.max(0.0002, peakGain), startTime + Math.min(0.006, durationSec * 0.25))
  gain.gain.exponentialRampToValueAtTime(0.0001, endTime)

  osc.connect(gain)
  gain.connect(ctx.destination)

  osc.start(startTime)
  osc.stop(endTime + 0.005)
}

export function playLoomSound(
  cue: LoomSoundCue,
  options: { fromUserGesture?: boolean } = {},
): void {
  const prefs = getSoundPreferences()
  if (!prefs.enabled || prefs.volume <= 0) return

  const bridge =
    typeof window !== 'undefined' ? window.LoomAndroidBridge : undefined
  if (bridge) {
    try {
      bridge.playCue(cue)
    } catch {
      /* ignore Android bridge errors */
    }
    return
  }

  const ctx = getAudioContext(options.fromUserGesture ?? false)
  if (!ctx) return

  try {
    const now = ctx.currentTime
    const level = 0.22 * prefs.volume

    switch (cue) {
      case 'click': {
        playTone(ctx, now, 0.028, 680, 420, level * 0.75, 'triangle')
        break
      }
      case 'dispatch': {
        playTone(ctx, now, 0.12, 440, 660, level, 'sine')
        break
      }
      case 'stateChange': {
        playTone(ctx, now, 0.065, 520, 520, level * 0.7, 'sine')
        break
      }
      case 'complete':
      case 'test': {
        const triad = [523.25, 659.25, 783.99]
        triad.forEach((freq, idx) => {
          playTone(ctx, now + idx * 0.055, 0.13, freq, freq, level * 0.85, 'sine')
        })
        break
      }
      case 'alert': {
        playTone(ctx, now, 0.08, 240, 180, level, 'triangle')
        playTone(ctx, now + 0.1, 0.08, 240, 180, level, 'triangle')
        break
      }
    }
  } catch {
    /* ignore WebAudio runtime errors */
  }
}

export function notifyLoomEvent(
  title: string,
  body: string,
  kind: LoomNotificationKind,
  options: { skipAudio?: boolean } = {},
): void {
  const prefs = getSoundPreferences()
  const bridge =
    typeof window !== 'undefined' ? window.LoomAndroidBridge : undefined

  if (prefs.enabled && !options.skipAudio) {
    const cue: LoomSoundCue =
      kind === 'alert'
        ? 'alert'
        : kind === 'complete'
          ? 'complete'
          : 'stateChange'
    playLoomSound(cue)
    if (bridge) {
      try {
        bridge.haptic(kind)
      } catch {
        /* ignore Android bridge errors */
      }
    }
  }

  if (prefs.notifications) {
    if (bridge) {
      try {
        bridge.notify(title, body, kind)
      } catch {
        /* ignore Android bridge errors */
      }
      return
    }
    if (
      typeof Notification !== 'undefined' &&
      Notification.permission === 'granted'
    ) {
      try {
        new Notification(title, { body, icon: '/favicon.svg' })
        return
      } catch {
        /* fallback to ServiceWorker notification on mobile Chromium/PWA */
      }
      if (typeof navigator !== 'undefined' && 'serviceWorker' in navigator) {
        void navigator.serviceWorker
          .getRegistration()
          .then((reg) => reg?.showNotification(title, { body, icon: '/favicon.svg' }))
          .catch(() => {
            /* ignore ServiceWorker notification errors */
          })
      }
    }
  }
}

export async function requestNotificationPermission(): Promise<
  NotificationPermission | 'unsupported'
> {
  if (typeof window !== 'undefined' && window.LoomAndroidBridge) {
    return 'granted'
  }
  if (typeof Notification === 'undefined') {
    return 'unsupported'
  }
  if (Notification.permission === 'granted') {
    return 'granted'
  }
  try {
    return await Notification.requestPermission()
  } catch {
    return Notification.permission
  }
}
