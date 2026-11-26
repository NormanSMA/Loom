import { useId } from 'react'
import { ChevronDown, Globe, Lock, Monitor, Moon, Sun, User } from 'lucide-react'
import { useI18n, type LocaleCode } from '../i18n'

export interface UserProfile {
  name: string
  role: string
  defaultSensitivity: 'internal' | 'confidential'
}

export interface SettingsViewProps {
  theme: 'dark' | 'light'
  onChangeTheme: (theme: 'dark' | 'light') => void
  profile: UserProfile
  onChangeProfile: (profile: UserProfile) => void
}

interface SettingsCopy {
  title: string
  subtitle: string
  profileTitle: string
  nameLabel: string
  namePlaceholder: string
  roleLabel: string
  rolePlaceholder: string
  sensitivityLabel: string
  interfaceTitle: string
  languageLabel: string
  themeLabel: string
  integrationTitle: string
  integrationBody: string
  integrationDesktopBadge: string
  integrationWebBadge: string
}

const SETTINGS_COPY: Record<LocaleCode, SettingsCopy> = {
  es: {
    title: 'Ajustes y perfil',
    subtitle: 'Preferencias locales del operador, idioma y apariencia de la interfaz.',
    profileTitle: 'Perfil local',
    nameLabel: 'Nombre',
    namePlaceholder: 'NormanSMA',
    roleLabel: 'Rol',
    rolePlaceholder: 'Arquitecto / Orquestador',
    sensitivityLabel: 'Sensibilidad predeterminada',
    interfaceTitle: 'Preferencias de interfaz',
    languageLabel: 'Idioma',
    themeLabel: 'Tema',
    integrationTitle: 'Integración de sistema (Desktop vs Web)',
    integrationBody:
      'La cápsula flotante (Dynamic Island / Notch estilo ClaudeHub) pertenece al overlay nativo de escritorio en Windows, macOS y Linux (Tauri v2). En la vista web, la actividad del agente se muestra integrada directamente encima del compositor de misiones.',
    integrationDesktopBadge: 'Desktop · Tauri v2 Overlay',
    integrationWebBadge: 'Web · Dock inline sobre Omnibox',
  },
  en: {
    title: 'Settings & profile',
    subtitle: 'Local operator preferences, language, and interface appearance.',
    profileTitle: 'Local profile',
    nameLabel: 'Name',
    namePlaceholder: 'NormanSMA',
    roleLabel: 'Role',
    rolePlaceholder: 'Architect / Orchestrator',
    sensitivityLabel: 'Default sensitivity',
    interfaceTitle: 'Interface preferences',
    languageLabel: 'Language',
    themeLabel: 'Theme',
    integrationTitle: 'System integration (Desktop vs Web)',
    integrationBody:
      'The floating capsule (ClaudeHub-style Dynamic Island / Notch) belongs to the native desktop overlay on Windows, macOS, and Linux (Tauri v2). On the web, agent activity is rendered inline directly above the mission composer.',
    integrationDesktopBadge: 'Desktop · Tauri v2 Overlay',
    integrationWebBadge: 'Web · Inline Omnibox dock',
  },
  'pt-BR': {
    title: 'Ajustes e perfil',
    subtitle: 'Preferências locais do operador, idioma e aparência da interface.',
    profileTitle: 'Perfil local',
    nameLabel: 'Nome',
    namePlaceholder: 'NormanSMA',
    roleLabel: 'Função',
    rolePlaceholder: 'Arquiteto / Orquestrador',
    sensitivityLabel: 'Sensibilidade padrão',
    interfaceTitle: 'Preferências de interface',
    languageLabel: 'Idioma',
    themeLabel: 'Tema',
    integrationTitle: 'Integração do sistema (Desktop vs Web)',
    integrationBody:
      'A cápsula flutuante (Dynamic Island / Notch estilo ClaudeHub) pertence ao overlay nativo de desktop no Windows, macOS e Linux (Tauri v2). Na web, a atividade do agente é exibida integrada diretamente acima do compositor de missões.',
    integrationDesktopBadge: 'Desktop · Overlay Tauri v2',
    integrationWebBadge: 'Web · Dock inline no Omnibox',
  },
}

export function SettingsView({
  theme,
  onChangeTheme,
  profile,
  onChangeProfile,
}: SettingsViewProps) {
  const { locale, setLocale, t } = useI18n()
  const copy = SETTINGS_COPY[locale]
  const nameId = useId()
  const roleId = useId()
  const languageId = useId()

  return (
    <div className="space-y-4">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">{copy.title}</h1>
        <p className="text-sm text-[var(--text-secondary)]">{copy.subtitle}</p>
      </div>

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <section className="loom-card rounded-2xl p-4 sm:p-5">
          <div className="flex items-center gap-2">
            <User className="h-4 w-4 shrink-0 text-[var(--accent-blue)]" aria-hidden="true" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">
              {copy.profileTitle}
            </h2>
          </div>

          <div className="mt-4 space-y-4">
            <div>
              <label
                htmlFor={nameId}
                className="mb-1.5 block font-mono text-xs text-[var(--text-secondary)]"
              >
                {copy.nameLabel}
              </label>
              <input
                id={nameId}
                type="text"
                value={profile.name}
                placeholder={copy.namePlaceholder}
                onChange={(e) => onChangeProfile({ ...profile, name: e.target.value })}
                className="loom-elevated min-h-11 w-full rounded-xl px-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              />
            </div>

            <div>
              <label
                htmlFor={roleId}
                className="mb-1.5 block font-mono text-xs text-[var(--text-secondary)]"
              >
                {copy.roleLabel}
              </label>
              <input
                id={roleId}
                type="text"
                value={profile.role}
                placeholder={copy.rolePlaceholder}
                onChange={(e) => onChangeProfile({ ...profile, role: e.target.value })}
                className="loom-elevated min-h-11 w-full rounded-xl px-3 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              />
            </div>

            <div>
              <p className="mb-1.5 font-mono text-xs text-[var(--text-secondary)]">
                {copy.sensitivityLabel}
              </p>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  aria-pressed={profile.defaultSensitivity === 'internal'}
                  onClick={() =>
                    onChangeProfile({ ...profile, defaultSensitivity: 'internal' })
                  }
                  className={`inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border px-3 text-xs font-medium transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                    profile.defaultSensitivity === 'internal'
                      ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)] text-[var(--accent-blue)]'
                      : 'border-[var(--border-subtle)] bg-[var(--bg-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Lock className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
                  {t('omnibox.sens.internal')}
                </button>

                <button
                  type="button"
                  aria-pressed={profile.defaultSensitivity === 'confidential'}
                  onClick={() =>
                    onChangeProfile({ ...profile, defaultSensitivity: 'confidential' })
                  }
                  className={`inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border px-3 text-xs font-medium transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                    profile.defaultSensitivity === 'confidential'
                      ? 'border-[var(--accent-amber)] bg-[var(--accent-amber-soft)] text-[var(--accent-amber)]'
                      : 'border-[var(--border-subtle)] bg-[var(--bg-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Lock className="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
                  {t('omnibox.sens.confidential')}
                </button>
              </div>
            </div>
          </div>
        </section>

        <section className="loom-card rounded-2xl p-4 sm:p-5">
          <div className="flex items-center gap-2">
            <Globe className="h-4 w-4 shrink-0 text-[var(--accent-green)]" aria-hidden="true" />
            <h2 className="text-sm font-semibold text-[var(--text-primary)]">
              {copy.interfaceTitle}
            </h2>
          </div>

          <div className="mt-4 space-y-4">
            <div>
              <label
                htmlFor={languageId}
                className="mb-1.5 block font-mono text-xs text-[var(--text-secondary)]"
              >
                {copy.languageLabel}
              </label>
              <div className="relative">
                <select
                  id={languageId}
                  aria-label="language"
                  value={locale}
                  onChange={(e) => setLocale(e.target.value as LocaleCode)}
                  className="loom-elevated min-h-11 w-full appearance-none rounded-xl pr-8 pl-3 text-sm text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
                >
                  <option value="es">Español</option>
                  <option value="en">English</option>
                  <option value="pt-BR">Português</option>
                </select>
                <ChevronDown
                  className="pointer-events-none absolute top-1/2 right-3 h-4 w-4 -translate-y-1/2 text-[var(--text-muted)]"
                  aria-hidden="true"
                />
              </div>
            </div>

            <div>
              <p className="mb-1.5 font-mono text-xs text-[var(--text-secondary)]">
                {copy.themeLabel}
              </p>
              <div className="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  aria-pressed={theme === 'dark'}
                  onClick={() => onChangeTheme('dark')}
                  className={`inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border px-3 text-xs font-medium transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                    theme === 'dark'
                      ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)] text-[var(--accent-blue)]'
                      : 'border-[var(--border-subtle)] bg-[var(--bg-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Moon className="h-4 w-4 shrink-0" aria-hidden="true" />
                  {t('theme.dark')}
                </button>

                <button
                  type="button"
                  aria-pressed={theme === 'light'}
                  onClick={() => onChangeTheme('light')}
                  className={`inline-flex min-h-11 items-center justify-center gap-2 rounded-xl border px-3 text-xs font-medium transition-colors focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)] ${
                    theme === 'light'
                      ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)] text-[var(--accent-blue)]'
                      : 'border-[var(--border-subtle)] bg-[var(--bg-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                  }`}
                >
                  <Sun className="h-4 w-4 shrink-0" aria-hidden="true" />
                  {t('theme.light')}
                </button>
              </div>
            </div>
          </div>
        </section>
      </div>

      <section className="loom-card rounded-2xl p-4 sm:p-5">
        <div className="flex items-center gap-2">
          <Monitor className="h-4 w-4 shrink-0 text-[var(--accent-blue)]" aria-hidden="true" />
          <h2 className="text-sm font-semibold text-[var(--text-primary)]">
            {copy.integrationTitle}
          </h2>
        </div>
        <p className="mt-2 text-sm leading-relaxed text-[var(--text-secondary)]">
          {copy.integrationBody}
        </p>
        <div className="mt-3 flex flex-wrap gap-2">
          <span className="loom-elevated inline-flex min-h-8 items-center rounded-full px-3 font-mono text-xs text-[var(--text-secondary)]">
            {copy.integrationDesktopBadge}
          </span>
          <span className="loom-elevated inline-flex min-h-8 items-center rounded-full px-3 font-mono text-xs text-[var(--accent-green)]">
            {copy.integrationWebBadge}
          </span>
        </div>
      </section>
    </div>
  )
}
