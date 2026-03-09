export const siteConfig = {
  title: 'GuvCode',
  description:
    'GuvCode is a blazingly fast, 100% Rust-native AI coding agent built for your terminal. Multi-model vibecoding with zero bloat.',
  keywords: () => [
    'GuvCode',
    'AI Coding Agent',
    'Terminal AI',
    'Rust CLI',
    'Code Assistant',
    'Multi-Model AI',
    'Vibecoding',
  ],
  url: () => process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000',
  supportEmail: () =>
    process.env.NEXT_PUBLIC_SUPPORT_EMAIL || 'support@guv.dev',
  github: 'https://github.com/open-biz/guv-code',
}
