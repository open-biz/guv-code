import { Suspense } from 'react'

import type { Metadata } from 'next'

import { Skeleton } from '@/components/ui/skeleton'

import { ProfileClient } from './profile-client'

export const metadata: Metadata = {
  title: 'Profile',
  description: 'Manage your GuvCode profile and API keys.',
}

export default function ProfilePage() {
  return (
    <Suspense
      fallback={
        <div className="container mx-auto px-4 py-20 max-w-3xl">
          <Skeleton className="h-10 w-48 mb-8" />
          <Skeleton className="h-64" />
        </div>
      }
    >
      <ProfileClient />
    </Suspense>
  )
}
