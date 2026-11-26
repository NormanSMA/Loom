import { defineConfig } from '@playwright/test'

const externalUrl = process.env.LOOM_E2E_URL

export default defineConfig({
  testDir: './e2e',
  fullyParallel: false,
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  reporter: 'list',
  use: {
    baseURL: externalUrl ?? 'http://127.0.0.1:4321',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'desktop',
      use: { viewport: { width: 1280, height: 800 } },
    },
    {
      name: 'mobile',
      use: { viewport: { width: 375, height: 812 } },
    },
  ],
  ...(externalUrl
    ? {}
    : {
        webServer: {
          command: 'cargo run -p loomd -- --demo --static-dir apps/web/dist',
          cwd: '../..',
          url: 'http://127.0.0.1:4321/api/health',
          reuseExistingServer: !process.env.CI,
          timeout: 120_000,
        },
      }),
})
