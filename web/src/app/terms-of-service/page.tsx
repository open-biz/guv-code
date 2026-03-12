import type { Metadata } from 'next'

import { siteConfig } from '@/lib/constant'

export const metadata: Metadata = {
  title: 'Terms of Service',
}

export default function TermsOfServicePage() {
  return (
    <div className="container mx-auto px-4 py-20 max-w-3xl prose prose-invert">
      <h1>Terms of Service</h1>
      <p className="text-muted-foreground">
        Last updated: {new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}
      </p>

      <h2>1. Acceptance of Terms</h2>
      <p>
        By accessing and using {siteConfig.title}, you agree to be bound by
        these Terms of Service.
      </p>

      <h2>2. Use of Service</h2>
      <p>
        {siteConfig.title} provides AI-powered coding assistance via a CLI
        tool. You are responsible for your use of the service and any content
        you generate.
      </p>

      <h2>3. API Usage</h2>
      <p>
        Usage of the {siteConfig.title} API is subject to rate limits and
        quotas as defined by your plan. Abuse of the API may result in
        suspension.
      </p>

      <h2>4. BYOK Policy</h2>
      <p>
        {siteConfig.title} supports Bring Your Own Key (BYOK). You are
        responsible for securing your own API keys and any charges incurred
        through your providers.
      </p>

      <h2>5. Contact</h2>
      <p>
        For questions about these terms, contact us at{' '}
        <a href={`mailto:${siteConfig.supportEmail()}`}>
          {siteConfig.supportEmail()}
        </a>
        .
      </p>
    </div>
  )
}
