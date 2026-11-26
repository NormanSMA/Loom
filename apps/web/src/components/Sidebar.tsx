import { Activity, BookOpen, Cpu, Layers, Settings, ShieldAlert } from 'lucide-react'
import { useI18n, type LocaleCode, type MessageKey } from '../i18n'

export type NavTab =
  | 'missions'
  | 'connectors'
  | 'telemetry'
  | 'knowledge'
  | 'security'
  | 'settings'

interface SidebarProps {
  activeTab: NavTab
  onSelect: (tab: NavTab) => void
  onNewMission: () => void
  profileName?: string
}

const PRIMARY_ITEMS: { id: Exclude<NavTab, 'settings'>; label: MessageKey; icon: typeof Layers }[] = [
  { id: 'missions', label: 'nav.missions', icon: Layers },
  { id: 'connectors', label: 'nav.connectors', icon: Cpu },
  { id: 'telemetry', label: 'nav.telemetry', icon: Activity },
  { id: 'knowledge', label: 'nav.knowledge', icon: BookOpen },
  { id: 'security', label: 'nav.security', icon: ShieldAlert },
]

const SETTINGS_LABEL: Record<LocaleCode, string> = {
  es: 'Ajustes',
  en: 'Settings',
  'pt-BR': 'Ajustes',
}

export function Sidebar({
  activeTab,
  onSelect,
  onNewMission,
  profileName,
}: SidebarProps) {
  const { locale, t } = useI18n()
  const settingsText = SETTINGS_LABEL[locale]
  const resolvedProfile = profileName?.trim() || 'NormanSMA'

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
            className="loom-elevated mb-4 flex min-h-11 w-full items-center justify-between rounded-xl px-3 text-sm font-medium text-[var(--text-primary)] transition-colors hover:border-[var(--accent-blue)] active:scale-[0.99] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
          >
            <span>+ {t('action.newMission')}</span>
            <kbd className="rounded border border-[var(--border-subtle)] px-1.5 py-0.5 font-mono text-xs text-[var(--text-muted)]">
              N
            </kbd>
          </button>

          <nav className="space-y-1" aria-label="Principal">
            {PRIMARY_ITEMS.map((item) => {
              const Icon = item.icon
              const active = activeTab === item.id
              return (
                <button
                  key={item.id}
                  type="button"
                  aria-current={active ? 'page' : undefined}
                  onClick={() => onSelect(item.id)}
                  className={`flex min-h-11 w-full items-center justify-between rounded-xl px-3 text-sm transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                    active
                      ? 'loom-elevated font-medium text-[var(--text-primary)]'
                      : 'text-[var(--text-secondary)] hover:bg-[var(--bg-elevated)]/60 hover:text-[var(--text-primary)]'
                  }`}
                >
                  <span className="flex items-center gap-2.5">
                    <Icon className="h-4 w-4 shrink-0" aria-hidden="true" />
                    {t(item.label)}
                  </span>
                  {active && <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent-green)]" />}
                </button>
              )
            })}

            <div className="my-2 border-t border-[var(--border-subtle)] pt-2">
              <button
                type="button"
                aria-current={activeTab === 'settings' ? 'page' : undefined}
                onClick={() => onSelect('settings')}
                className={`flex min-h-11 w-full items-center justify-between rounded-xl px-3 text-sm transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                  activeTab === 'settings'
                    ? 'loom-elevated font-medium text-[var(--text-primary)]'
                    : 'text-[var(--text-secondary)] hover:bg-[var(--bg-elevated)]/60 hover:text-[var(--text-primary)]'
                }`}
              >
                <span className="flex items-center gap-2.5">
                  <Settings className="h-4 w-4 shrink-0" aria-hidden="true" />
                  {settingsText}
                </span>
                {activeTab === 'settings' && (
                  <span className="h-1.5 w-1.5 rounded-full bg-[var(--accent-green)]" />
                )}
              </button>
            </div>
          </nav>
        </div>

        <div className="border-t border-[var(--border-subtle)] px-2 pt-3">
          <p className="truncate text-xs font-medium text-[var(--text-primary)]">
            {resolvedProfile}
          </p>
          <p className="mt-0.5 font-mono text-xs text-[var(--text-muted)]">127.0.0.1:4320</p>
        </div>
      </aside>

      <nav
        aria-label="Móvil"
        className="loom-card fixed inset-x-0 bottom-0 z-30 flex items-stretch justify-around border-x-0 border-b-0 bg-[var(--bg-surface)]/95 px-1 pt-1.5 pb-[max(0.45rem,env(safe-area-inset-bottom))] backdrop-blur-md md:hidden"
      >
        {PRIMARY_ITEMS.map((item) => {
          const Icon = item.icon
          const active = activeTab === item.id
          return (
            <button
              key={item.id}
              type="button"
              aria-current={active ? 'page' : undefined}
              onClick={() => onSelect(item.id)}
              className={`relative flex min-h-11 min-w-0 flex-1 flex-col items-center justify-center rounded-xl px-1 text-xs transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
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
