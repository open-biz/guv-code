'use client'

import { Menu, DollarSign, LogIn, BarChart2 } from 'lucide-react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useSession } from 'next-auth/react'

import { UserDropdown } from './user-dropdown'
import { Icons } from '../icons'
import { Button } from '../ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu'
import { Skeleton } from '../ui/skeleton'

import { siteConfig } from '@/lib/constant'
import { cn } from '@/lib/utils'

const HIDDEN_PATHS = ['/subscribe']

export const Navbar = () => {
  const pathname = usePathname()
  const { data: session, status } = useSession()

  if (pathname && HIDDEN_PATHS.includes(pathname)) return null

  return (
    <header className="sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto px-4 py-3 flex justify-between items-center">
        <Link
          href="/"
          className="flex items-center space-x-2 group transition-all duration-300 hover:scale-105"
        >
          <span className="text-2xl">🎩</span>
          <span className="font-bold text-lg tracking-tight">
            {siteConfig.title}
          </span>
        </Link>
        <nav className="hidden md:flex items-center space-x-1 ml-auto">
          <Link
            href="/pricing"
            className="relative font-medium px-3 py-2 rounded-md transition-all duration-200 hover:bg-accent hover:text-accent-foreground group"
          >
            <span className="relative z-10">Pricing</span>
          </Link>
          <Link
            href={siteConfig.github}
            target="_blank"
            rel="noopener noreferrer"
            className="relative font-medium px-3 py-2 rounded-md transition-all duration-200 hover:bg-accent hover:text-accent-foreground flex items-center gap-2 group"
          >
            <Icons.github className="h-4 w-4" />
            <span className="relative z-10">GitHub</span>
          </Link>
          {status !== 'loading' && session && (
            <Link
              href="/usage"
              className="relative font-medium px-3 py-2 rounded-md transition-all duration-200 hover:bg-accent hover:text-accent-foreground group"
            >
              <span className="relative z-10">Usage</span>
            </Link>
          )}
        </nav>
        <div className="flex items-center space-x-2 ml-4">
          <DropdownMenu>
            <DropdownMenuTrigger asChild className="md:hidden">
              <Button variant="ghost" size="icon">
                <Menu className="h-5 w-5" />
                <span className="sr-only">Toggle menu</span>
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem asChild>
                <Link href="/pricing" className="flex items-center cursor-pointer">
                  <DollarSign className="mr-2 h-4 w-4" />
                  <span>Pricing</span>
                </Link>
              </DropdownMenuItem>
              <DropdownMenuItem asChild>
                <Link
                  href={siteConfig.github}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center cursor-pointer"
                >
                  <Icons.github className="mr-2 h-4 w-4" />
                  <span>GitHub</span>
                </Link>
              </DropdownMenuItem>
              {status !== 'loading' && session && (
                <DropdownMenuItem asChild>
                  <Link href="/usage" className="flex items-center cursor-pointer">
                    <BarChart2 className="mr-2 h-4 w-4" />
                    <span>Usage</span>
                  </Link>
                </DropdownMenuItem>
              )}
              {status !== 'loading' && !session && (
                <DropdownMenuItem asChild>
                  <Link href="/login" className="flex items-center cursor-pointer">
                    <LogIn className="mr-2 h-4 w-4" />
                    <span>Log in</span>
                  </Link>
                </DropdownMenuItem>
              )}
            </DropdownMenuContent>
          </DropdownMenu>

          {status === 'loading' ? (
            <div className="hidden md:flex items-center">
              <Skeleton className="h-9 w-20 rounded-md" />
            </div>
          ) : session ? (
            <UserDropdown session={session} />
          ) : (
            <Link href="/login" className="hidden md:inline-block">
              <Button
                className={cn(
                  'bg-white text-black hover:bg-white/90',
                  'border border-white/50',
                  'transition-all duration-300',
                )}
              >
                Log in
              </Button>
            </Link>
          )}
        </div>
      </div>
    </header>
  )
}
