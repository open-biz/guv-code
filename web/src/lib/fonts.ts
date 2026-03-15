import { Geist, Geist_Mono, Playfair_Display } from 'next/font/google'

const fontSans = Geist({
  subsets: ['latin'],
  variable: '--font-geist-sans',
})

const fontMono = Geist_Mono({
  subsets: ['latin'],
  variable: '--font-geist-mono',
})

const fontDisplay = Playfair_Display({
  subsets: ['latin'],
  variable: '--font-display',
  weight: ['400', '700', '900'],
})

export const fonts = [fontSans.variable, fontMono.variable, fontDisplay.variable]
