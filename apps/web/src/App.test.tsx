import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import App from './App'
import { OfflineCanvas } from './components/OfflineCanvas'
import { I18nProvider } from './i18n'

describe('Loom UI shell', () => {
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
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null)
  })

  afterEach(() => {
    cleanup()
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
    storage.clear()
  })

  it('renders dashboard and switches language from es to en', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        json: async () => [],
      })),
    )

    render(<App />)
    await waitFor(() => expect(screen.getByText('Centro de misiones')).toBeTruthy())

    const select = screen.getByLabelText('language')
    fireEvent.change(select, { target: { value: 'en' } })
    expect(screen.getByText('Mission control')).toBeTruthy()
  })

  it('renders interactive offline standby screen with cached missions', () => {
    const onRetry = vi.fn()
    render(
      <I18nProvider>
        <OfflineCanvas
          lastSync="2026-11-26T00:00:00Z"
          cachedMissions={[
            {
              id: 'm1',
              title: 'Misión cacheada',
              mode: 'direct',
              effort: 'normal',
              sensitivity: 'internal',
              provider_id: 'claude',
              status: 'running',
              state: 'thinking',
              worktree: '.loom/worktrees/m1',
              gate_passed: true,
              updated_at: '2026-11-26T00:00:00Z',
            },
          ]}
          simulated={false}
          onRetry={onRetry}
          onExitSimulated={vi.fn()}
        />
      </I18nProvider>,
    )

    expect(screen.getByText('Misión cacheada')).toBeTruthy()
    fireEvent.click(screen.getByText('Reintentar conexión'))
    expect(onRetry).toHaveBeenCalledTimes(1)
  })
})
