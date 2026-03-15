'use client'

import { useSession } from 'next-auth/react'
import Image from 'next/image'
import { useRouter, useSearchParams } from 'next/navigation'
import { useEffect } from 'react'

import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

export function ProfileClient() {
  const { data: session, status } = useSession()
  const router = useRouter()
  const searchParams = useSearchParams()
  const defaultTab = searchParams.get('tab') || 'profile'

  useEffect(() => {
    if (status === 'unauthenticated') {
      router.push('/login')
    }
  }, [status, router])

  if (status === 'loading') {
    return (
      <div className="container mx-auto px-4 py-20 max-w-3xl">
        <Skeleton className="h-10 w-48 mb-8" />
        <Skeleton className="h-64" />
      </div>
    )
  }

  if (!session) {
    return null
  }

  return (
    <div className="container mx-auto px-4 py-20 md:py-28 max-w-3xl page-enter">
      <h1 className="section-heading mb-8">Settings</h1>

      <Tabs defaultValue={defaultTab}>
        <TabsList>
          <TabsTrigger value="profile">Profile</TabsTrigger>
          <TabsTrigger value="api-keys">API Keys</TabsTrigger>
        </TabsList>

        <TabsContent value="profile" className="mt-6">
          <Card className="border-border/50 bg-card/50 card-hover-glow">
            <CardHeader>
              <CardTitle>Profile</CardTitle>
              <CardDescription>
                Your account information from GitHub.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="flex items-center gap-4">
                {session.user?.image && (
                  <Image
                    src={session.user.image}
                    alt={session.user.name ?? 'User'}
                    width={64}
                    height={64}
                    className="rounded-full"
                  />
                )}
                <div>
                  <p className="font-medium text-lg">{session.user?.name}</p>
                  <p className="text-sm text-muted-foreground">
                    {session.user?.email}
                  </p>
                </div>
              </div>
              <Separator />
              <div className="space-y-4">
                <div>
                  <Label>Name</Label>
                  <Input
                    value={session.user?.name ?? ''}
                    disabled
                    className="mt-1.5"
                  />
                </div>
                <div>
                  <Label>Email</Label>
                  <Input
                    value={session.user?.email ?? ''}
                    disabled
                    className="mt-1.5"
                  />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="api-keys" className="mt-6">
          <Card className="border-border/50 bg-card/50 card-hover-glow">
            <CardHeader>
              <CardTitle>API Keys</CardTitle>
              <CardDescription>
                Manage your GuvCode API keys for CLI authentication.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground">
                No API keys yet. Generate one to authenticate the GuvCode CLI.
              </p>
              <Button>Generate API Key</Button>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
