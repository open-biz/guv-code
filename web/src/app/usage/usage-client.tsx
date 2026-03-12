'use client'

import { useSession } from 'next-auth/react'
import { useRouter } from 'next/navigation'
import { useEffect } from 'react'

import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'

export function UsageClient() {
  const { data: session, status } = useSession()
  const router = useRouter()

  useEffect(() => {
    if (status === 'unauthenticated') {
      router.push('/login')
    }
  }, [status, router])

  if (status === 'loading') {
    return (
      <div className="container mx-auto px-4 py-20 max-w-4xl">
        <Skeleton className="h-10 w-48 mb-8" />
        <div className="grid gap-6">
          <Skeleton className="h-40" />
          <Skeleton className="h-40" />
          <Skeleton className="h-40" />
        </div>
      </div>
    )
  }

  if (!session) {
    return null
  }

  return (
    <div className="container mx-auto px-4 py-20 max-w-4xl">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Usage Dashboard</h1>
          <p className="text-muted-foreground mt-1">
            Monitor your API usage and manage quotas.
          </p>
        </div>
        <Badge variant="secondary">Free Plan</Badge>
      </div>

      <div className="grid gap-6">
        {/* Current Period */}
        <Card>
          <CardHeader>
            <CardTitle>Current Period</CardTitle>
            <CardDescription>
              Your usage for the current billing cycle.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium">API Requests</span>
                <span className="text-sm text-muted-foreground">
                  0 / 50 daily
                </span>
              </div>
              <Progress value={0} className="h-2" />
            </div>
            <div>
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium">Tokens Used</span>
                <span className="text-sm text-muted-foreground">0</span>
              </div>
              <Progress value={0} className="h-2" />
            </div>
          </CardContent>
        </Card>

        {/* Budget */}
        <Card>
          <CardHeader>
            <CardTitle>Budget</CardTitle>
            <CardDescription>
              Set spending limits to control costs.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div>
                <p className="text-2xl font-bold">$0.00</p>
                <p className="text-sm text-muted-foreground">spent this month</p>
              </div>
              <div className="text-right">
                <p className="text-2xl font-bold text-muted-foreground">$0.00</p>
                <p className="text-sm text-muted-foreground">budget limit</p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Model Usage Breakdown */}
        <Card>
          <CardHeader>
            <CardTitle>Model Usage</CardTitle>
            <CardDescription>
              Breakdown by model provider.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {[
                { name: 'Gemini (Scout)', requests: 0, color: 'bg-emerald-500' },
                { name: 'Claude (Coder)', requests: 0, color: 'bg-blue-500' },
                { name: 'GPT-4o (Fallback)', requests: 0, color: 'bg-purple-500' },
                { name: 'Local (Reviewer)', requests: 0, color: 'bg-orange-500' },
              ].map((model) => (
                <div key={model.name}>
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-2">
                      <div className={`w-2 h-2 rounded-full ${model.color}`} />
                      <span className="text-sm">{model.name}</span>
                    </div>
                    <span className="text-sm text-muted-foreground">
                      {model.requests} requests
                    </span>
                  </div>
                  <Separator />
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
