'use client'

import { User, Key, BarChart2 } from 'lucide-react'
import Image from 'next/image'
import { useRouter } from 'next/navigation'
import { signOut } from 'next-auth/react'

import type { Session } from 'next-auth'

import { Icons } from '@/components/icons'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'

export const UserDropdown = ({ session: { user } }: { session: Session }) => {
  const router = useRouter()

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <div className="relative group">
          <div className="relative bg-white border border-white/50 rounded-md overflow-hidden transition-all duration-300 group-hover:brightness-110">
            {user?.image ? (
              <Image
                className="w-8 h-8"
                src={user.image}
                alt={user.name ?? 'User'}
                width={32}
                height={32}
              />
            ) : (
              <div className="w-8 h-8 bg-muted flex items-center justify-center">
                <User className="w-4 h-4" />
              </div>
            )}
          </div>
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuLabel>{user?.name}</DropdownMenuLabel>
        <DropdownMenuItem onClick={() => router.push('/usage')}>
          <BarChart2 className="mr-2 size-4" /> <span>Usage</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => router.push('/profile')}>
          <User className="mr-2 size-4" /> <span>Profile</span>
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => router.push('/profile?tab=api-keys')}>
          <Key className="mr-2 size-4" /> <span>API Keys</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => signOut()}>
          <Icons.logOut className="mr-2 size-4" /> <span>Log out</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
