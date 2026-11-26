import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import App from './App'
import { OfflineCanvas } from './components/OfflineCanvas'
import { I18nProvider } from './i18n'

describe('Loom UI shell', () => {
  it('renders dashboard and switches language from es to en', () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        json: async () => [],
      })),
    )

    render(<App />)
    expect(screen.getByText('Centro de misiones')).toBeTruthy()

    const select = screen.getByLabelText('language')
    fireEvent.change(select, { target: { value: 'en' } })
    expect(screen.getByText('Mission control')).toBeTruthy()

    vi.unstubAllGlobals()
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
              title: 'Mision cacheada',
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

    expect(screen.getByText('Mision cacheada')).toBeTruthy()
    fireEvent.click(screen.getByText('Reintentar conexion'))
    expect(onRetry).toHaveBeenCalledTimes(1)
  })
})
