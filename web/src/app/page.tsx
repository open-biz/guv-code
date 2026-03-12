import Link from 'next/link'

import type { Metadata } from 'next'

import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { siteConfig } from '@/lib/constant'

export async function generateMetadata(): Promise<Metadata> {
  const title = 'GuvCode – Blazingly Fast AI Coding Agent for Your Terminal'
  const description = siteConfig.description

  return {
    title,
    description,
    openGraph: {
      title,
      description,
      url: siteConfig.url(),
      type: 'website',
      siteName: siteConfig.title,
    },
  }
}

function WebSiteJsonLd() {
  const jsonLd = {
    '@context': 'https://schema.org',
    '@type': 'SoftwareApplication',
    name: 'GuvCode',
    applicationCategory: 'DeveloperApplication',
    operatingSystem: 'macOS, Windows, Linux',
    description: siteConfig.description,
    url: siteConfig.url(),
    offers: {
      '@type': 'Offer',
      price: '0',
      priceCurrency: 'USD',
      description: 'Free tier available with usage-based pricing',
    },
  }

  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
    />
  )
}

const features = [
  {
    title: 'Zero Bloat, Zero OOMs',
    description:
      'Single Rust binary. Installs in 1 second. Maps 10,000+ files in milliseconds with ripgrep-style multithreaded directory walking.',
    icon: '🚀',
  },
  {
    title: 'Multi-Agent Routing',
    description:
      'Delegates tasks to the best model — Gemini for context mapping, Claude for code generation, local models for validation.',
    icon: '🧠',
  },
  {
    title: 'Bulletproof AST Diffs',
    description:
      'Native tree-sitter bindings for surgical code injection. No broken indentation, no hallucinated syntax.',
    icon: '🌳',
  },
  {
    title: 'Git-Safe & Budgeted',
    description:
      'Auto-commits before every edit. Built-in token budgeting ensures you never burn cash on runaway loops.',
    icon: '🛡️',
  },
  {
    title: 'BYOK — Bring Your Own Key',
    description:
      'Use any provider: Google, Anthropic, OpenAI, or local Ollama. No vendor lock-in.',
    icon: '🔑',
  },
  {
    title: 'API Quota Management',
    description:
      'Track your usage, manage quotas, and control spending from the dashboard.',
    icon: '📊',
  },
]

export default function HomePage() {
  return (
    <>
      <WebSiteJsonLd />
      <main>
        {/* Hero */}
        <section className="relative overflow-hidden py-24 md:py-32">
          <div className="container mx-auto px-4 text-center">
            <div className="inline-flex items-center gap-2 rounded-full border border-border/60 bg-secondary/50 px-4 py-1.5 text-sm text-muted-foreground mb-8">
              <span className="text-lg">🎩</span>
              <span>&quot;Right away, Guv&apos;nor.&quot;</span>
            </div>
            <h1 className="hero-heading mb-6">
              Your AI Coding Agent,
              <br />
              <span className="bg-gradient-to-r from-white to-white/60 bg-clip-text text-transparent">
                Built in Rust
              </span>
            </h1>
            <p className="hero-subtext mb-10">
              GuvCode is a blazingly fast, multi-model AI coding agent for your
              terminal. Parse massive codebases instantly, plan architectures,
              and surgically apply AST-aware code edits.
            </p>
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
              <Link href="/login">
                <Button size="lg" className="text-base px-8">
                  Get Started
                </Button>
              </Link>
              <Link
                href={siteConfig.github}
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button variant="outline" size="lg" className="text-base px-8">
                  View on GitHub
                </Button>
              </Link>
            </div>

            {/* Install command */}
            <div className="mt-12 max-w-lg mx-auto">
              <div className="rounded-lg border bg-card p-4 font-mono text-sm text-left">
                <span className="text-muted-foreground">$</span>{' '}
                <span className="text-foreground">
                  curl -sL https://guv.dev/install.sh | bash
                </span>
              </div>
            </div>
          </div>
        </section>

        {/* Features */}
        <section className="py-20 border-t">
          <div className="container mx-auto px-4">
            <div className="text-center mb-16">
              <h2 className="text-3xl md:text-4xl font-bold tracking-tight mb-4">
                Why hire Guv?
              </h2>
              <p className="text-muted-foreground text-lg max-w-2xl mx-auto">
                Current agents crash on large codebases, force vendor lock-in,
                and burn your wallet. Guv fixes all of that.
              </p>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {features.map((feature) => (
                <Card
                  key={feature.title}
                  className="bg-card/50 hover:bg-card/80 transition-colors"
                >
                  <CardHeader>
                    <div className="text-3xl mb-2">{feature.icon}</div>
                    <CardTitle className="text-xl">{feature.title}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <CardDescription className="text-base">
                      {feature.description}
                    </CardDescription>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>

        {/* CTA */}
        <section className="py-20 border-t">
          <div className="container mx-auto px-4 text-center">
            <h2 className="text-3xl md:text-4xl font-bold tracking-tight mb-4">
              Ready to start vibecoding?
            </h2>
            <p className="text-muted-foreground text-lg max-w-xl mx-auto mb-8">
              Stop wrestling with dependencies and start shipping. Install
              GuvCode in seconds.
            </p>
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
              <Link href="/login">
                <Button size="lg" className="text-base px-8">
                  Sign up with GitHub
                </Button>
              </Link>
              <Link href="/pricing">
                <Button variant="outline" size="lg" className="text-base px-8">
                  View Pricing
                </Button>
              </Link>
            </div>
          </div>
        </section>
      </main>
    </>
  )
}
