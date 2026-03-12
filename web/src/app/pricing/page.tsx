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

export const metadata: Metadata = {
  title: 'Pricing',
  description: `${siteConfig.title} pricing plans. Choose the plan that works for you.`,
}

const plans = [
  {
    name: 'Free',
    price: '$0',
    period: 'forever',
    description: 'Perfect for trying out GuvCode.',
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
    period: '/month',
    description: 'For individual developers who need more power.',
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
    period: '/user/month',
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
    <div className="container mx-auto px-4 py-20">
      <div className="text-center mb-16">
        <h1 className="text-4xl md:text-5xl font-bold tracking-tight mb-4">
          Simple, transparent pricing
        </h1>
        <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
          Start free, upgrade when you need to. No hidden fees, no surprises.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-5xl mx-auto">
        {plans.map((plan) => (
          <Card
            key={plan.name}
            className={`relative flex flex-col ${
              plan.popular ? 'border-primary shadow-lg' : ''
            }`}
          >
            {plan.popular && (
              <div className="absolute -top-3 left-1/2 -translate-x-1/2">
                <Badge>Most Popular</Badge>
              </div>
            )}
            <CardHeader>
              <CardTitle className="text-xl">{plan.name}</CardTitle>
              <div className="mt-2">
                <span className="text-4xl font-bold">{plan.price}</span>
                <span className="text-muted-foreground ml-1">
                  {plan.period}
                </span>
              </div>
              <CardDescription className="mt-2">
                {plan.description}
              </CardDescription>
            </CardHeader>
            <CardContent className="flex-1">
              <ul className="space-y-3">
                {plan.features.map((feature) => (
                  <li key={feature} className="flex items-center gap-2 text-sm">
                    <span className="text-primary">✓</span>
                    {feature}
                  </li>
                ))}
              </ul>
            </CardContent>
            <CardFooter>
              <Link href={plan.href} className="w-full">
                <Button
                  className="w-full"
                  variant={plan.popular ? 'default' : 'outline'}
                >
                  {plan.cta}
                </Button>
              </Link>
            </CardFooter>
          </Card>
        ))}
      </div>
    </div>
  )
}
