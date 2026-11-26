import type { ReactNode } from 'react'

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
  if (value.includes('codex') || value.includes('openai') || value.includes('gpt') || value.includes('o1') || value.includes('o3')) {
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

const BRAND_SHAPES: Record<BrandKey, ReactNode> = {
  claudehub: (
    <>
      <circle cx="8" cy="12" r="4.5" />
      <circle cx="16" cy="12" r="4.5" />
      <path d="M12 8.5L14.2 12L12 15.5L9.8 12Z" fill="currentColor" stroke="none" />
    </>
  ),
  claude: (
    <>
      <circle cx="12" cy="12" r="2.2" fill="currentColor" stroke="none" />
      <path d="M12 3.2V7.2M12 16.8V20.8M3.2 12H7.2M16.8 12H20.8M5.8 5.8L8.6 8.6M15.4 15.4L18.2 18.2M18.2 5.8L15.4 8.6M8.6 15.4L5.8 18.2" />
    </>
  ),
  codex: (
    <>
      <polygon points="12,3 19.8,7.5 19.8,16.5 12,21 4.2,16.5 4.2,7.5" />
      <polygon points="12,7.2 16.2,9.6 16.2,14.4 12,16.8 7.8,14.4 7.8,9.6" />
      <path d="M12 3V7.2M19.8 16.5L16.2 14.4M4.2 16.5L7.8 14.4" />
    </>
  ),
  gemini: (
    <path
      d="M12 2.5C12.8 7.9 16.1 11.2 21.5 12C16.1 12.8 12.8 16.1 12 21.5C11.2 16.1 7.9 12.8 2.5 12C7.9 11.2 11.2 7.9 12 2.5Z"
      fill="currentColor"
      stroke="none"
    />
  ),
  antigravity: (
    <>
      <path d="M12 3.5L4.5 19.5H8.5L12 11.8L15.5 19.5H19.5L12 3.5Z" />
      <path d="M4 15.2C8.5 12.6 15.5 12.6 20 15.2" />
    </>
  ),
  opencode: (
    <>
      <path d="M8.5 7.5L4 12L8.5 16.5" />
      <path d="M15.5 7.5L20 12L15.5 16.5" />
      <path d="M10.2 16.5H13.8" />
    </>
  ),
  omniroute: (
    <>
      <circle cx="12" cy="5.5" r="2.2" />
      <circle cx="6" cy="17.5" r="2.2" />
      <circle cx="18" cy="17.5" r="2.2" />
      <path d="M12 7.7V12M12 12L7.7 15.8M12 12L16.3 15.8" />
      <circle cx="12" cy="12" r="1.4" fill="currentColor" stroke="none" />
    </>
  ),
  ollama: (
    <>
      <circle cx="12" cy="12" r="8.2" />
      <circle cx="9.2" cy="11.5" r="1.6" fill="currentColor" stroke="none" />
      <circle cx="14.8" cy="11.5" r="1.6" fill="currentColor" stroke="none" />
      <path d="M9.5 15.5C10.5 16.4 13.5 16.4 14.5 15.5" />
    </>
  ),
  git: (
    <>
      <circle cx="7" cy="6" r="2.2" />
      <circle cx="7" cy="18" r="2.2" />
      <circle cx="17" cy="9.5" r="2.2" />
      <path d="M7 8.2V15.8M7 12C7 10.2 9.2 9.5 14.8 9.5" />
    </>
  ),
}

export function BrandIcon({ id, className = 'h-4 w-4' }: BrandIconProps) {
  const brand = normalizeBrandId(id)
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.75}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      {BRAND_SHAPES[brand]}
    </svg>
  )
}
