export interface UserProfile {
  id: string
  githubId: string
  name: string | null
  email: string | null
  image: string | null
  plan: string
  createdAt: Date
  updatedAt: Date
}

export interface ApiKey {
  id: string
  name: string
  keyPrefix: string
  isActive: boolean
  lastUsedAt: Date | null
  createdAt: Date
}

export interface UsageRecord {
  id: string
  model: string
  provider: string
  tokensInput: number
  tokensOutput: number
  costCents: number
  createdAt: Date
}
