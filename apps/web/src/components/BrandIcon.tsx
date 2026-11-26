export type BrandKey =
  | 'claudehub'
  | 'claude'
  | 'codex'
  | 'gemini'
  | 'antigravity'
  | 'opencode'
  | 'omniroute'
  | 'ollama'
  | 'git'

export interface BrandIconProps {
  id: string
  className?: string
}

const BRAND_ASSETS: Record<BrandKey, string> = {
  claudehub: '/brands/claudehub.png',
  claude: '/brands/claude.svg',
  codex: '/brands/codex.svg',
  gemini: '/brands/gemini.svg',
  antigravity: '/brands/antigravity.svg',
  opencode: '/brands/opencode.svg',
  omniroute: '/brands/omniroute.svg',
  ollama: '/brands/ollama.svg',
  git: '/brands/git.svg',
}

export function normalizeBrandId(id: string): BrandKey {
  const value = id.trim().toLowerCase()
  if (value.includes('claudehub') || value.includes('claude-hub') || value.includes('hub')) {
    return 'claudehub'
  }
  if (
    value.includes('claude') ||
    value.includes('anthropic') ||
    value.includes('sonnet') ||
    value.includes('opus') ||
    value.includes('haiku')
  ) {
    return 'claude'
  }
  if (
    value.includes('codex') ||
    value.includes('openai') ||
    value.includes('gpt') ||
    value.includes('o1') ||
    value.includes('o3')
  ) {
    return 'codex'
  }
  if (value.includes('gemini') || value.includes('google')) {
    return 'gemini'
  }
  if (value.includes('antigravity') || value === 'agy') {
    return 'antigravity'
  }
  if (value.includes('opencode')) {
    return 'opencode'
  }
  if (value.includes('omniroute') || value.includes('omni')) {
    return 'omniroute'
  }
  if (value.includes('ollama') || value.includes('llama')) {
    return 'ollama'
  }
  return 'git'
}

export function BrandIcon({ id, className = 'h-4 w-4' }: BrandIconProps) {
  const brand = normalizeBrandId(id)
  return (
    <img
      src={BRAND_ASSETS[brand]}
      alt=""
      aria-hidden="true"
      draggable={false}
      className={`${className} shrink-0 rounded-xs object-contain`}
    />
  )
}
