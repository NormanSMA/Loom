import { useState, type FormEvent } from 'react'
import { QrCode, ShieldCheck, Smartphone, WifiOff } from 'lucide-react'
import { useI18n } from '../i18n'
import type { DeviceView, PairingTicket, TotpSetup, TotpStatus } from '../types/api'

interface SecurityViewProps {
  devices: DeviceView[]
  totpStatus: TotpStatus
  onCreatePairing: () => Promise<PairingTicket>
  onRevokeDevice: (id: string) => Promise<void>
  onSetupTotp: () => Promise<TotpSetup>
  onVerifyTotp: (code: string) => Promise<boolean>
  onSimulateOffline: () => void
}

export function SecurityView({
  devices,
  totpStatus,
  onCreatePairing,
  onRevokeDevice,
  onSetupTotp,
  onVerifyTotp,
  onSimulateOffline,
}: SecurityViewProps) {
  const { t } = useI18n()
  const [ticket, setTicket] = useState<PairingTicket | null>(null)
  const [totpSetup, setTotpSetup] = useState<TotpSetup | null>(null)
  const [totpCode, setTotpCode] = useState('')
  const [totpError, setTotpError] = useState(false)

  const handlePair = async () => {
    const next = await onCreatePairing()
    setTicket(next)
  }

  const handleSetupTotp = async () => {
    setTotpError(false)
    const next = await onSetupTotp()
    setTotpSetup(next)
  }

  const handleVerifyTotp = async (e: FormEvent) => {
    e.preventDefault()
    if (!totpCode.trim()) return
    setTotpError(false)
    try {
      const ok = await onVerifyTotp(totpCode.trim())
      if (!ok) setTotpError(true)
      else setTotpCode('')
    } catch {
      setTotpError(true)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h1 className="text-xl font-semibold text-[var(--text-primary)]">
            {t('security.title')}
          </h1>
          <p className="text-sm text-[var(--text-secondary)]">{t('security.subtitle')}</p>
        </div>
        <span className="inline-flex min-h-8 items-center rounded-full border border-[var(--border-subtle)] bg-[var(--bg-surface)] px-3 font-mono text-xs text-[var(--text-secondary)]">
          {totpStatus.remote_enabled ? t('security.remoteEnabled') : t('security.remoteLocalOnly')}
        </span>
      </div>

      <div className="flex flex-wrap gap-3">
        <button
          type="button"
          onClick={() => void handlePair()}
          className="inline-flex min-h-11 items-center gap-2 rounded-xl bg-[var(--accent-blue)] px-4 text-sm font-medium text-white focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
        >
          <QrCode className="h-4 w-4" />
          {t('action.pairQr')}
        </button>

        <button
          type="button"
          onClick={() => void handleSetupTotp()}
          className="loom-elevated inline-flex min-h-11 items-center gap-2 rounded-xl px-4 text-sm text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
        >
          <ShieldCheck className="h-4 w-4" />
          {t('action.setupTotp')}
        </button>

        <button
          type="button"
          onClick={onSimulateOffline}
          className="loom-elevated inline-flex min-h-11 items-center gap-2 rounded-xl px-4 text-sm text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
        >
          <WifiOff className="h-4 w-4" />
          {t('action.simulateOffline')}
        </button>
      </div>

      {ticket && (
        <section className="loom-card rounded-2xl p-4">
          <p className="font-mono text-xs text-[var(--accent-green)]">{ticket.code}</p>
          <p className="mt-1 break-all font-mono text-xs text-[var(--text-secondary)]">
            {ticket.qr_uri}
          </p>
        </section>
      )}

      <section className="loom-card rounded-2xl p-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <h2 className="text-sm font-medium text-[var(--text-primary)]">
            {t('security.totpTitle')}
          </h2>
          <span
            className={`font-mono text-xs ${
              totpStatus.enrolled ? 'text-[var(--accent-green)]' : 'text-[var(--accent-amber)]'
            }`}
          >
            {totpStatus.enrolled
              ? t('security.totpStatusEnrolled')
              : t('security.totpStatusPending')}
          </span>
        </div>

        {totpSetup && (
          <div className="mt-3 space-y-3">
            <p className="font-mono text-xs text-[var(--accent-green)]">{totpSetup.secret_base32}</p>
            <p className="break-all font-mono text-xs text-[var(--text-secondary)]">
              {totpSetup.otpauth_uri}
            </p>
            <form onSubmit={(e) => void handleVerifyTotp(e)} className="flex flex-wrap gap-2">
              <input
                type="text"
                inputMode="numeric"
                maxLength={8}
                value={totpCode}
                onChange={(e) => setTotpCode(e.target.value)}
                placeholder={t('security.totpCodePlaceholder')}
                aria-label={t('security.totpCodePlaceholder')}
                className="loom-elevated min-h-11 rounded-xl px-3 font-mono text-xs text-[var(--text-primary)] focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              />
              <button
                type="submit"
                className="inline-flex min-h-11 items-center rounded-xl bg-[var(--accent-blue)] px-4 text-xs font-medium text-white focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              >
                {t('action.verifyTotp')}
              </button>
            </form>
            {totpError && (
              <p role="alert" className="text-xs text-[var(--accent-red)]">
                {t('security.totpInvalid')}
              </p>
            )}
          </div>
        )}
      </section>

      <section className="loom-card rounded-2xl p-4">
        <h2 className="mb-3 text-sm font-medium text-[var(--text-primary)]">
          {t('security.devices')}
        </h2>
        {devices.length === 0 ? (
          <p className="text-xs text-[var(--text-muted)]">{t('security.noDevices')}</p>
        ) : (
          <div className="space-y-2">
            {devices.map((dev) => (
              <div
                key={dev.id}
                className="loom-elevated flex items-center justify-between gap-3 rounded-xl px-3 py-2"
              >
                <div className="flex items-center gap-2.5">
                  <Smartphone className="h-4 w-4 text-[var(--text-secondary)]" />
                  <div>
                    <p className="text-xs font-medium text-[var(--text-primary)]">{dev.label}</p>
                    <p className="font-mono text-[11px] text-[var(--text-muted)]">
                      {dev.revoked ? t('security.revoked') : t('security.active')}
                    </p>
                  </div>
                </div>
                {!dev.revoked && (
                  <button
                    type="button"
                    onClick={() => void onRevokeDevice(dev.id)}
                    className="min-h-11 rounded-lg px-3 text-xs font-medium text-[var(--accent-red)] focus-visible:outline-2 focus-visible:outline-[var(--accent-red)]"
                  >
                    {t('action.revoke')}
                  </button>
                )}
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
