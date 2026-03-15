'use client'

import Link from 'next/link'
import { signIn } from 'next-auth/react'

import { Icons } from '@/components/icons'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { siteConfig } from '@/lib/constant'

export default function LoginPage() {
  return (
    <div className="flex min-h-[calc(100vh-8rem)] items-center justify-center px-4">
      <div className="w-full max-w-md space-y-6">
        <Card className="border-border/50 bg-card/80 backdrop-blur">
          <CardHeader className="text-center pb-2">
            <div className="mx-auto mb-4 text-5xl">🎩</div>
            <CardTitle className="text-2xl tracking-tight">Sign in to GuvCode</CardTitle>
            <CardDescription className="text-sm">
              Manage API keys, track usage, control budgets.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-4 pt-4">
            <Button
              size="lg"
              className="w-full text-base gap-2"
              onClick={() => signIn('github', { callbackUrl: '/usage' })}
            >
              <Icons.github className="h-5 w-5" />
              Continue with GitHub
            </Button>
            <p className="text-center text-xs text-muted-foreground leading-relaxed">
              By signing in, you agree to our{' '}
              <Link href="/terms-of-service" className="underline hover:text-foreground transition-colors">
                Terms
              </Link>{' '}
              and{' '}
              <Link href="/privacy-policy" className="underline hover:text-foreground transition-colors">
                Privacy Policy
              </Link>
              .
            </p>
          </CardContent>
        </Card>
        <p className="text-center text-xs text-muted-foreground">
          GuvCode is{' '}
          <Link
            href={siteConfig.github}
            target="_blank"
            rel="noopener noreferrer"
            className="underline hover:text-foreground transition-colors"
          >
            open source
          </Link>
          {' '}and free to start.
        </p>
      </div>
    </div>
  )
}
