import Link from 'next/link'

import type { Metadata } from 'next'

import { Icons } from '@/components/icons'
import { Button } from '@/components/ui/button'
import { siteConfig } from '@/lib/constant'

export async function generateMetadata(): Promise<Metadata> {
  const title = 'GuvCode – AI Coding Agent for Your Terminal'
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
      description: 'Free tier with usage-based pricing',
    },
    sameAs: [siteConfig.github],
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
    icon: '�',
  },
]

const models = [
  { role: 'Scout', model: 'Gemini 2.5 Pro', job: 'Index codebase, find relevant files', color: 'text-blue-400' },
  { role: 'Architect', model: 'Gemini 3.1 Pro', job: 'Plan multi-file edits', color: 'text-purple-400' },
  { role: 'Coder', model: 'Claude 3 Opus', job: 'Generate AST-aware diffs', color: 'text-orange-400' },
  { role: 'Reviewer', model: 'Local Ollama', job: 'Validate syntax post-edit', color: 'text-green-400' },
]

const terminalLines = [
  { type: 'prompt', text: '$ guv "Add JWT auth middleware to the Express routes"' },
  { type: 'output', text: '' },
  { type: 'success', text: '🎩 Guv\'nor at your service.' },
  { type: 'output', text: '' },
  { type: 'output', text: '⠧ Scout: Indexing repository... 2,847 files mapped in 340ms' },
  { type: 'file', text: '  → src/middleware/auth.ts' },
  { type: 'file', text: '  → src/routes/api.ts' },
  { type: 'file', text: '  → src/lib/jwt.ts' },
  { type: 'output', text: '' },
  { type: 'output', text: '⠧ Architect: Planning edits across 3 files...' },
  { type: 'success', text: '✔ Coder: Patch applied to src/middleware/auth.ts' },
  { type: 'success', text: '✔ Coder: Patch applied to src/routes/api.ts' },
  { type: 'success', text: '✔ Coder: Created src/lib/jwt.ts' },
  { type: 'output', text: '' },
  { type: 'success', text: '✔ Reviewer: All files pass build check.' },
  { type: 'success', text: '✔ Git: Changes committed. Run `guv undo` to revert.' },
  { type: 'output', text: '' },
  { type: 'output', text: '  $0.03 spent · $4.97 remaining' },
]

export default function HomePage() {
  return (
    <>
      <WebSiteJsonLd />
      <main>
        {/* Hero */}
        <section className="relative overflow-hidden pt-20 pb-16 md:pt-32 md:pb-24">
          <div className="container mx-auto px-4 text-center">
            <div className="inline-flex items-center gap-2 rounded-full border border-brand/20 bg-brand/5 px-4 py-1.5 text-sm mb-8">
              <Icons.github className="h-4 w-4" />
              <Link
                href={siteConfig.github}
                target="_blank"
                rel="noopener noreferrer"
                className="text-muted-foreground hover:text-foreground transition-colors"
              >
                Open source — star us on GitHub
              </Link>
            </div>
            <h1 className="hero-heading mb-6">
              Ship code faster.
              <br />
              <span className="bg-gradient-to-r from-[oklch(0.85_0.15_195)] to-[oklch(0.65_0.15_250)] bg-clip-text text-transparent">
                Let Guv handle it.
              </span>
            </h1>
            <p className="hero-subtext mb-10">
              AI coding agent that lives in your terminal. Single Rust binary.
              Multi-model. Indexes your entire codebase in milliseconds.
            </p>
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
              <Link href="/login">
                <Button size="lg" className="text-base px-8 animate-pulse-glow">
                  Get Started Free
                </Button>
              </Link>
              <Link
                href={siteConfig.github}
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button variant="outline" size="lg" className="text-base px-8 gap-2">
                  <Icons.github className="h-4 w-4" />
                  View Source
                </Button>
              </Link>
            </div>
          </div>
        </section>

        {/* Terminal Demo */}
        <section className="pb-20 md:pb-28">
          <div className="container mx-auto px-4 max-w-3xl">
            <div className="terminal">
              <div className="terminal-header">
                <div className="terminal-dot bg-red-500/80" />
                <div className="terminal-dot bg-yellow-500/80" />
                <div className="terminal-dot bg-green-500/80" />
                <span className="ml-3 text-xs text-white/30 font-mono">guv — ~/my-project</span>
              </div>
              <div className="terminal-body">
                {terminalLines.map((line, i) => (
                  <div key={i} className={`terminal-line terminal-${line.type}`}>
                    {line.text}
                  </div>
                ))}
              </div>
            </div>
          </div>
        </section>

        <div className="gradient-divider" />

        {/* Features */}
        <section className="py-20 md:py-28">
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
                  className="group rounded-xl border border-border/50 bg-card/50 p-6 transition-all duration-300 hover:border-border hover:bg-card/80 hover:shadow-lg hover:shadow-brand/5"
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
                GuvCode automatically routes each step to a specialized model.
                No single &quot;god prompt.&quot;
              </p>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 stagger-children">
              {models.map((m) => (
                <div
                  key={m.role}
                  className="flex items-start gap-4 rounded-xl border border-border/50 bg-card/30 p-5 transition-all duration-300 hover:border-border hover:bg-card/60"
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

        {/* Install CTA */}
        <section className="py-20 md:py-28">
          <div className="container mx-auto px-4 text-center max-w-2xl">
            <h2 className="section-heading mb-4">
              Ready to ship?
            </h2>
            <p className="hero-subtext mb-10">
              Install in one command. Configure your keys. Start building.
            </p>

            <div className="terminal max-w-lg mx-auto mb-8">
              <div className="terminal-header">
                <div className="terminal-dot bg-red-500/80" />
                <div className="terminal-dot bg-yellow-500/80" />
                <div className="terminal-dot bg-green-500/80" />
              </div>
              <div className="p-4 font-mono text-sm text-left space-y-1">
                <div className="terminal-prompt">$ curl -sL https://guv.dev/install.sh | bash</div>
                <div className="terminal-prompt">$ guv auth --gemini &quot;KEY&quot; --anthropic &quot;KEY&quot;</div>
                <div className="terminal-prompt">$ guv &quot;Refactor the database layer&quot;</div>
              </div>
            </div>

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
            <p className="mt-6 text-xs text-muted-foreground">
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
    </>
  )
}
