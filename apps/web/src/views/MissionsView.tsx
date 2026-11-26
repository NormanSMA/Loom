import { useState } from 'react'
import { motion } from 'motion/react'
import { Check, ChevronDown, ChevronUp, FolderGit2, Image as ImageIcon, MessageSquareCode, X } from 'lucide-react'
import { useI18n } from '../i18n'
import type {
  ArtifactPreview,
  GeneratedImageResp,
  LiveEvent,
  MissionDiff,
  MissionView,
  WorkspaceInfo,
} from '../types/api'
import { AgentGlyph } from '../components/AgentGlyph'
import { GlassFolderCard } from '../components/GlassFolderCard'
import { Omnibox } from '../components/Omnibox'

interface MissionsViewProps {
  missions: MissionView[]
  workspaces?: WorkspaceInfo[]
  generatedImage?: GeneratedImageResp | null
  imageError?: string | null
  defaultSensitivity?: 'internal' | 'confidential'
  liveEvent?: LiveEvent | null
  onDismissImage?: () => void
  onCreateMission: (payload: {
    title: string
    mode: string
    effort: string
    sensitivity: string
    repo_root?: string
  }) => Promise<void>
  onFetchDiff?: (id: string) => Promise<MissionDiff>
  onFetchArtifacts?: (id: string) => Promise<ArtifactPreview[]>
  onDecideMission?: (id: string, decision: 'approve' | 'reject') => Promise<MissionView>
}

type StatusKey =
  | 'status.thinking'
  | 'status.reading'
  | 'status.analysing'
  | 'status.debugging'
  | 'status.creating'
  | 'status.idle'
  | 'status.approved'
  | 'status.rejected'

const STATUS_KEY_MAP: Record<string, StatusKey> = {
  thinking: 'status.thinking',
  reading: 'status.reading',
  analysing: 'status.analysing',
  debugging: 'status.debugging',
  creating: 'status.creating',
  idle: 'status.idle',
  approved: 'status.approved',
  rejected: 'status.rejected',
}

export function MissionsView({
  missions,
  workspaces = [],
  generatedImage = null,
  imageError = null,
  defaultSensitivity = 'internal',
  liveEvent = null,
  onDismissImage,
  onCreateMission,
  onFetchDiff,
  onFetchArtifacts,
  onDecideMission,
}: MissionsViewProps) {
  const { t } = useI18n()
  const [selectedRepo, setSelectedRepo] = useState<string | null>(null)
  const [showChats, setShowChats] = useState(false)
  const [expanded, setExpanded] = useState<Record<string, boolean>>({})
  const [diffs, setDiffs] = useState<Record<string, MissionDiff>>({})
  const [artifactsByMission, setArtifactsByMission] = useState<Record<string, ArtifactPreview[]>>({})

  const activeChats = workspaces.flatMap((ws) =>
    ws.active_chats.map((chat) => ({ ...chat, workspace: ws.name, path: ws.path })),
  )

  const toggleDetail = async (id: string) => {
    const next = !expanded[id]
    setExpanded((prev) => ({ ...prev, [id]: next }))
    if (!next) return
    if (onFetchDiff && !diffs[id]) {
      try {
        const diff = await onFetchDiff(id)
        setDiffs((prev) => ({ ...prev, [id]: diff }))
      } catch {
        /* ignore fetch error */
      }
    }
    if (onFetchArtifacts) {
      try {
        const arts = await onFetchArtifacts(id)
        setArtifactsByMission((prev) => ({ ...prev, [id]: arts }))
      } catch {
        /* ignore fetch error */
      }
    }
  }

  return (
    <div className="space-y-5">
      <div>
        <h1 className="text-xl font-semibold text-[var(--text-primary)]">
          {t('missions.title')}
        </h1>
        <p className="text-sm text-[var(--text-secondary)]">{t('missions.subtitle')}</p>
      </div>

      {workspaces.length > 0 && (
        <section className="loom-card space-y-2.5 rounded-2xl p-3.5">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <span className="flex items-center gap-1.5 text-xs font-medium text-[var(--text-secondary)]">
              <FolderGit2 className="h-3.5 w-3.5 text-[var(--accent-green)]" aria-hidden="true" />
              {t('missions.workspaces')}
            </span>
            {activeChats.length > 0 && (
              <button
                type="button"
                onClick={() => setShowChats((prev) => !prev)}
                className="inline-flex min-h-8 items-center gap-1 rounded-full bg-[var(--accent-blue-soft)] px-2.5 font-mono text-[11px] text-[var(--accent-blue)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              >
                <MessageSquareCode className="h-3 w-3" aria-hidden="true" />
                {activeChats.length} {t('missions.activeChats')}
                {showChats ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
              </button>
            )}
          </div>

          <div className="flex gap-3 overflow-x-auto pb-1.5 [mask-image:linear-gradient(to_right,black_92%,transparent)]">
            {workspaces.map((ws) => (
              <GlassFolderCard
                key={ws.path}
                workspace={ws}
                selected={selectedRepo === ws.path}
                onClick={() => setSelectedRepo(selectedRepo === ws.path ? null : ws.path)}
              />
            ))}
          </div>

          {showChats && activeChats.length > 0 && (
            <div className="space-y-1.5 border-t border-[var(--border-subtle)] pt-2">
              {activeChats.map((chat) => (
                <button
                  key={`${chat.source}-${chat.session}`}
                  type="button"
                  onClick={() => setSelectedRepo(chat.path)}
                  className="loom-elevated flex min-h-11 w-full items-center justify-between gap-2 rounded-xl px-3 py-2 text-left text-xs focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
                >
                  <div className="min-w-0">
                    <p className="truncate font-medium text-[var(--text-primary)]">{chat.title}</p>
                    <p className="truncate font-mono text-[11px] text-[var(--text-muted)]">
                      {chat.workspace} · {chat.model}
                    </p>
                  </div>
                  <span className="shrink-0 rounded-md bg-[var(--bg-surface)] px-2 py-0.5 font-mono text-[10px] text-[var(--accent-blue)]">
                    {chat.source}
                  </span>
                </button>
              ))}
            </div>
          )}
        </section>
      )}

      <Omnibox
        workspaces={workspaces}
        selectedRepo={selectedRepo}
        onSelectRepo={setSelectedRepo}
        defaultSensitivity={defaultSensitivity}
        liveEvent={liveEvent}
        onSubmit={onCreateMission}
      />

      {(generatedImage || imageError) && (
        <section className="loom-card space-y-2 rounded-2xl p-4">
          <div className="flex items-center justify-between gap-2">
            <span className="flex items-center gap-2 text-xs font-medium text-[var(--text-primary)]">
              <ImageIcon className="h-4 w-4 text-[var(--accent-blue)]" aria-hidden="true" />
              {t('missions.imagePreview')}
            </span>
            {onDismissImage && (
              <button
                type="button"
                onClick={onDismissImage}
                className="min-h-11 rounded-lg px-2 text-xs text-[var(--text-muted)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              >
                {t('action.dismiss')}
              </button>
            )}
          </div>
          {imageError && <p className="text-xs text-[var(--accent-amber)]">{imageError}</p>}
          {generatedImage && (
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
              <img
                src={generatedImage.data_uri}
                alt={t('missions.imagePreview')}
                className="h-32 w-32 rounded-xl border border-[var(--border-subtle)] object-cover"
              />
              <p className="font-mono text-xs text-[var(--text-secondary)]">
                {generatedImage.backend_used} · {generatedImage.width}x{generatedImage.height} ·{' '}
                {generatedImage.mime_type}
              </p>
            </div>
          )}
        </section>
      )}

      {missions.length === 0 ? (
        <div className="loom-card rounded-2xl p-6 text-center text-sm text-[var(--text-muted)]">
          {t('missions.empty')}
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-3">
          {missions.map((m, idx) => {
            const isOpen = Boolean(expanded[m.id])
            const diff = diffs[m.id]
            const arts = artifactsByMission[m.id] ?? []
            const decided = m.status === 'approved' || m.status === 'rejected'

            return (
              <motion.article
                key={m.id}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.18, delay: Math.min(idx * 0.04, 0.2) }}
                className="loom-card min-w-0 space-y-3 rounded-2xl p-4"
              >
                <div className="flex min-w-0 flex-col justify-between gap-3 sm:flex-row sm:items-center">
                  <div className="flex min-w-0 flex-1 items-start gap-3">
                    <div className="shrink-0 pt-0.5">
                      <AgentGlyph state={m.state} />
                    </div>
                    <div className="min-w-0 flex-1">
                      <h2 className="break-words text-sm font-medium text-[var(--text-primary)]">
                        {m.title}
                      </h2>
                      <p className="mt-1 truncate font-mono text-xs text-[var(--text-muted)]">
                        {m.worktree} · {m.mode} · {m.effort}
                      </p>
                    </div>
                  </div>

                  <div className="flex flex-wrap items-center gap-2">
                    <span className="loom-elevated rounded-lg px-2.5 py-1 font-mono text-xs text-[var(--accent-blue)]">
                      {m.provider_id}
                    </span>
                    <span
                      className={`loom-elevated rounded-lg px-2.5 py-1 font-mono text-xs ${
                        m.status === 'approved'
                          ? 'text-[var(--accent-green)]'
                          : m.status === 'rejected'
                            ? 'text-[var(--accent-red)]'
                            : 'text-[var(--text-secondary)]'
                      }`}
                    >
                      {(() => {
                        const raw = decided ? m.status : m.state
                        const key = STATUS_KEY_MAP[raw]
                        return key ? t(key) : raw
                      })()}
                    </span>
                    {onFetchDiff && (
                      <button
                        type="button"
                        aria-expanded={isOpen}
                        onClick={() => void toggleDetail(m.id)}
                        className="loom-elevated inline-flex min-h-11 items-center gap-1 rounded-xl px-3 text-xs text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
                      >
                        {isOpen ? t('action.hideDiff') : t('action.viewDiff')}
                        {isOpen ? (
                          <ChevronUp className="h-3.5 w-3.5" />
                        ) : (
                          <ChevronDown className="h-3.5 w-3.5" />
                        )}
                      </button>
                    )}
                  </div>
                </div>

                {isOpen && (
                  <div className="min-w-0 space-y-3 border-t border-[var(--border-subtle)] pt-3">
                    {diff ? (
                      <div className="min-w-0 space-y-2">
                        <p className="break-words font-mono text-xs text-[var(--text-secondary)]">
                          {diff.summary}
                        </p>
                        <pre className="loom-elevated max-h-60 max-w-full overflow-x-auto rounded-xl p-3 font-mono text-[11px] leading-relaxed">
                          {diff.diff.split('\n').map((line, lineIdx) => {
                            const colorClass = line.startsWith('+')
                              ? 'text-[var(--accent-green)]'
                              : line.startsWith('-')
                                ? 'text-[var(--accent-red)]'
                                : line.startsWith('@@')
                                  ? 'text-[var(--accent-blue)]'
                                  : 'text-[var(--text-secondary)]'
                            return (
                              <span key={lineIdx} className={`block ${colorClass}`}>
                                {line}
                              </span>
                            )
                          })}
                        </pre>
                      </div>
                    ) : (
                      <p className="text-xs text-[var(--text-muted)]">{t('missions.loadingDiff')}</p>
                    )}

                    <div className="space-y-2">
                      <p className="text-xs font-medium text-[var(--text-secondary)]">
                        {t('missions.artifactsTitle')}
                      </p>
                      {arts.length === 0 ? (
                        <p className="text-xs text-[var(--text-muted)]">
                          {t('missions.noArtifacts')}
                        </p>
                      ) : (
                        <div className="flex flex-wrap gap-3">
                          {arts.map((art) => (
                            <div
                              key={art.id}
                              className="loom-elevated flex items-center gap-2.5 rounded-xl p-2"
                            >
                              <img
                                src={art.data_uri}
                                alt={art.name}
                                className="h-14 w-14 rounded-lg object-cover"
                              />
                              <div className="font-mono text-[11px]">
                                <p className="text-[var(--text-primary)]">{art.name}</p>
                                <p className="text-[var(--text-muted)]">
                                  {art.width ?? 0}x{art.height ?? 0}
                                </p>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>

                    {onDecideMission && (
                      <div className="flex flex-wrap gap-2">
                        <button
                          type="button"
                          disabled={m.status === 'approved'}
                          onClick={() => void onDecideMission(m.id, 'approve')}
                          className="inline-flex min-h-11 items-center gap-1.5 rounded-xl bg-[var(--accent-green)] px-4 text-xs font-medium text-white disabled:opacity-45 focus-visible:outline-2 focus-visible:outline-[var(--accent-green)]"
                        >
                          <Check className="h-3.5 w-3.5" />
                          {t('action.approve')}
                        </button>
                        <button
                          type="button"
                          disabled={m.status === 'rejected'}
                          onClick={() => void onDecideMission(m.id, 'reject')}
                          className="inline-flex min-h-11 items-center gap-1.5 rounded-xl border border-[var(--accent-red)] bg-[var(--accent-red-soft)] px-4 text-xs font-medium text-[var(--accent-red)] disabled:opacity-45 focus-visible:outline-2 focus-visible:outline-[var(--accent-red)]"
                        >
                          <X className="h-3.5 w-3.5" />
                          {t('action.reject')}
                        </button>
                      </div>
                    )}
                  </div>
                )}
              </motion.article>
            )
          })}
        </div>
      )}
    </div>
  )
}
