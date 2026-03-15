# GuvCode Web Dashboard

Account management, API keys, and usage tracking for the GuvCode CLI.

## Stack

- **Next.js 16** (App Router, React 19, TypeScript)
- **shadcn/ui** (Radix primitives + Tailwind CSS v4)
- **NextAuth** (GitHub OAuth)
- **Drizzle ORM** (Postgres)
- **Stripe** (billing)

## Setup

```bash
cp .env.example .env.local   # fill in your keys
bun install
bun run dev                   # → http://localhost:3000
```

See [`.env.example`](./.env.example) for required variables.

## Scripts

| Command | Description |
|---|---|
| `bun run dev` | Start dev server |
| `bun run build` | Production build |
| `bun run test` | Unit tests (Jest) |
| `bun run e2e` | E2E tests (Playwright) |
| `bun run lint` | ESLint |
| `bun run typecheck` | TypeScript check |
| `bun run db:generate` | Generate Drizzle migrations |
| `bun run db:migrate` | Run migrations |

## Pages

| Route | Description |
|---|---|
| `/` | Landing page |
| `/login` | GitHub OAuth sign-in |
| `/usage` | Usage dashboard (auth required) |
| `/profile` | Profile & API key management (auth required) |
| `/pricing` | Plans (Free / Pro / Team) |
| `/privacy-policy` | Privacy policy |
| `/terms-of-service` | Terms of service |

## API Routes

- `POST /api/auth/[...nextauth]` — NextAuth handlers
- `GET /api/healthz` — Health check
- `GET /api/user` — Current user info (auth required)
