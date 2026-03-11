'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

import { Separator } from '@/components/ui/separator'
import { siteConfig } from '@/lib/constant'

type LinkInfo = { text: string; href: string; target?: string }

const siteLinks: LinkInfo[] = [
  { text: 'Home', href: '/' },
  { text: 'Pricing', href: '/pricing' },
]

const legalLinks: LinkInfo[] = [
  { text: 'Privacy Policy', href: '/privacy-policy' },
  { text: 'Terms of Service', href: '/terms-of-service' },
]

const communityLinks: LinkInfo[] = [
  {
    text: 'GitHub',
    href: siteConfig.github,
    target: '_blank',
  },
]

const publicPaths = [
  ...legalLinks,
  ...communityLinks,
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
    <footer className="w-full border-t z-10">
      <div className="container mx-auto flex flex-col gap-4 py-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-12 py-4">
          <div className="flex items-center space-x-2">
            <Link href="/" className="flex items-center space-x-2">
              <span className="text-2xl">🎩</span>
              <span className="font-bold text-lg">{siteConfig.title}</span>
            </Link>
          </div>

          <div>
            <h3 className="font-semibold mb-4">Site</h3>
            <nav className="flex flex-col space-y-2">
              {siteLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  target={link.target}
                  className="text-muted-foreground hover:text-primary"
                >
                  {link.text}
                </Link>
              ))}
            </nav>
          </div>

          <div>
            <h3 className="font-semibold mb-4">Legal</h3>
            <nav className="flex flex-col space-y-2">
              {legalLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  className="text-muted-foreground hover:text-primary"
                >
                  {link.text}
                </Link>
              ))}
            </nav>
          </div>

          <div>
            <h3 className="font-semibold mb-4">Community</h3>
            <nav className="flex flex-col space-y-2">
              {communityLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  target={link.target}
                  className="text-muted-foreground hover:text-primary"
                >
                  {link.text}
                </Link>
              ))}
            </nav>
          </div>
        </div>

        <Separator />

        <div className="text-center text-xs text-muted-foreground">
          &copy; {new Date().getFullYear()} {siteConfig.title}. All rights
          reserved.
        </div>
      </div>
    </footer>
  )
}
