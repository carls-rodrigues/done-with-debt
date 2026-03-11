<p align="center">
  <img src="assets/logo.svg" alt="Done With Debt" width="480" />
</p>

<h1 align="center">Done With Debt</h1>

<p align="center">
  Personal finance management — track spending, manage budgets, and crush your debt.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.80+-orange?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Next.js-16-black?logo=next.js" alt="Next.js" />
  <img src="https://img.shields.io/badge/Flutter-3.x-blue?logo=flutter" alt="Flutter" />
  <img src="https://img.shields.io/badge/PostgreSQL-16-blue?logo=postgresql" alt="PostgreSQL" />
</p>

---

## Overview

**Done With Debt** is a cross-platform personal finance app that gives you a complete picture of your financial life — where your money goes, how your debts are progressing, and what you can do to reach financial freedom faster.

- **Web** — Next.js 16 (App Router) with a rich dashboard experience
- **Mobile** — Flutter app for iOS and Android
- **API** — Rust + Axum backend with hexagonal architecture

## Stack

| Layer | Technology |
|---|---|
| API | Rust, Axum, SQLx, PostgreSQL 16 |
| Web | Next.js 16, TanStack Query, Zustand, shadcn/ui, Recharts |
| Mobile | Flutter, Riverpod, go_router, fl_chart |
| Auth | JWT via httpOnly cookies (web) / secure storage (mobile) |
| Payments | Stripe (web) + RevenueCat (mobile) |
| Storage | Cloudflare R2 (transaction attachments) |
| Infra | Docker Compose (local), PostgreSQL |

## Project Structure

```
done_with_debt/
├── api/          # Rust + Axum backend (hexagonal architecture)
├── web/          # Next.js 16 frontend
├── mobile/       # Flutter mobile app
├── assets/       # Project assets (logo, etc.)
└── docker-compose.yml
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.80+
- [Node.js](https://nodejs.org/) 20+
- [Flutter](https://flutter.dev/) 3.x
- [Docker](https://www.docker.com/) (for PostgreSQL)

### 1. Start the database

```bash
docker compose up -d
```

### 2. Run the API

```bash
cd api
cp .env.example .env   # fill in your values
cargo run
```

The API will be available at `http://localhost:8080`.

### 3. Run the web app

```bash
cd web
npm install
npm run dev
```

The web app will be available at `http://localhost:3000`.

### 4. Run the mobile app

```bash
cd mobile
flutter pub get
flutter run
```

## Environment Variables

See `api/.env` for the full list of required environment variables. Key ones:

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Secret for signing JWT tokens |
| `STRIPE_SECRET_KEY` | Stripe API key (payments) |
| `REVENUECAT_WEBHOOK_SECRET` | RevenueCat webhook secret (mobile payments) |
| `R2_*` | Cloudflare R2 credentials (file storage) |

## Development

This project follows **Extreme Programming (XP)** practices:

- Test-Driven Development (Red → Green → Refactor)
- Small, frequent releases
- Continuous integration

### Running tests

```bash
# API
cd api && cargo test

# Web
cd web && npm test

# Mobile
cd mobile && flutter test
```

## License

Private — all rights reserved.
