import Link from 'next/link'

import type { Metadata } from 'next'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { siteConfig } from '@/lib/constant'
import { cn } from '@/lib/utils'

export const metadata: Metadata = {
  title: 'Pricing',
  description: `${siteConfig.title} pricing. Start free, upgrade when you need to.`,
}

const plans = [
  {
    name: 'Free',
    price: '$0',
    period: 'forever',
    description: 'Try it out. No credit card.',
    features: [
      '50 requests/day',
      'Community support',
      'Basic model access',
      'Single user',
    ],
    cta: 'Get Started',
    href: '/login',
    popular: false,
  },
  {
    name: 'Pro',
    price: '$20',
    period: '/mo',
    description: 'For developers who ship daily.',
    features: [
      'Unlimited requests',
      'Priority support',
      'All models (Gemini, Claude, GPT-4o)',
      'Usage analytics',
      'API key management',
      'Budget controls',
    ],
    cta: 'Start Pro Trial',
    href: '/login',
    popular: true,
  },
  {
    name: 'Team',
    price: '$50',
    period: '/user/mo',
    description: 'For teams that build together.',
    features: [
      'Everything in Pro',
      'Team management',
      'Shared budgets & quotas',
      'SSO / SAML',
      'Audit logs',
      'Dedicated support',
    ],
    cta: 'Contact Sales',
    href: 'mailto:sales@guv.dev',
    popular: false,
  },
]

export default function PricingPage() {
  return (
    <div className="container mx-auto px-4 py-20 md:py-28">
      <div className="text-center mb-16">
        <h1 className="section-heading mb-4">
          Simple pricing
        </h1>
        <p className="hero-subtext">
          Start free. Upgrade when you need to. No surprises.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto stagger-children">
        {plans.map((plan) => (
          <Card
            key={plan.name}
            className={cn(
              'relative flex flex-col border-border/50 bg-card/50 transition-all duration-300 hover:bg-card/80',
              plan.popular && 'glow-border',
            )}
          >
            {plan.popular && (
              <div className="absolute -top-3 left-1/2 -translate-x-1/2">
                <Badge className="bg-brand text-black font-semibold">Most Popular</Badge>
              </div>
            )}
            <CardHeader>
              <CardTitle className="text-xl">{plan.name}</CardTitle>
              <div className="mt-2">
                <span className="text-4xl font-bold tracking-tight">{plan.price}</span>
                <span className="text-muted-foreground ml-1 text-sm">
                  {plan.period}
                </span>
              </div>
              <CardDescription className="mt-2 text-sm">
                {plan.description}
              </CardDescription>
            </CardHeader>
            <CardContent className="flex-1">
              <ul className="space-y-3">
                {plan.features.map((feature) => (
                  <li key={feature} className="flex items-center gap-2.5 text-sm">
                    <span className="text-green-400 text-xs">✓</span>
                    <span className="text-muted-foreground">{feature}</span>
                  </li>
                ))}
              </ul>
            </CardContent>
            <CardFooter>
              <Link href={plan.href} className="w-full">
                <Button
                  className="w-full"
                  variant={plan.popular ? 'default' : 'outline'}
                  size="lg"
                >
                  {plan.cta}
                </Button>
              </Link>
            </CardFooter>
          </Card>
        ))}
      </div>

      <div className="text-center mt-12">
        <p className="text-sm text-muted-foreground">
          GuvCode is{' '}
          <Link
            href={siteConfig.github}
            target="_blank"
            rel="noopener noreferrer"
            className="underline hover:text-foreground transition-colors"
          >
            open source (MIT)
          </Link>
          . Self-host for free, or use our managed dashboard.
        </p>
      </div>
    </div>
  )
}
