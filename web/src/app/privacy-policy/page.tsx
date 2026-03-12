import type { Metadata } from 'next'

import { siteConfig } from '@/lib/constant'

export const metadata: Metadata = {
  title: 'Privacy Policy',
}

export default function PrivacyPolicyPage() {
  return (
    <div className="container mx-auto px-4 py-20 max-w-3xl prose prose-invert">
      <h1>Privacy Policy</h1>
      <p className="text-muted-foreground">
        Last updated: {new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}
      </p>

      <h2>1. Information We Collect</h2>
      <p>
        When you use {siteConfig.title}, we collect information you provide
        directly, such as your GitHub account information when you sign in.
      </p>

      <h2>2. How We Use Your Information</h2>
      <p>
        We use the information we collect to provide, maintain, and improve our
        services, including managing your API quotas and billing.
      </p>

      <h2>3. Data Storage</h2>
      <p>
        Your data is stored securely and we do not sell your personal
        information to third parties.
      </p>

      <h2>4. Contact</h2>
      <p>
        For privacy-related questions, contact us at{' '}
        <a href={`mailto:${siteConfig.supportEmail()}`}>
          {siteConfig.supportEmail()}
        </a>
        .
      </p>
    </div>
  )
}
