import type { WorkspaceInfo } from '../types/api'
import { BrandIcon, normalizeBrandId, type BrandKey } from './BrandIcon'

export interface GlassFolderCardProps {
  workspace: WorkspaceInfo
  selected: boolean
  onClick: () => void
}

function collectBrandChips(workspace: WorkspaceInfo): { chips: BrandKey[]; extra: number } {
  const seen = new Set<BrandKey>()
  const add = (raw: string) => {
    if (!raw) return
    seen.add(normalizeBrandId(raw))
  }

  for (const src of workspace.sources) {
    add(src)
  }
  for (const chat of workspace.active_chats) {
    add(chat.source)
    add(chat.model)
  }

  if (seen.size === 0 && workspace.is_git_repo) {
    seen.add('git')
  }

  const all = [...seen]
  return {
    chips: all.slice(0, 4),
    extra: Math.max(0, all.length - 4),
  }
}

function compactPathHint(fullPath: string): string {
  const normalized = fullPath.replace(/\\/g, '/')
  const segments = normalized.split('/').filter(Boolean)
  if (segments.length <= 2) return normalized
  return `.../${segments.slice(-2).join('/')}`
}

export function GlassFolderCard({ workspace, selected, onClick }: GlassFolderCardProps) {
  const working = workspace.active_chats.some((chat) => chat.working)
  const chatCount = workspace.active_chats.length
  const { chips, extra } = collectBrandChips(workspace)
  const pathHint = compactPathHint(workspace.path)

  return (
    <button
      type="button"
      aria-pressed={selected}
      onClick={onClick}
      className="group relative flex min-h-11 min-w-44 shrink-0 flex-col pt-3 text-left transition-transform duration-150 hover:-translate-y-0.5 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent-blue)] motion-reduce:transition-none motion-reduce:transform-none"
    >
      {/* Folder top-left tab */}
      <span
        aria-hidden="true"
        className={`relative z-10 ml-0.5 h-2.5 w-16 rounded-t-lg border-x border-t transition-colors duration-150 motion-reduce:transition-none ${
          selected
            ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)]'
            : 'border-[var(--border-strong)] bg-[var(--bg-elevated)] group-hover:border-[var(--text-muted)]'
        }`}
      />

      {/* Peeking translucent document sheets */}
      <span
        aria-hidden="true"
        className="pointer-events-none absolute top-1.5 right-3 left-4 z-0 h-5 rounded-t-lg border border-[var(--border-subtle)] bg-[var(--text-primary)]/10 transition-transform duration-150 group-hover:-translate-y-0.5 motion-reduce:transition-none motion-reduce:transform-none"
      />
      <span
        aria-hidden="true"
        className="pointer-events-none absolute top-2.5 right-2 left-2.5 z-0 h-5 rounded-t-lg border border-[var(--border-subtle)] bg-[var(--text-primary)]/5 transition-transform duration-150 group-hover:-translate-y-0.5 motion-reduce:transition-none motion-reduce:transform-none"
      />

      {/* Smoked-glass front folder body */}
      <div
        className={`relative z-10 flex w-full flex-col justify-between gap-2.5 rounded-2xl rounded-tl-none border px-3 py-2.5 backdrop-blur-md transition-colors duration-150 motion-reduce:transition-none ${
          selected
            ? 'border-[var(--accent-blue)] bg-[var(--accent-blue-soft)] shadow-sm'
            : 'border-[var(--border-strong)] bg-[var(--bg-glass)] group-hover:border-[var(--text-muted)]'
        }`}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0 flex-1">
            <p
              className={`truncate text-xs font-semibold ${
                selected ? 'text-[var(--accent-blue)]' : 'text-[var(--text-primary)]'
              }`}
            >
              {workspace.name}
            </p>
            <p className="truncate font-mono text-xs text-[var(--text-muted)]" title={workspace.path}>
              {pathHint}
            </p>
          </div>

          <div className="flex shrink-0 items-center gap-1.5">
            <span
              aria-hidden="true"
              className={`h-2 w-2 rounded-full ${
                working ? 'bg-[var(--accent-green)] glyph-pulse' : 'bg-[var(--text-muted)]'
              }`}
            />
            <span className="rounded-full border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-1.5 py-0.5 font-mono text-xs leading-none text-[var(--text-secondary)] tabular-nums">
              {chatCount}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-1.5">
          {chips.map((brand) => (
            <span
              key={brand}
              title={brand}
              className="inline-flex h-6 w-6 items-center justify-center rounded-md border border-[var(--border-subtle)] bg-[var(--bg-elevated)] text-[var(--text-secondary)]"
            >
              <BrandIcon id={brand} className="h-3.5 w-3.5" />
            </span>
          ))}
          {extra > 0 && (
            <span className="rounded-md border border-[var(--border-subtle)] bg-[var(--bg-elevated)] px-1.5 py-0.5 font-mono text-xs text-[var(--text-muted)] tabular-nums">
              +{extra}
            </span>
          )}
        </div>
      </div>
    </button>
  )
}
