<h1 align="center">GuvCode Web Dashboard</h1>

<p align="center">
  <em>Account management, API keys, and usage tracking for the GuvCode CLI.</em>
</p>

<p align="center">
  <a href="https://nextjs.org/"><img src="https://img.shields.io/badge/Next.js-16-000000.svg?style=flat-square&logo=nextdotjs" alt="Next.js"></a>
  <a href="https://ui.shadcn.com/"><img src="https://img.shields.io/badge/UI-shadcn-000000.svg?style=flat-square&logo=shadcnui" alt="shadcn/ui"></a>
  <a href="https://www.typescriptlang.org/"><img src="https://img.shields.io/badge/TypeScript-007ACC?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript"></a>
</p>

---

The GuvCode Web Dashboard is the central hub for managing your interaction with the GuvCode ecosystem. While the CLI is your primary coding interface, this dashboard provides visibility into your usage, billing, and access controls.

## 📑 Table of Contents

- [Features](#-features)
- [Getting Started](#-getting-started)
- [Available Scripts](#-available-scripts)
- [Configuration Reference](#-configuration-reference)
- [Project Structure](#-project-structure)

---

## ✨ Features

- 🔐 **GitHub OAuth:** Secure authentication powered by NextAuth.
- 📊 **Usage Monitoring:** Track your API consumption and budget quotas in real-time.
- 🔑 **API Key Management:** Generate, rotate, and revoke keys used by your local GuvCode CLI.
- 💳 **Billing & Pricing:** Manage your Free, Pro, or Team tier subscriptions (powered by Stripe).
- 🎨 **Modern Stack:** Built on Next.js App Router, styled with Tailwind CSS v4 and `shadcn/ui`.

---

## 🚀 Getting Started

Follow these steps to spin up the web dashboard locally.

### Prerequisites

- [Bun](https://bun.sh/) (Package Manager)
- PostgreSQL (or an equivalent connection string for Drizzle ORM)

### Installation

**1. Clone the repository and navigate to the web directory:**
```bash
git clone https://github.com/your-username/guv-code.git
cd guv-code/web
```

**2. Set up environment variables:**
Copy the example environment file and fill in your specific credentials.
```bash
cp .env.example .env.local
```

**3. Install dependencies:**
```bash
bun install
```

**4. Start the development server:**
```bash
bun run dev
```

The application will now be running at [http://localhost:3000](http://localhost:3000).

---

## 🛠️ Available Scripts

We use `bun` to run our project scripts. Here are the primary commands you'll need:

| Command | Description |
|---|---|
| `bun run dev` | Start the development server with Hot Module Replacement. |
| `bun run build` | Create an optimized production build. |
| `bun run test` | Run the unit test suite using Jest. |
| `bun run e2e` | Run end-to-end tests using Playwright. |
| `bun run lint` | Run ESLint to find and fix problems. |
| `bun run typecheck` | Run the TypeScript compiler to check for type errors. |
| `bun run db:generate` | Generate new Drizzle migrations based on your schema. |
| `bun run db:migrate` | Apply pending migrations to the database. |

---

## ⚙️ Configuration Reference

### Environment Variables

Your `.env.local` file should contain the following configurations (refer to `.env.example` for the complete list):

| Variable Name | Description |
|---|---|
| `DATABASE_URL` | Your PostgreSQL connection string. |
| `NEXTAUTH_URL` | The base URL of your application (e.g., `http://localhost:3000`). |
| `NEXTAUTH_SECRET` | A random string used to encrypt session data. |
| `GITHUB_ID` | Your GitHub OAuth App Client ID. |
| `GITHUB_SECRET` | Your GitHub OAuth App Client Secret. |
| `STRIPE_SECRET_KEY` | Your Stripe API secret key for billing. |

### API Routes & Pages

<details>
  <summary>Click to view routing structure</summary>
  
  **Application Routes:**
  - `/` - Landing page
  - `/login` - GitHub OAuth sign-in
  - `/usage` - Usage dashboard (auth required)
  - `/profile` - Profile & API key management (auth required)
  - `/pricing` - Plans (Free / Pro / Team)
  - `/privacy-policy` - Privacy policy
  - `/terms-of-service` - Terms of service

  **API Endpoints:**
  - `POST /api/auth/[...nextauth]` - NextAuth authentication handlers
  - `GET /api/healthz` - Basic health check endpoint
  - `GET /api/user` - Fetch current user information (auth required)
</details>

---

## 🤝 Contributing

We welcome improvements to the dashboard! Please see the root [`CONTRIBUTING.md`](../CONTRIBUTING.md) for general guidelines before submitting a pull request.

---

## 📄 License

This project is licensed under the MIT License - see the root [LICENSE](../LICENSE) file for details.
