import type { MetadataRoute } from 'next'

import { siteConfig } from '@/lib/constant'

export default function robots(): MetadataRoute.Robots {
  return {
    rules: {
      userAgent: '*',
      allow: '/',
      disallow: ['/api/', '/profile/', '/usage/'],
    },
    sitemap: `${siteConfig.url()}/sitemap.xml`,
  }
}
