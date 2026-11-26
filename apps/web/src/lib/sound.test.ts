import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  getSoundPreferences,
  notifyLoomEvent,
  playLoomSound,
  setSoundPreferences,
  subscribeSoundPreferences,
  type LoomAndroidBridge,
} from './sound'

describe('Loom sound & notification engine', () => {
  const storage = new Map<string, string>()
  const mockStorage: Storage = {
    get length() {
      return storage.size
    },
    clear: () => storage.clear(),
    getItem: (key: string) => storage.get(key) ?? null,
    key: (index: number) => [...storage.keys()][index] ?? null,
    removeItem: (key: string) => {
      storage.delete(key)
    },
    setItem: (key: string, value: string) => {
      storage.set(key, value)
    },
  }

  beforeEach(() => {
    storage.clear()
    vi.stubGlobal('localStorage', mockStorage)
    Object.defineProperty(window, 'localStorage', {
      value: mockStorage,
      configurable: true,
    })
    delete window.LoomAndroidBridge
  })

  afterEach(() => {
    delete window.LoomAndroidBridge
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
    storage.clear()
  })

  it('persists preferences under loom.sound.v1 and notifies subscribers', () => {
    expect(getSoundPreferences()).toEqual({
      enabled: true,
      notifications: true,
      volume: 0.45,
    })

    const listener = vi.fn()
    const unsubscribe = subscribeSoundPreferences(listener)

    const updated = setSoundPreferences({
      enabled: false,
      notifications: false,
      volume: 1.5,
    })

    expect(updated).toEqual({
      enabled: false,
      notifications: false,
      volume: 1,
    })
    expect(storage.get('loom.sound.v1')).toBe(JSON.stringify(updated))
    expect(listener).toHaveBeenCalledWith(updated)

    unsubscribe()
    setSoundPreferences({ enabled: true })
    expect(listener).toHaveBeenCalledTimes(1)
  })

  it('performs clean no-op without AudioContext and respects enabled: false', () => {
    vi.stubGlobal('AudioContext', undefined)
    vi.stubGlobal('webkitAudioContext', undefined)

    expect(() => playLoomSound('click')).not.toThrow()
    expect(() => playLoomSound('complete')).not.toThrow()

    const oscillatorSpy = vi.fn()
    class FakeAudioContext {
      state: AudioContextState = 'running'
      currentTime = 0
      destination = {} as AudioDestinationNode
      createOscillator() {
        oscillatorSpy()
        return {
          type: 'sine',
          frequency: { setValueAtTime: vi.fn(), exponentialRampToValueAtTime: vi.fn() },
          connect: vi.fn(),
          start: vi.fn(),
          stop: vi.fn(),
        }
      }
      createGain() {
        return {
          gain: {
            setValueAtTime: vi.fn(),
            linearRampToValueAtTime: vi.fn(),
            exponentialRampToValueAtTime: vi.fn(),
          },
          connect: vi.fn(),
        }
      }
      resume = vi.fn(async () => undefined)
    }

    vi.stubGlobal('AudioContext', FakeAudioContext)
    setSoundPreferences({ enabled: false })
    playLoomSound('dispatch')
    expect(oscillatorSpy).not.toHaveBeenCalled()
  })

  it('delegates sound cues, haptics and notifications to window.LoomAndroidBridge', () => {
    const bridge: LoomAndroidBridge = {
      notify: vi.fn(),
      playCue: vi.fn(),
      haptic: vi.fn(),
      getDeviceInfo: vi.fn(() => 'Pixel'),
    }
    window.LoomAndroidBridge = bridge

    notifyLoomEvent('Misión completada', 'apps/web', 'complete')

    expect(bridge.playCue).toHaveBeenCalledWith('complete')
    expect(bridge.haptic).toHaveBeenCalledWith('complete')
    expect(bridge.notify).toHaveBeenCalledWith('Misión completada', 'apps/web', 'complete')

    setSoundPreferences({ enabled: false, notifications: true })
    notifyLoomEvent('Parada total activada', 'Agentes detenidos', 'alert')

    expect(bridge.playCue).toHaveBeenCalledTimes(1)
    expect(bridge.haptic).toHaveBeenCalledTimes(1)
    expect(bridge.notify).toHaveBeenLastCalledWith(
      'Parada total activada',
      'Agentes detenidos',
      'alert',
    )
  })
})
