import type { Metadata } from 'next'

import { UsageClient } from './usage-client'

export const metadata: Metadata = {
  title: 'Usage',
  description: 'Monitor your GuvCode API usage and manage quotas.',
}

export default function UsagePage() {
  return <UsageClient />
}
