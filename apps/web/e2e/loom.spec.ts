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

    const noHorizontalOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth + 1,
    )
    expect(noHorizontalOverflow).toBe(true)

    const lang = page.getByLabel('language')
    await lang.selectOption('en')
    await expect(page.getByRole('heading', { name: 'Mission control' })).toBeVisible()
    await lang.selectOption('es')
    await expect(page.getByRole('heading', { name: 'Centro de misiones' })).toBeVisible()

    await page.getByRole('button', { name: 'Claro' }).click()
    await expect
      .poll(() => page.evaluate(() => document.documentElement.getAttribute('data-theme')))
      .toBe('light')

    const input = page.getByPlaceholder('Describe una mision o pide un recurso visual...')
    await input.fill('Prueba E2E multiagente')
    await page.getByRole('button', { name: 'Despachar' }).click()
    await expect(page.getByText('Prueba E2E multiagente')).toBeVisible()

    await page.locator('button:visible', { hasText: 'Seguridad' }).first().click()
    await expect(page.getByRole('heading', { name: 'Acceso remoto y control' })).toBeVisible()

    await page.getByRole('button', { name: 'Generar QR efimero' }).click()
    await expect(page.getByText(/^lm_/)).toBeVisible()

    await page.getByRole('button', { name: 'Configurar TOTP' }).click()
    await expect(page.getByText(/^otpauth:\/\/totp\//)).toBeVisible()

    await page.getByRole('button', { name: 'Probar reposo' }).click()
    await expect(page.getByText('El nodo Loom en tu equipo esta en reposo')).toBeVisible()
    await expect(page.locator('canvas')).toBeVisible()
    await page.getByRole('button', { name: 'Volver al nodo' }).click()
    await expect(page.getByRole('heading', { name: 'Acceso remoto y control' })).toBeVisible()
    await page.locator('button:visible', { hasText: 'Misiones' }).first().click()
    await expect(page.getByRole('heading', { name: 'Centro de misiones' })).toBeVisible()

    expect(errors).toEqual([])
  })
})
