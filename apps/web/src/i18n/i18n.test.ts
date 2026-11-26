import { describe, expect, it } from 'vitest'
import es from './locales/es.json'
import en from './locales/en.json'
import ptBr from './locales/pt-BR.json'

describe('i18n locale parity', () => {
  it('keeps identical keys and non-empty values across es, en, and pt-BR', () => {
    const baseKeys = Object.keys(es).sort()
    const enKeys = Object.keys(en).sort()
    const ptKeys = Object.keys(ptBr).sort()

    expect(enKeys).toEqual(baseKeys)
    expect(ptKeys).toEqual(baseKeys)

    for (const key of baseKeys) {
      expect(es[key as keyof typeof es].trim().length).toBeGreaterThan(0)
      expect(en[key as keyof typeof en].trim().length).toBeGreaterThan(0)
      expect(ptBr[key as keyof typeof ptBr].trim().length).toBeGreaterThan(0)
    }
  })
})
