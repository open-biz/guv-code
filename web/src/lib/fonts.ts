import { Geist, Geist_Mono } from 'next/font/google'

const fontSans = Geist({
  subsets: ['latin'],
  variable: '--font-geist-sans',
})

const fontMono = Geist_Mono({
  subsets: ['latin'],
  variable: '--font-geist-mono',
})

export const fonts = [fontSans.variable, fontMono.variable]
