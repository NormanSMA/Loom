import { Activity, BookOpen, Cpu, Layers, ShieldAlert } from 'lucide-react'
import { useI18n, type MessageKey } from '../i18n'

export type NavTab = 'missions' | 'connectors' | 'telemetry' | 'knowledge' | 'security'

interface SidebarProps {
  activeTab: NavTab
  onSelect: (tab: NavTab) => void
  onNewMission: () => void
}

const ITEMS: { id: NavTab; label: MessageKey; icon: typeof Layers }[] = [
  { id: 'missions', label: 'nav.missions', icon: Layers },
  { id: 'connectors', label: 'nav.connectors', icon: Cpu },
  { id: 'telemetry', label: 'nav.telemetry', icon: Activity },
  { id: 'knowledge', label: 'nav.knowledge', icon: BookOpen },
  { id: 'security', label: 'nav.security', icon: ShieldAlert },
]

export function Sidebar({ activeTab, onSelect, onNewMission }: SidebarProps) {
  const { t } = useI18n()

  return (
    <>
      <aside className="loom-card hidden w-64 shrink-0 flex-col justify-between border-y-0 border-l-0 p-4 md:flex">
        <div>
          <div className="flex items-center justify-between px-1 pb-4">
            <div>
              <p className="font-mono text-base font-bold tracking-widest text-[var(--text-primary)]">
                {t('app.name').toUpperCase()}
              </p>
              <p className="font-mono text-xs text-[var(--text-muted)]">{t('app.subtitle')}</p>
            </div>
            <span className="h-2.5 w-2.5 rounded-full bg-[var(--accent-green)]" />
          </div>

          <button
            type="button"
            onClick={onNewMission}
            className="loom-elevated mb-4 flex min-h-11 w-full items-center justify-between rounded-xl px-3 text-sm font-medium text-[var(--text-primary)] transition-colors hover:border-[var(--accent-blue)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <span>+ {t('action.newMission')}</span>
            <kbd className="rounded border border-[var(--border-subtle)] px-1.5 py-0.5 font-mono text-[10px] text-[var(--text-muted)]">
              N
            </kbd>
          </button>

          <nav className="space-y-1" aria-label="Principal">
            {ITEMS.map((item) => {
              const Icon = item.icon
              const active = activeTab === item.id
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => onSelect(item.id)}
                  className={`flex min-h-11 w-full items-center justify-between rounded-xl px-3 text-sm transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                    active
                      ? 'loom-elevated font-medium text-[var(--text-primary)]'
                      : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <span className="flex items-center gap-2.5">
                    <Icon className="h-4 w-4" aria-hidden="true" />
                    {t(item.label)}
                  </span>
                  {active && <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent-green)]" />}
                </button>
              )
            })}
          </nav>
        </div>

        <p className="px-2 font-mono text-[11px] text-[var(--text-muted)]">
          127.0.0.1:4320 · local-only
        </p>
      </aside>

      <nav
        aria-label="Móvil"
        className="loom-card fixed inset-x-0 bottom-0 z-30 flex items-stretch justify-around border-x-0 border-b-0 bg-[var(--bg-surface)]/95 px-1 pt-1.5 pb-[max(0.45rem,env(safe-area-inset-bottom))] backdrop-blur-md md:hidden"
      >
        {ITEMS.map((item) => {
          const Icon = item.icon
          const active = activeTab === item.id
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => onSelect(item.id)}
              className={`relative flex min-h-11 min-w-0 flex-1 flex-col items-center justify-center rounded-xl px-1 text-[11px] transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                active ? 'font-medium text-[var(--accent-blue)]' : 'text-[var(--text-muted)]'
              }`}
            >
              {active && (
                <span className="absolute top-0.5 h-0.5 w-5 rounded-full bg-[var(--accent-blue)]" />
              )}
              <Icon className="h-4 w-4 shrink-0" aria-hidden="true" />
              <span className="mt-0.5 max-w-full truncate">{t(item.label)}</span>
            </button>
          )
        })}
      </nav>
    </>
  )
}
