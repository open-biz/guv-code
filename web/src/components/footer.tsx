'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

import { Icons } from '@/components/icons'
import { siteConfig } from '@/lib/constant'

type LinkInfo = { text: string; href: string; target?: string }

const siteLinks: LinkInfo[] = [
  { text: 'Home', href: '/' },
  { text: 'Pricing', href: '/pricing' },
  { text: 'Usage', href: '/usage' },
]

const legalLinks: LinkInfo[] = [
  { text: 'Privacy Policy', href: '/privacy-policy' },
  { text: 'Terms of Service', href: '/terms-of-service' },
]

const ossLinks: LinkInfo[] = [
  { text: 'GitHub', href: siteConfig.github, target: '_blank' },
  { text: 'Issues', href: `${siteConfig.github}/issues`, target: '_blank' },
  { text: 'Contributing', href: `${siteConfig.github}/blob/main/CONTRIBUTING.md`, target: '_blank' },
]

const publicPaths = [
  ...legalLinks,
  ...ossLinks,
  ...siteLinks,
  { text: 'Login', href: '/login' },
]
  .map((link) => link.href)
  .filter((href) => !href.startsWith('http'))

const FOOTER_HIDDEN_PATHS = ['/subscribe']

export const Footer = () => {
  const pathname = usePathname() ?? '/'
  const isPublicPage = publicPaths.includes(pathname)

  if (!isPublicPage || FOOTER_HIDDEN_PATHS.includes(pathname)) {
    return null
  }

  return (
    <footer className="w-full border-t border-border/40 z-10">
      <div className="container mx-auto flex flex-col gap-6 py-8 px-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-10">
          <div className="space-y-3">
            <Link href="/" className="flex items-center space-x-2">
              <span className="text-2xl">🎩</span>
              <span className="font-bold text-lg">{siteConfig.title}</span>
            </Link>
            <p className="text-sm text-muted-foreground leading-relaxed">
              Open-source AI coding agent. Built in Rust.
            </p>
            <Link
              href={siteConfig.github}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
            >
              <Icons.github className="h-4 w-4" />
              <span>Star us on GitHub</span>
            </Link>
          </div>

          <div>
            <h3 className="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Product</h3>
            <nav className="flex flex-col space-y-2">
              {siteLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  target={link.target}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  {link.text}
                </Link>
              ))}
            </nav>
          </div>

          <div>
            <h3 className="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Open Source</h3>
            <nav className="flex flex-col space-y-2">
              {ossLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  target={link.target}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  {link.text}
                </Link>
              ))}
            </nav>
          </div>

          <div>
            <h3 className="font-semibold mb-4 text-sm uppercase tracking-wider text-muted-foreground">Legal</h3>
            <nav className="flex flex-col space-y-2">
              {legalLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  {link.text}
                </Link>
              ))}
            </nav>
          </div>
        </div>

        <div className="gradient-divider" />

        <div className="flex flex-col sm:flex-row items-center justify-between gap-2 text-xs text-muted-foreground">
          <span>&copy; {new Date().getFullYear()} {siteConfig.title}. MIT License.</span>
          <Link
            href={siteConfig.github}
            target="_blank"
            rel="noopener noreferrer"
            className="hover:text-foreground transition-colors"
          >
            100% open source
          </Link>
        </div>
      </div>
    </footer>
  )
}
