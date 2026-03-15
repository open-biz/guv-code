'use client'

import {
  ChevronRight,
  Files,
  Search,
  GitBranch,
  Bug,
  Package,
  X,
  Split,
  Plus,
  Trash,
  ChevronDown,
} from 'lucide-react'
import React, { useState, useEffect, useRef } from 'react'

import { cn } from '@/lib/utils'

interface FileItem {
  name: string
  type: 'file' | 'folder'
  children?: FileItem[]
  extension?: string
  active?: boolean
}

const fileStructure: FileItem[] = [
  {
    name: 'src',
    type: 'folder',
    children: [
      { name: 'main.rs', type: 'file', extension: 'rs', active: true },
      {
        name: 'agent_logic',
        type: 'folder',
        children: [
          { name: 'planner.rs', type: 'file', extension: 'rs' },
          { name: 'coder.rs', type: 'file', extension: 'rs' },
          { name: 'reviewer.rs', type: 'file', extension: 'rs' },
        ],
      },
      {
        name: 'ui',
        type: 'folder',
        children: [
          { name: 'app.rs', type: 'file', extension: 'rs' },
        ],
      },
      { name: 'llm.rs', type: 'file', extension: 'rs' },
      { name: 'config.rs', type: 'file', extension: 'rs' },
      { name: 'index.rs', type: 'file', extension: 'rs' },
      { name: 'git.rs', type: 'file', extension: 'rs' },
    ],
  },
  {
    name: 'web',
    type: 'folder',
    children: [
      { name: 'src', type: 'folder', children: [
        { name: 'app', type: 'folder', children: [
          { name: 'page.tsx', type: 'file', extension: 'tsx' },
          { name: 'layout.tsx', type: 'file', extension: 'tsx' },
        ]},
      ]},
    ],
  },
  { name: 'Cargo.toml', type: 'file', extension: 'toml' },
  { name: 'README.md', type: 'file', extension: 'md' },
]

const extColor: Record<string, string> = {
  rs: 'text-orange-400',
  tsx: 'text-blue-400',
  toml: 'text-yellow-300',
  md: 'text-zinc-400',
}

function FileIcon({ extension }: { extension?: string }) {
  return (
    <span className={cn('text-xs mr-1.5 leading-none', extension ? (extColor[extension] || 'text-zinc-400') : 'text-zinc-500')}>
      {extension ? '●' : '▸'}
    </span>
  )
}

function FileTreeItem({ item, depth = 0 }: { item: FileItem; depth?: number }) {
  const [isOpen, setIsOpen] = useState(item.type === 'folder')

  return (
    <div>
      <div
        className={cn(
          'flex items-center text-[13px] text-zinc-400 hover:bg-zinc-800/50 rounded px-1.5 py-[3px] cursor-pointer transition-colors duration-100',
          item.active && 'bg-zinc-800 text-zinc-200',
        )}
        style={{ paddingLeft: `${depth * 12 + 6}px` }}
        onClick={() => item.type === 'folder' && setIsOpen(!isOpen)}
      >
        {item.type === 'folder' && (
          <ChevronRight
            size={12}
            className={cn('mr-1 text-zinc-600 transition-transform duration-150 flex-shrink-0', isOpen && 'rotate-90')}
          />
        )}
        <FileIcon extension={item.extension} />
        <span className="truncate">{item.name}</span>
      </div>
      {isOpen && item.children && (
        <div>
          {item.children.map((child, i) => (
            <FileTreeItem key={child.name + i} item={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  )
}

const PHRASES = [
  'add JWT auth to the Express API routes',
  'refactor the database layer for connection pooling',
  'fix the race condition in the websocket handler',
  'optimize the tree-sitter query for large files',
  'add rate limiting middleware to all endpoints',
]

export function IDEDemo({ className }: { className?: string }) {
  const [showIDE, setShowIDE] = useState(false)
  const [expandTerminal, setExpandTerminal] = useState(false)
  const [terminalLines, setTerminalLines] = useState<string[]>([])
  const [typingText, setTypingText] = useState('')
  const [isTyping, setIsTyping] = useState(false)
  const phraseIndexRef = useRef(0)

  useEffect(() => {
    const t = setTimeout(() => setShowIDE(true), 800)
    return () => clearTimeout(t)
  }, [])

  useEffect(() => {
    const t = setTimeout(() => setExpandTerminal(true), 2000)
    return () => clearTimeout(t)
  }, [])

  useEffect(() => {
    if (!showIDE) return

    const messages = [
      'GuvCode will read and write files in "/Users/you/projects/my-app".',
      'Type "help" for commands.',
      'Welcome back! What would you like to do?',
    ]

    let currentIdx = 0
    const lines: string[] = []
    const timeouts: NodeJS.Timeout[] = []

    const showNext = () => {
      if (currentIdx >= messages.length) {
        startTyping()
        return
      }
      const msg = messages[currentIdx]
      const words = msg.split(/\s+/)
      let wordIdx = 0
      let partial = ''

      const streamWords = () => {
        const batch = Math.min(Math.floor(Math.random() * 3) + 2, words.length - wordIdx)
        const segment = words.slice(wordIdx, wordIdx + batch).join(' ')
        partial += (partial ? ' ' : '') + segment
        wordIdx += batch

        lines[currentIdx] = partial
        setTerminalLines([...lines])

        if (wordIdx < words.length) {
          timeouts.push(setTimeout(streamWords, Math.random() * 80 + 30))
        } else {
          currentIdx++
          timeouts.push(setTimeout(showNext, currentIdx === 1 ? 100 : 600))
        }
      }

      streamWords()
    }

    const startTyping = () => {
      setIsTyping(true)

      const typePhrase = () => {
        const phrase = PHRASES[phraseIndexRef.current]
        let charIdx = 0
        setTypingText('')

        const typeChar = () => {
          if (charIdx < phrase.length) {
            charIdx++
            setTypingText(phrase.substring(0, charIdx))
            timeouts.push(setTimeout(typeChar, Math.floor(Math.random() * 80) + 40))
          } else {
            timeouts.push(setTimeout(() => {
              setTypingText('')
              phraseIndexRef.current = (phraseIndexRef.current + 1) % PHRASES.length
              timeouts.push(setTimeout(typePhrase, 400))
            }, 2000))
          }
        }

        timeouts.push(setTimeout(typeChar, 300))
      }

      typePhrase()
    }

    timeouts.push(setTimeout(showNext, 600))
    return () => timeouts.forEach(clearTimeout)
  }, [showIDE])

  return (
    <div className={cn('relative', className)}>
      {/* Decorative offset blocks */}
      <div
        className="absolute rounded-lg"
        style={{
          inset: 0,
          background: 'linear-gradient(165deg, oklch(0.75 0.15 195), oklch(0.55 0.15 250))',
          transform: 'translate(8px, 8px)',
          opacity: 0.6,
          zIndex: -2,
        }}
      />
      <div
        className="absolute rounded-lg"
        style={{
          inset: 0,
          background: 'linear-gradient(165deg, oklch(0.6 0.12 160), oklch(0.45 0.15 195))',
          transform: 'translate(16px, 16px)',
          opacity: 0.35,
          zIndex: -3,
        }}
      />

      <div className="border border-zinc-800 rounded-lg overflow-hidden shadow-2xl bg-black">
        <div className={cn(
          'relative w-full transition-all duration-1000 ease-in-out overflow-hidden',
          showIDE ? 'h-[500px] md:h-[560px]' : 'h-[200px]',
        )}>
          <div className={cn(
            'absolute inset-0 bg-black transition-all duration-1000',
            showIDE ? 'opacity-100' : 'opacity-0 scale-95',
          )}>
            <div className="flex h-full">
              {/* Activity bar */}
              <div className="w-11 border-r border-zinc-800 flex-col items-center py-2 bg-zinc-950/50 hidden md:flex">
                <div className="flex flex-col items-center space-y-3 mt-1">
                  <button className="p-1.5 text-zinc-500 hover:text-zinc-300 transition-colors"><Files size={18} /></button>
                  <button className="p-1.5 text-zinc-500 hover:text-zinc-300 transition-colors"><Search size={18} /></button>
                  <button className="p-1.5 text-zinc-500 hover:text-zinc-300 transition-colors"><GitBranch size={18} /></button>
                  <button className="p-1.5 text-zinc-500 hover:text-zinc-300 transition-colors"><Bug size={18} /></button>
                  <button className="p-1.5 text-zinc-500 hover:text-zinc-300 transition-colors"><Package size={18} /></button>
                </div>
              </div>

              {/* File explorer */}
              <div className={cn(
                'border-r border-zinc-800 bg-zinc-950/30 transition-all duration-700 overflow-hidden hidden md:block',
                showIDE ? 'w-52' : 'w-0',
              )}>
                <div className="p-2">
                  <div className="text-[11px] text-zinc-500 uppercase tracking-wider mb-2 flex items-center justify-between px-1">
                    <span>Explorer</span>
                    <ChevronDown size={12} />
                  </div>
                  <div className="space-y-px">
                    {fileStructure.map((item, i) => (
                      <FileTreeItem key={item.name + i} item={item} />
                    ))}
                  </div>
                </div>
              </div>

              {/* Editor + terminal */}
              <div className="flex-1 flex flex-col min-w-0">
                {/* Tabs */}
                <div className="border-b border-zinc-800 h-8 flex items-center px-1 bg-zinc-950/40">
                  <div className="flex items-center space-x-px">
                    <div className="flex items-center bg-zinc-800/80 rounded-t px-3 py-1 text-xs text-zinc-300 group cursor-pointer">
                      <span className="text-orange-400 text-[10px] mr-1.5">●</span>
                      <span>main.rs</span>
                      <X size={12} className="ml-2 opacity-0 group-hover:opacity-100 text-zinc-500" />
                    </div>
                    <div className="flex items-center hover:bg-zinc-800/30 rounded-t px-3 py-1 text-xs text-zinc-500 group cursor-pointer">
                      <span className="text-orange-400 text-[10px] mr-1.5">●</span>
                      <span>planner.rs</span>
                      <X size={12} className="ml-2 opacity-0 group-hover:opacity-100 text-zinc-500" />
                    </div>
                    <div className="flex items-center hover:bg-zinc-800/30 rounded-t px-3 py-1 text-xs text-zinc-500 group cursor-pointer">
                      <span className="text-blue-400 text-[10px] mr-1.5">●</span>
                      <span>page.tsx</span>
                      <X size={12} className="ml-2 opacity-0 group-hover:opacity-100 text-zinc-500" />
                    </div>
                  </div>
                </div>

                {/* Code editor */}
                <div className={cn(
                  'flex-1 p-3 font-mono text-[13px] leading-relaxed relative transition-all duration-700 overflow-hidden',
                  expandTerminal && 'max-h-[40%]',
                )}>
                  <div className="absolute inset-0 bg-gradient-to-b from-transparent via-transparent to-black/60 pointer-events-none z-10" />
                  {[
                    { n: 1, code: <><span className="text-purple-400">use</span> <span className="text-amber-300">clap</span>::<span className="text-amber-300">Parser</span>;</> },
                    { n: 2, code: <><span className="text-purple-400">use</span> <span className="text-amber-300">tokio</span>;</> },
                    { n: 3, code: null },
                    { n: 4, code: <><span className="text-zinc-500">/// GuvCode — AI coding agent</span></> },
                    { n: 5, code: <><span className="text-purple-400">#[derive(Parser)]</span></> },
                    { n: 6, code: <><span className="text-purple-400">pub struct</span> <span className="text-amber-300">Cli</span> {'{'}</> },
                    { n: 7, code: <><span className="pl-6"><span className="text-purple-400">#[command(subcommand)]</span></span></> },
                    { n: 8, code: <><span className="pl-6">command: <span className="text-amber-300">Option</span>{'<'}<span className="text-amber-300">Commands</span>{'>'}</span>,</> },
                    { n: 9, code: <>{'}'}</> },
                    { n: 10, code: null },
                    { n: 11, code: <><span className="text-purple-400">pub enum</span> <span className="text-amber-300">Commands</span> {'{'}</> },
                    { n: 12, code: <><span className="pl-6"><span className="text-amber-300">Auth</span>,</span></> },
                    { n: 13, code: <><span className="pl-6"><span className="text-amber-300">Budget</span>,</span></> },
                    { n: 14, code: <><span className="pl-6"><span className="text-amber-300">Undo</span>,</span></> },
                    { n: 15, code: <>{'}'}</> },
                  ].map(({ n, code }) => (
                    <div key={n} className="flex">
                      <div className="text-zinc-600 mr-4 select-none w-6 text-right text-xs leading-relaxed">{n}</div>
                      <div className="text-zinc-300">{code || '\u00A0'}</div>
                    </div>
                  ))}
                </div>

                {/* Terminal */}
                <div className={cn(
                  'border-t border-zinc-800 bg-black transition-all duration-700 z-10',
                  expandTerminal ? 'h-[55%]' : 'h-[200px]',
                )}>
                  <div className="flex items-center border-b border-zinc-800 px-3 py-1 bg-zinc-950/40">
                    <span className="text-[11px] text-zinc-500 uppercase tracking-wider">Terminal</span>
                    <div className="ml-auto flex items-center space-x-1.5">
                      <button className="p-0.5 hover:bg-zinc-800 rounded"><Split size={12} className="text-zinc-600" /></button>
                      <button className="p-0.5 hover:bg-zinc-800 rounded"><Plus size={12} className="text-zinc-600" /></button>
                      <button className="p-0.5 hover:bg-zinc-800 rounded"><Trash size={12} className="text-zinc-600" /></button>
                    </div>
                  </div>
                  <div className="p-3 text-[13px] font-mono overflow-auto h-[calc(100%-28px)]">
                    {terminalLines.map((line, i) => (
                      <div key={i} className="text-zinc-400 my-0.5 leading-relaxed">{line}</div>
                    ))}
                    {isTyping && (
                      <div className="text-zinc-300 my-0.5 leading-relaxed">
                        <span className="text-brand">my-app {'>'}</span>{' '}
                        {typingText}
                        <span className="inline-block w-[7px] h-[15px] ml-0.5 bg-brand animate-pulse align-middle relative -top-px" />
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Initial state before IDE loads */}
          {!showIDE && (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="text-center space-y-3">
                <div className="text-2xl">🎩</div>
                <div className="text-sm text-zinc-500 font-mono">Starting GuvCode...</div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default IDEDemo
