import { useEffect, useState } from 'react'
import { LazyMotion, domAnimation } from 'motion/react'
import { LiveIsland } from './components/LiveIsland'
import { OfflineCanvas } from './components/OfflineCanvas'
import { Sidebar, type NavTab } from './components/Sidebar'
import { TopBar } from './components/TopBar'
import { useLoomState } from './hooks/useLoomState'
import { I18nProvider, useI18n } from './i18n'
import { ConnectorsView } from './views/ConnectorsView'
import { KnowledgeView } from './views/KnowledgeView'
import { MissionsView } from './views/MissionsView'
import { SecurityView } from './views/SecurityView'
import { TelemetryView } from './views/TelemetryView'

function LoomShell() {
  const { t } = useI18n()
  const [tab, setTab] = useState<NavTab>('missions')
  const [theme, setTheme] = useState<'dark' | 'light'>('dark')
  const loom = useLoomState()

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
  }, [theme])

  if (loom.isOffline) {
    return (
      <div className="loom-dot-bg min-h-screen">
        <OfflineCanvas
          lastSync={loom.lastSync}
          cachedMissions={loom.missions}
          simulated={loom.simulatedOffline}
          onRetry={() => void loom.refresh()}
          onExitSimulated={() => loom.setSimulatedOffline(false)}
        />
      </div>
    )
  }

  return (
    <div className="loom-dot-bg flex min-h-screen pb-[calc(5.5rem+env(safe-area-inset-bottom))] md:pb-0">
      <Sidebar
        activeTab={tab}
        onSelect={setTab}
        onNewMission={() => setTab('missions')}
      />

      <div className="flex min-w-0 flex-1 flex-col">
        <TopBar
          demo={loom.health?.demo ?? false}
          theme={theme}
          onToggleTheme={() => setTheme((prev) => (prev === 'dark' ? 'light' : 'dark'))}
          onEmergencyStop={loom.emergencyStop}
        />

        <LiveIsland event={loom.liveEvent} />

        {loom.pairingNotice && (
          <div
            role="status"
            className="mx-auto mt-3 flex w-full max-w-5xl items-center justify-between gap-3 px-4 sm:px-6"
          >
            <div
              className={`flex w-full items-center justify-between gap-3 rounded-xl border px-4 py-2.5 text-xs ${
                loom.pairingNotice === 'ok'
                  ? 'border-[var(--accent-green)] bg-[var(--accent-green-soft)] text-[var(--accent-green)]'
                  : 'border-[var(--accent-red)] bg-[var(--accent-red-soft)] text-[var(--accent-red)]'
              }`}
            >
              <span>
                {loom.pairingNotice === 'ok'
                  ? `${t('security.pairedBanner')} ${loom.pairedDevice?.label ?? ''}`
                  : t('security.pairFailedBanner')}
              </span>
              <button
                type="button"
                onClick={loom.dismissPairingNotice}
                className="min-h-11 rounded-lg px-3 font-medium focus-visible:outline-2 focus-visible:outline-[var(--accent-blue)]"
              >
                {t('action.dismiss')}
              </button>
            </div>
          </div>
        )}

        <main className="mx-auto w-full max-w-5xl flex-1 px-4 py-5 sm:px-6">
          {tab === 'missions' && (
            <MissionsView
              missions={loom.missions}
              workspaces={loom.workspaces}
              generatedImage={loom.generatedImage}
              imageError={loom.imageError}
              onDismissImage={loom.dismissGeneratedImage}
              onCreateMission={loom.createMission}
              onFetchDiff={loom.fetchMissionDiff}
              onFetchArtifacts={loom.fetchMissionArtifacts}
              onDecideMission={loom.decideMission}
            />
          )}
          {tab === 'connectors' && (
            <ConnectorsView
              doctor={loom.doctor}
              providers={loom.providers}
              claudehub={loom.claudehub}
            />
          )}
          {tab === 'telemetry' && <TelemetryView telemetry={loom.telemetry} />}
          {tab === 'knowledge' && <KnowledgeView skills={loom.skills} />}
          {tab === 'security' && (
            <SecurityView
              devices={loom.devices}
              network={loom.network}
              totpStatus={loom.totpStatus}
              onCreatePairing={loom.createPairing}
              onRevokeDevice={loom.revokeDevice}
              onSetupTotp={loom.setupTotp}
              onVerifyTotp={loom.verifyTotp}
              onSimulateOffline={() => loom.setSimulatedOffline(true)}
            />
          )}
        </main>
      </div>
    </div>
  )
}

export function App() {
  return (
    <I18nProvider>
      <LazyMotion features={domAnimation}>
        <LoomShell />
      </LazyMotion>
    </I18nProvider>
  )
}

export default App
