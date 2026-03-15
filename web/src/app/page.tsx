import type { Metadata } from 'next'

import HomeClient from './home-client'

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

export default function HomePage() {
  return (
    <>
      <WebSiteJsonLd />
      <HomeClient />
    </>
  )
}
