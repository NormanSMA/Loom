import { expect, test } from '@playwright/test'

test.describe('Loom UI E2E', () => {
  test('flujo completo en escritorio y movil sin desbordamiento', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text())
    })

    await page.goto('/')
    await expect(page.getByRole('heading', { name: 'Centro de misiones' })).toBeVisible()

    // Check zero horizontal overflow across all tabs + Settings
    for (const tabLabel of ['Conectores', 'Telemetría', 'Memoria', 'Seguridad', 'Misiones']) {
      await page.locator('button:visible', { hasText: tabLabel }).first().click()
      const fits = await page.evaluate(
        () => document.documentElement.scrollWidth <= window.innerWidth + 1,
      )
      expect(fits, `overflow on tab ${tabLabel}`).toBe(true)
    }

    await page.getByRole('button', { name: 'Ajustes y perfil' }).click()
    const settingsFits = await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth + 1,
    )
    expect(settingsFits, 'overflow on tab Ajustes').toBe(true)
    await page.locator('button:visible', { hasText: 'Misiones' }).first().click()

    // Select detected glass folder card and inspect mission diff + approve
    await page.locator('main button:visible', { hasText: 'Loom' }).first().click()
    await page.getByRole('button', { name: 'Ver diff y artefactos' }).first().click()
    await page.getByRole('button', { name: 'Aprobar' }).click()
    await expect(page.getByText('Aprobada').first()).toBeVisible()

    // Generate image preview in Imagen mode
    await page.getByLabel('mode').selectOption('image')
    const input = page.getByPlaceholder('Describe una misión o pide una imagen...')
    await input.fill('Diagrama vector de pruebas')
    await page.getByRole('button', { name: 'Despachar' }).click()
    await expect(page.getByAltText('Vista previa generada')).toBeVisible()

    // Navigate to Ajustes for language, profile and theme toggles
    await page.getByRole('button', { name: 'Ajustes y perfil' }).click()
    const lang = page.getByLabel('language')
    await lang.selectOption('en')
    await expect(page.getByRole('heading', { name: 'Settings & profile' })).toBeVisible()
    await lang.selectOption('es')
    await expect(page.getByRole('heading', { name: 'Ajustes y perfil' })).toBeVisible()

    await page.getByRole('button', { name: 'Claro' }).click()
    await expect
      .poll(() => page.evaluate(() => document.documentElement.getAttribute('data-theme')))
      .toBe('light')

    // Filter skills in Memoria
    await page.locator('button:visible', { hasText: 'Memoria' }).first().click()
    await page.getByRole('button', { name: 'agents' }).click()
    await expect(page.getByText('code-review')).toBeVisible()

    // Security QR + TOTP + Offline + One-Tap auto-pair
    await page.locator('button:visible', { hasText: 'Seguridad' }).first().click()
    await expect(page.getByRole('heading', { name: 'Acceso remoto y control' })).toBeVisible()

    await page.getByRole('button', { name: 'Generar QR efímero' }).click()
    await expect(page.getByText(/^lm_/)).toBeVisible()
    await expect(page.getByAltText('Código QR para emparejar dispositivo')).toBeVisible()
    const pairLink = (await page.locator('a[href*="?pair="]').getAttribute('href')) ?? ''
    expect(pairLink).toContain('?pair=lm_')

    await page.getByRole('button', { name: 'Configurar TOTP' }).click()
    await expect(page.getByText(/^otpauth:\/\/totp\//)).toBeVisible()
    await expect(page.getByAltText('Código QR para configurar TOTP')).toBeVisible()

    await page.getByRole('button', { name: 'Probar reposo' }).click()
    await expect(page.getByText('El nodo Loom en tu equipo está en reposo')).toBeVisible()
    await expect(page.locator('canvas')).toBeVisible()
    await page.getByRole('button', { name: 'Volver al nodo' }).click()
    await expect(page.getByRole('heading', { name: 'Acceso remoto y control' })).toBeVisible()

    await page.goto(pairLink)
    await expect(page.getByText(/Dispositivo vinculado correctamente:/)).toBeVisible()
    await expect.poll(() => page.url()).not.toContain('?pair=')

    await page.locator('button:visible', { hasText: 'Seguridad' }).first().click()
    await expect(page.locator('main').getByText('Activo', { exact: true }).first()).toBeVisible()

    await page.locator('button:visible', { hasText: 'Misiones' }).first().click()
    await expect(page.getByRole('heading', { name: 'Centro de misiones' })).toBeVisible()

    expect(errors).toEqual([])
  })

  test('superficie de isla nativa (?surface=island) sin errores', async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (err) => errors.push(err.message))
    page.on('console', (msg) => {
      if (msg.type() === 'error') errors.push(msg.text())
    })

    await page.goto('/?surface=island')

    const notch = page
      .locator('button.notch-shell')
      .filter({ hasText: /En espera|Pensando|Leyendo|Analizando|Depurando|Creando/ })
    await expect(notch).toBeVisible()
    await expect(notch).toHaveAttribute('aria-expanded', 'false')

    await notch.click()
    await expect(notch).toHaveAttribute('aria-expanded', 'true')
    await expect(notch.locator('p').nth(1)).toBeVisible()

    await notch.click()
    await expect(notch).toHaveAttribute('aria-expanded', 'false')

    expect(errors).toEqual([])
  })
})
