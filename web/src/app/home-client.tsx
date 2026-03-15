'use client'

import Link from 'next/link'
import { useState } from 'react'

import { CopyInstallButton } from '@/components/copy-install-button'
import { Icons } from '@/components/icons'
import { IDEDemo } from '@/components/ide-demo'
import { Button } from '@/components/ui/button'
import { siteConfig } from '@/lib/constant'
import { cn } from '@/lib/utils'

const features = [
  {
    title: 'Single binary, zero deps',
    description: 'One Rust binary. Installs in 1s. Indexes 10k+ files in milliseconds.',
    icon: '⚡',
  },
  {
    title: 'Multi-model routing',
    description: 'Gemini maps your codebase. Claude writes the code. Ollama validates locally.',
    icon: '🧠',
  },
  {
    title: 'AST-aware edits',
    description: 'tree-sitter bindings for surgical code injection. No broken indentation.',
    icon: '🌳',
  },
  {
    title: 'Git-safe',
    description: 'Auto-commits before every edit. One command to revert.',
    icon: '🛡️',
  },
  {
    title: 'BYOK',
    description: 'Bring your own keys. Google, Anthropic, OpenAI, or local Ollama.',
    icon: '🔑',
  },
  {
    title: 'Budget controls',
    description: 'Set spend caps. Track usage. Never burn cash on a runaway loop.',
    icon: '📊',
  },
]

const models = [
  { role: 'Scout', model: 'Gemini 2.5 Pro', job: 'Index codebase, find relevant files', color: 'text-blue-400' },
  { role: 'Architect', model: 'Gemini 2.5 Pro', job: 'Plan multi-file edits', color: 'text-purple-400' },
  { role: 'Coder', model: 'Claude Sonnet 4', job: 'Generate AST-aware diffs', color: 'text-orange-400' },
  { role: 'Reviewer', model: 'Local Ollama', job: 'Validate syntax post-edit', color: 'text-green-400' },
]

export default function HomeClient() {
  const [buttonHovered, setButtonHovered] = useState(false)

  return (
    <main>
      {/* Hero */}
      <section className="relative overflow-hidden pt-20 pb-8 md:pt-28 md:pb-12 hero-glow">
        <div className="container mx-auto px-4 text-center">
          <div className="inline-flex items-center gap-2 rounded-full border border-brand/20 bg-brand/5 px-4 py-1.5 text-sm mb-8">
            <Icons.github className="h-3.5 w-3.5" />
            <Link
              href={siteConfig.github}
              target="_blank"
              rel="noopener noreferrer"
              className="text-muted-foreground hover:text-foreground transition-colors"
            >
              Open source — star us on GitHub
            </Link>
          </div>
          <h1 className="hero-heading mb-6 text-white">
            Ship code faster.{' '}
            <br className="hidden sm:block" />
            Let Guv handle it.
          </h1>
          <p className="hero-subtext mb-10">
            AI coding agent that lives in your terminal. Single Rust binary.
            Multi-model. Indexes your entire codebase in milliseconds.
          </p>
          <div className="flex flex-col md:flex-row items-center justify-center gap-5 max-w-2xl mx-auto">
            {/* Offset shadow Get Started button */}
            <div
              className="relative w-full md:w-auto"
              onMouseEnter={() => setButtonHovered(true)}
              onMouseLeave={() => setButtonHovered(false)}
            >
              <div className="absolute inset-0 bg-brand rounded-md -translate-x-1 translate-y-1" />
              <div
                className="relative transition-transform duration-200"
                style={{
                  transform: buttonHovered ? 'translate(2px, -2px)' : 'translate(0, 0)',
                }}
              >
                <Link href="/login" className="block">
                  <Button
                    size="lg"
                    className={cn(
                      'relative w-full',
                      'px-8 py-4 h-auto text-base font-medium',
                      'bg-white text-black hover:bg-white',
                      'transition-all duration-200',
                    )}
                  >
                    Get Started
                  </Button>
                </Link>
              </div>
            </div>

            {/* Install copy button */}
            <CopyInstallButton className="h-[52px]" />
          </div>
        </div>
      </section>

      {/* IDE Demo */}
      <section className="pb-16 md:pb-24 pt-8 md:pt-12">
        <div className="container mx-auto px-4 max-w-5xl">
          <IDEDemo />
        </div>
      </section>

      <div className="gradient-divider" />

      {/* Features */}
      <section className="py-20 md:py-28 dot-grid">
        <div className="container mx-auto px-4">
          <div className="text-center mb-16">
            <h2 className="section-heading mb-4">
              Why GuvCode
            </h2>
            <p className="hero-subtext">
              Other agents crash on large repos, force vendor lock-in, and burn your wallet.
            </p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 max-w-5xl mx-auto stagger-children">
            {features.map((feature) => (
              <div
                key={feature.title}
                className="group rounded-xl border border-border/50 bg-card/50 p-6 card-hover-glow"
              >
                <div className="text-2xl mb-3">{feature.icon}</div>
                <h3 className="font-semibold text-lg mb-2">{feature.title}</h3>
                <p className="text-sm text-muted-foreground leading-relaxed">
                  {feature.description}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <div className="gradient-divider" />

      {/* Model Routing */}
      <section className="py-20 md:py-28">
        <div className="container mx-auto px-4 max-w-4xl">
          <div className="text-center mb-16">
            <h2 className="section-heading mb-4">
              The right model for every task
            </h2>
            <p className="hero-subtext">
              GuvCode routes each step to a specialized model.
              No single &quot;god prompt.&quot;
            </p>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 stagger-children">
            {models.map((m) => (
              <div
                key={m.role}
                className="flex items-start gap-4 rounded-xl border border-border/50 bg-card/30 p-5 card-hover-glow"
              >
                <div className="flex-shrink-0">
                  <div className={`text-xs font-mono font-bold uppercase tracking-wider ${m.color}`}>
                    {m.role}
                  </div>
                  <div className="text-sm font-medium mt-1">{m.model}</div>
                </div>
                <div className="text-sm text-muted-foreground leading-relaxed">
                  {m.job}
                </div>
              </div>
            ))}
          </div>
          <div className="text-center mt-8">
            <Link
              href={`${siteConfig.github}/blob/main/MODELS.md`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              See full model routing docs →
            </Link>
          </div>
        </div>
      </section>

      <div className="gradient-divider" />

      {/* CTA */}
      <section className="py-20 md:py-28 hero-glow">
        <div className="container mx-auto px-4 text-center max-w-2xl">
          <h2 className="hero-heading mb-4 text-white">
            Ready to ship?
          </h2>
          <p className="hero-subtext mb-10">
            Install in one command. Configure your keys. Start building.
          </p>

          <div className="flex flex-col items-center gap-6 max-w-lg mx-auto">
            <CopyInstallButton />

            <div className="flex flex-col sm:flex-row items-center justify-center gap-4 w-full">
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

          <p className="mt-8 text-xs text-muted-foreground">
            Free tier included · No credit card required ·{' '}
            <Link
              href={siteConfig.github}
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-foreground transition-colors underline"
            >
              MIT licensed
            </Link>
          </p>
        </div>
      </section>
    </main>
  )
}
