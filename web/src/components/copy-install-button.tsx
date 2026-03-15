'use client'

import { Check, Copy, Terminal } from 'lucide-react'
import { useState } from 'react'

import { cn } from '@/lib/utils'

const INSTALL_COMMAND = 'curl -sL https://guv.dev/install.sh | bash'

export function CopyInstallButton({ className }: { className?: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(INSTALL_COMMAND)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }

  return (
    <button
      onClick={handleCopy}
      className={cn(
        'group relative w-full md:w-auto md:min-w-[380px]',
        'bg-zinc-800/60 border border-zinc-700/50 rounded-md overflow-hidden',
        'flex items-center justify-between h-14 px-4',
        'transition-all duration-300',
        'hover:border-brand/40 hover:shadow-[0_0_15px_oklch(0.75_0.15_195_/_0.15)]',
        className,
      )}
      aria-label="Copy install command"
    >
      <div className="flex items-center space-x-3">
        <Terminal size={18} className="text-brand flex-shrink-0" />
        <code className="font-mono text-sm text-white/90 select-all">
          {INSTALL_COMMAND}
        </code>
      </div>
      <div className="flex items-center ml-3 p-1.5 rounded text-white/50 hover:text-white hover:bg-white/5 transition-colors">
        {copied ? (
          <div className="flex items-center gap-1 text-green-400">
            <Check size={16} />
            <span className="text-xs font-medium">Copied!</span>
          </div>
        ) : (
          <Copy size={16} />
        )}
      </div>
    </button>
  )
}
