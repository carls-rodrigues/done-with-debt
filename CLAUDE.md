# CLAUDE.md — Done With Debt

> Este documento é a fonte de verdade do projeto. Leia inteiro antes de qualquer
> implementação. Ele evolui a cada sessão — atualize sempre que descobrir um hurdle
> novo, estabelecer um padrão ou tomar uma decisão arquitetural importante.

---

## 1. Visão Geral do Projeto

**Done With Debt** é um app de gestão financeira pessoal que dá visibilidade total sobre as finanças do usuário — onde está o dinheiro, para onde vai, e como gastar melhor. Disponível em web e mobile.

### Problema que resolve
Pessoas não sabem para onde vai o seu dinheiro. Falta uma ferramenta que unifique carteiras, transações, orçamentos e análises em um só lugar — com insights reais que ajudam a tomar decisões financeiras mais conscientes, sem depender de integração bancária automática.

### Fluxo principal
```
Cadastra conta → Adiciona carteiras → Lança transações →
Categoriza gastos → Acompanha orçamento →
Obtém insights e análises → Toma decisões financeiras conscientes
```

### Módulos
- **Carteiras**: onde está o dinheiro (contas, cartões, dinheiro em espécie, investimentos)
- **Transações**: para onde vai o dinheiro (entradas, saídas, transferências)
- **Orçamentos**: limites mensais por categoria
- **Relatórios & Insights**: padrões, tendências, alertas, análise completa
- **Dívidas**: módulo complementar para planejamento de quitação (avalanche/snowball)

### Estado atual
- [x] MVP em desenvolvimento
- [ ] Em produção
- [ ] Versão atual: 0.1.0

---

## 2. Stack Tecnológico

### Backend (`api/`)
- **Linguagem**: Rust
- **Framework HTTP**: Axum
- **Runtime async**: tokio
- **Middlewares HTTP**: tower-http (CORS, logging, compression)
- **Banco principal**: PostgreSQL
- **Acesso a dados**: SQLx (queries compiladas em tempo de compilação, sem ORM)
- **Migrations**: SQLx CLI (`sqlx migrate run`)
- **Auth**: JWT — jsonwebtoken crate (email/password no MVP)
- **Hashing de senhas**: argon2
- **Serialização**: serde + serde_json
- **Validação**: validator
- **Datas**: chrono
- **IDs**: uuid
- **Config**: dotenv
- **Error handling**: thiserror (erros tipados) + anyhow (erros contextuais)
- **Observabilidade**: tracing + tracing-subscriber
- **Documentação**: utoipa + utoipa-swagger-ui + utoipa-axum
- **Testes de integração**: testcontainers (PostgreSQL real)

### Web (`web/`)
- **Framework**: Next.js 16 (App Router)
- **Linguagem**: TypeScript
- **Data fetching**: TanStack Query v5
- **HTTP client**: axios
- **State management**: Zustand
- **Formulários**: react-hook-form + zod + @hookform/resolvers
- **Datas**: date-fns
- **UI**: Tailwind CSS + shadcn/ui
- **Charts**: Recharts
- **Auth**: JWT em cookie httpOnly (setado pelo backend Rust, lido pelo Next.js middleware para proteção de rotas)

### Mobile (`mobile/`)
- **Framework**: Flutter (iOS + Android)
- **Linguagem**: Dart
- **State management**: Riverpod
- **Navegação**: go_router
- **UI**: Flutter Material 3
- **Charts**: fl_chart
- **HTTP**: dio
- **Armazenamento seguro**: flutter_secure_storage
- **Push notifications**: firebase_messaging
- **Biometria**: local_auth

### Infra / Deploy
- **API**: Docker + Railway (ou Fly.io)
- **Web**: Vercel
- **Mobile**: App Store + Google Play (via `flutter build` + Fastlane)
- **CI/CD**: GitHub Actions
- **Monitoramento**: Sentry (web + mobile)
- **Analytics**: PostHog
- **Pagamentos web**: Stripe (assinaturas + webhooks)
- **Pagamentos mobile**: RevenueCat (App Store + Google Play)
- **Storage**: Cloudflare R2 (anexos de transações)

### Segurança
- **Pre-commit hooks**: lefthook (gerencia hooks para Rust + Next.js + Flutter)
- **Rust**: `cargo audit` (CVEs) + `cargo deny` (licenças/duplicatas) + `cargo clippy` (linting)
- **Web**: `npm audit` (CVEs)
- **CI**: mesmos checks + bloqueio de merge em vulnerabilidades críticas

---

## 3. Variáveis de Ambiente

### API (`api/.env`)
```env
# App
RUST_LOG=debug
HOST=0.0.0.0
PORT=8080

# Banco
DATABASE_URL=postgres://postgres:postgres@localhost:5432/done_with_debt

# Auth
JWT_SECRET=
JWT_EXPIRY_HOURS=168

# Cookie
COOKIE_SECURE=false
COOKIE_SAME_SITE=lax

# CORS
ALLOWED_ORIGINS=http://localhost:3000

# Monitoramento
SENTRY_DSN=

# Pagamentos
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
REVENUECAT_WEBHOOK_SECRET=

# Storage (Cloudflare R2)
R2_ACCOUNT_ID=
R2_ACCESS_KEY_ID=
R2_SECRET_ACCESS_KEY=
R2_BUCKET_NAME=
R2_PUBLIC_URL=
```

### Web (`web/.env.local`)
```env
NEXT_PUBLIC_API_URL=http://localhost:8080

# Monitoramento
NEXT_PUBLIC_SENTRY_DSN=
NEXT_PUBLIC_POSTHOG_KEY=
```

### Mobile (`mobile/.env`)
```env
API_URL=http://localhost:8080
```

> ⚠️ Sempre que adicionar uma nova variável, documente aqui ANTES de usar no código.

---

## 4. Estrutura de Diretórios

```
done_with_debt/
├── api/                                      # Rust backend (Axum + SQLx)
│   ├── src/
│   │   ├── main.rs                           # Entry point, bootstrap
│   │   ├── config.rs                         # Config struct from env vars
│   │   ├── db.rs                             # PgPool setup
│   │   ├── errors.rs                         # AppError + IntoResponse
│   │   ├── domain/
│   │   │   ├── entities/
│   │   │   │   ├── user.rs
│   │   │   │   ├── social_account.rs
│   │   │   │   ├── auth_token.rs
│   │   │   │   ├── user_settings.rs
│   │   │   │   ├── notification_settings.rs
│   │   │   │   ├── institution.rs
│   │   │   │   ├── wallet.rs
│   │   │   │   ├── credit_card_details.rs
│   │   │   │   ├── category.rs
│   │   │   │   ├── transaction.rs
│   │   │   │   ├── transfer.rs
│   │   │   │   ├── budget.rs
│   │   │   │   ├── bill.rs
│   │   │   │   ├── bill_payment.rs
│   │   │   │   ├── debt.rs
│   │   │   │   ├── debt_payment.rs
│   │   │   │   └── subscription.rs
│   │   │   ├── ports/
│   │   │   │   ├── inbound/                  # traits dos use cases
│   │   │   │   │   ├── auth_service.rs
│   │   │   │   │   ├── wallet_service.rs
│   │   │   │   │   ├── transaction_service.rs
│   │   │   │   │   ├── category_service.rs
│   │   │   │   │   ├── budget_service.rs
│   │   │   │   │   ├── debt_service.rs
│   │   │   │   │   └── subscription_service.rs
│   │   │   │   └── outbound/                 # traits dos repositórios e serviços externos
│   │   │   │       ├── user_repository.rs
│   │   │   │       ├── auth_token_repository.rs
│   │   │   │       ├── user_settings_repository.rs
│   │   │   │       ├── notification_settings_repository.rs
│   │   │   │       ├── institution_repository.rs
│   │   │   │       ├── wallet_repository.rs
│   │   │   │       ├── category_repository.rs
│   │   │   │       ├── transaction_repository.rs
│   │   │   │       ├── budget_repository.rs
│   │   │   │       ├── bill_repository.rs
│   │   │   │       ├── debt_repository.rs
│   │   │   │       ├── subscription_repository.rs
│   │   │   │       ├── payment_gateway.rs
│   │   │   │       ├── storage_service.rs
│   │   │   │       └── notification_service.rs
│   │   │   └── services/                     # implementações dos use cases
│   │   │       ├── auth_service.rs
│   │   │       ├── wallet_service.rs
│   │   │       ├── transaction_service.rs
│   │   │       ├── category_service.rs
│   │   │       ├── budget_service.rs
│   │   │       ├── debt_service.rs
│   │   │       └── subscription_service.rs
│   │   └── adapters/
│   │       ├── inbound/
│   │       │   └── http/
│   │       │       ├── router.rs             # Axum router
│   │       │       ├── middleware/
│   │       │       │   ├── auth.rs           # JWT extractor
│   │       │       │   └── cors.rs
│   │       │       ├── handlers/             # thin HTTP handlers
│   │       │       │   ├── auth.rs
│   │       │       │   ├── wallets.rs
│   │       │       │   ├── transactions.rs
│   │       │       │   ├── categories.rs
│   │       │       │   ├── budgets.rs
│   │       │       │   ├── debts.rs
│   │       │       │   └── subscriptions.rs
│   │       │       ├── dto/                  # Request/Response structs
│   │       │       │   ├── auth.rs
│   │       │       │   ├── wallets.rs
│   │       │       │   ├── transactions.rs
│   │       │       │   ├── categories.rs
│   │       │       │   ├── budgets.rs
│   │       │       │   ├── debts.rs
│   │       │       │   └── subscriptions.rs
│   │       │       └── webhooks/
│   │       │           ├── stripe.rs
│   │       │           └── revenuecat.rs
│   │       └── outbound/
│   │           ├── postgres/                 # implementações SQLx
│   │           │   ├── user_repository.rs
│   │           │   ├── auth_token_repository.rs
│   │           │   ├── user_settings_repository.rs
│   │           │   ├── notification_settings_repository.rs
│   │           │   ├── institution_repository.rs
│   │           │   ├── wallet_repository.rs
│   │           │   ├── category_repository.rs
│   │           │   ├── transaction_repository.rs
│   │           │   ├── budget_repository.rs
│   │           │   ├── bill_repository.rs
│   │           │   ├── debt_repository.rs
│   │           │   └── subscription_repository.rs
│   │           ├── r2/                       # Cloudflare R2 (anexos)
│   │           │   └── storage_service.rs
│   │           ├── stripe/
│   │           │   └── payment_gateway.rs
│   │           └── revenuecat/
│   │               └── payment_gateway.rs
│   ├── tests/
│   │   ├── unit/
│   │   │   ├── services/
│   │   │   │   ├── auth_service_test.rs
│   │   │   │   ├── wallet_service_test.rs
│   │   │   │   ├── transaction_service_test.rs
│   │   │   │   ├── category_service_test.rs
│   │   │   │   ├── budget_service_test.rs
│   │   │   │   ├── debt_service_test.rs
│   │   │   │   └── subscription_service_test.rs
│   │   │   └── entities/
│   │   ├── integration/
│   │   │   ├── auth_test.rs
│   │   │   ├── wallets_test.rs
│   │   │   ├── transactions_test.rs
│   │   │   ├── categories_test.rs
│   │   │   ├── budgets_test.rs
│   │   │   ├── debts_test.rs
│   │   │   └── subscriptions_test.rs
│   │   └── common/
│   │       ├── mod.rs                        # setup compartilhado
│   │       └── fixtures.rs                   # dados reutilizáveis
│   ├── migrations/
│   │   ├── 001_create_users.sql
│   │   ├── 002_create_auth_tokens.sql
│   │   ├── 003_create_user_settings.sql
│   │   ├── 004_create_notification_settings.sql
│   │   ├── 005_create_institutions.sql
│   │   ├── 006_create_wallets.sql
│   │   ├── 007_create_credit_card_details.sql
│   │   ├── 008_create_categories.sql
│   │   ├── 009_create_transactions.sql
│   │   ├── 010_create_transfers.sql
│   │   ├── 011_create_budgets.sql
│   │   ├── 012_create_bills.sql
│   │   ├── 013_create_bill_payments.sql
│   │   ├── 014_create_debts.sql
│   │   ├── 015_create_debt_payments.sql
│   │   └── 016_create_subscriptions.sql
│   ├── Cargo.toml
│   └── .env
├── web/                                      # Next.js 16 (TypeScript)
│   ├── app/
│   │   ├── (auth)/
│   │   │   ├── login/
│   │   │   │   └── page.tsx
│   │   │   ├── signup/
│   │   │   │   └── page.tsx
│   │   │   ├── forgot-password/
│   │   │   │   └── page.tsx
│   │   │   └── reset-password/
│   │   │       └── page.tsx
│   │   ├── (dashboard)/
│   │   │   ├── layout.tsx
│   │   │   ├── page.tsx                      # home/dashboard
│   │   │   ├── wallets/
│   │   │   │   ├── page.tsx
│   │   │   │   └── [id]/
│   │   │   │       └── page.tsx
│   │   │   ├── transactions/
│   │   │   │   └── page.tsx
│   │   │   ├── categories/
│   │   │   │   └── page.tsx
│   │   │   ├── budgets/
│   │   │   │   └── page.tsx
│   │   │   ├── debts/
│   │   │   │   └── page.tsx
│   │   │   ├── reports/
│   │   │   │   └── page.tsx
│   │   │   └── settings/
│   │   │       └── page.tsx
│   │   ├── layout.tsx
│   │   └── middleware.ts                      # proteção de rotas via cookie JWT
│   ├── components/
│   │   ├── ui/                               # shadcn/ui components
│   │   ├── wallets/
│   │   ├── transactions/
│   │   ├── categories/
│   │   ├── budgets/
│   │   ├── debts/
│   │   ├── reports/
│   │   └── shared/                           # componentes reutilizáveis
│   ├── lib/
│   │   ├── api.ts                            # axios client com interceptors
│   │   ├── queries/                          # TanStack Query hooks
│   │   │   ├── wallets.ts
│   │   │   ├── transactions.ts
│   │   │   ├── categories.ts
│   │   │   ├── budgets.ts
│   │   │   ├── debts.ts
│   │   │   └── subscriptions.ts
│   │   └── utils/
│   │       ├── debt-calculator.ts            # avalanche/snowball
│   │       ├── budget.ts                     # budget progress
│   │       └── formatters.ts                 # moeda, datas
│   ├── stores/                               # Zustand stores
│   │   ├── auth.ts
│   │   └── ui.ts
│   ├── types/                                # TypeScript interfaces
│   │   ├── wallet.ts
│   │   ├── transaction.ts
│   │   ├── category.ts
│   │   ├── budget.ts
│   │   ├── debt.ts
│   │   └── subscription.ts
│   ├── tests/
│   │   ├── unit/
│   │   │   ├── utils/
│   │   │   └── components/
│   │   └── integration/
│   ├── public/
│   ├── .env.local
│   └── package.json
├── mobile/                       # Flutter (Dart)
│   └── lib/
│       ├── main.dart             # Entry point + Dio setup
│       ├── features/
│       │   ├── auth/
│       │   ├── wallets/
│       │   ├── transactions/
│       │   ├── budgets/
│       │   ├── debts/
│       │   └── reports/
│       ├── services/             # API calls via Dio per entity
│       ├── models/               # Dart data classes
│       └── shared/               # Shared widgets, theme, utils
├── docker-compose.yml            # PostgreSQL local dev
├── .github/workflows/
└── CLAUDE.md
```

---

## 5. Models / Entidades

### `users`
```
id: UUID
full_name: String
email: String (unique)
password_hash: String (nullable — null para social login)
avatar_url: String (nullable)
language: String (default: "pt-BR")
region: String (default: "BR")
email_verified_at: DateTime (nullable)
created_at: DateTime
updated_at: DateTime
```

### `social_accounts`
```
id: UUID
user_id: UUID (FK users)
provider: Enum (google | apple)
provider_id: String
created_at: DateTime
```

### `institutions`
```
id: UUID
name: String
type: Enum (bank | fintech | credit_card | broker | other)
logo_url: String (nullable)
country: String (default: "BR")
is_system: Boolean (true = pré-cadastrada, false = criada pelo usuário)
created_at: DateTime
```

### `wallets`
```
id: UUID
user_id: UUID (FK users)
institution_id: UUID (nullable, FK institutions)
name: String
type: Enum (checking | savings | credit | investment | cash | digital | crypto | international)
currency: String (default: "BRL")
balance: Decimal
color: String
icon: String
is_archived: Boolean (default: false)
position: Integer (para reordenação)
created_at: DateTime
updated_at: DateTime
deleted_at: DateTime (nullable — soft delete)
```

### `credit_card_details`
```
id: UUID
wallet_id: UUID (FK wallets)
credit_limit: Decimal
closing_day: Integer (1-31)
due_day: Integer (1-31)
created_at: DateTime
updated_at: DateTime
```

### `categories`
```
id: UUID
user_id: UUID (nullable — null = categoria do sistema)
name: String
type: Enum (income | expense)
color: String
icon: String
parent_id: UUID (nullable, FK categories — para subcategorias)
is_system: Boolean (default: false)
created_at: DateTime
updated_at: DateTime
deleted_at: DateTime (nullable — soft delete)
```

### `transactions`
```
id: UUID
user_id: UUID (FK users)
wallet_id: UUID (FK wallets)
category_id: UUID (nullable, FK categories)
description: String
note: String (nullable)
amount: Decimal
type: Enum (income | expense | transfer)
date: Date
is_recurring: Boolean (default: false)
recurrence: Enum (nullable — weekly | monthly | yearly)
recurrence_end_date: Date (nullable)
attachment_url: String (nullable)
created_at: DateTime
updated_at: DateTime
deleted_at: DateTime (nullable — soft delete)
```

### `transfers`
```
id: UUID
transaction_id: UUID (FK transactions)
destination_wallet_id: UUID (FK wallets)
created_at: DateTime
```

### `budgets`
```
id: UUID
user_id: UUID (FK users)
category_id: UUID (FK categories)
amount: Decimal
month: String (formato YYYY-MM)
created_at: DateTime
updated_at: DateTime
```

### `bills`
```
id: UUID
user_id: UUID (FK users)
wallet_id: UUID (FK wallets)
category_id: UUID (nullable, FK categories)
description: String
amount: Decimal
due_day: Integer (1-31)
recurrence: Enum (weekly | monthly | yearly)
is_active: Boolean (default: true)
created_at: DateTime
updated_at: DateTime
```

### `bill_payments`
```
id: UUID
bill_id: UUID (FK bills)
user_id: UUID (FK users)
amount: Decimal
paid_at: Date
created_at: DateTime
```

### `debts`
```
id: UUID
user_id: UUID (FK users)
wallet_id: UUID (nullable, FK wallets)
name: String
creditor: String
type: Enum (credit_card | loan | mortgage | other)
original_balance: Decimal
balance: Decimal
interest_rate: Decimal
minimum_payment: Decimal
due_day: Integer (1-31)
color: String
is_paid: Boolean (default: false)
created_at: DateTime
updated_at: DateTime
deleted_at: DateTime (nullable — soft delete)
```

### `debt_payments`
```
id: UUID
debt_id: UUID (FK debts)
user_id: UUID (FK users)
amount: Decimal
is_early_payment: Boolean (default: false)
note: String (nullable)
paid_at: Date
created_at: DateTime
```

### `subscriptions`
```
id: UUID
user_id: UUID (FK users)
plan: Enum (free | premium)
status: Enum (active | canceled | past_due | trialing)
provider: Enum (stripe | revenuecat)
provider_subscription_id: String (nullable)
billing_period: Enum (monthly | yearly) (nullable — null para plano free)
current_period_start: DateTime
current_period_end: DateTime
canceled_at: DateTime (nullable)
created_at: DateTime
updated_at: DateTime
```

### `auth_tokens`
```
id: UUID
user_id: UUID (FK users)
token: String
type: Enum (password_reset | email_verification)
expires_at: DateTime
used_at: DateTime (nullable)
created_at: DateTime
```

### `user_settings`
```
id: UUID
user_id: UUID (FK users, unique)
default_currency: String (default: "BRL")
financial_month_start_day: Integer (default: 1)
theme: Enum (light | dark | system)
created_at: DateTime
updated_at: DateTime
```

### `notification_settings`
```
id: UUID
user_id: UUID (FK users, unique)
budget_alert_enabled: Boolean (default: true)
budget_alert_threshold: Integer (default: 80)
bill_due_alert_enabled: Boolean (default: true)
bill_due_alert_days_before: Integer (default: 3)
debt_due_alert_enabled: Boolean (default: true)
subscription_renewal_alert_enabled: Boolean (default: true)
created_at: DateTime
updated_at: DateTime
```

---

## 6. Services

> Todos os services vivem em `api/src/domain/services/` e implementam os traits de `domain/ports/inbound/`.
> Dependências são injetadas via traits de `domain/ports/outbound/` — nunca dependências concretas.

### `AuthService`
- `register(email, password, full_name)` — cria user + user_settings + notification_settings, envia email de verificação
- `login(email, password)` → `(user, jwt_token)` — valida senha com argon2, retorna JWT
- `login_social(provider, provider_id, email, full_name)` → `(user, jwt_token)` — cria ou recupera user via social_account
- `logout(user_id)` — invalida sessão (client-side no MVP)
- `request_password_reset(email)` — gera auth_token + envia email
- `reset_password(token, new_password)` — valida token, atualiza password_hash
- `verify_email(token)` — valida token, seta email_verified_at
- `delete_account(user_id)` — hard delete em cascata
- **Dependências**: `UserRepository`, `AuthTokenRepository`, `NotificationService`

### `WalletService`
- `list(user_id, filters)` → `PaginatedResponse<Wallet>` — lista carteiras ativas (deleted_at IS NULL)
- `find(user_id, wallet_id)` → `Wallet`
- `create(user_id, payload)` → `Wallet` — verifica limite do plano free, cria carteira + credit_card_details se type = credit
- `update(user_id, wallet_id, payload)` → `Wallet`
- `archive(user_id, wallet_id)` → `Wallet` — seta is_archived = true
- `delete(user_id, wallet_id)` — soft delete (seta deleted_at)
- `reorder(user_id, wallet_ids)` — atualiza position de cada carteira
- `net_worth(user_id)` → `Decimal` — soma de todos os saldos convertidos para moeda padrão
- `top_expenses(user_id, wallet_id, month)` → `Vec<TopExpense>`
- **Dependências**: `WalletRepository`, `CreditCardDetailsRepository`, `SubscriptionRepository`

### `TransactionService`
- `list(user_id, filters)` → `PaginatedResponse<Transaction>` — filtros: período, categoria, carteira, tipo
- `find(user_id, transaction_id)` → `Transaction`
- `create(user_id, payload)` → `Transaction` — DB transaction: INSERT transactions + UPDATE wallets.balance; se type = transfer, cria também em `transfers`
- `update(user_id, transaction_id, payload)` → `Transaction` — DB transaction: atualiza + recalcula wallets.balance
- `delete(user_id, transaction_id)` — soft delete + reverte wallets.balance em DB transaction
- `search(user_id, query)` → `PaginatedResponse<Transaction>` — busca por descrição ou nota
- `summary(user_id, period)` → `TransactionSummary` — total entradas, saídas, saldo
- `monthly_comparison(user_id, month_a, month_b)` → `MonthlyComparison`
- `upload_attachment(user_id, transaction_id, file)` → `String` — upload para R2, atualiza attachment_url
- **Dependências**: `TransactionRepository`, `WalletRepository`, `StorageService`

### `CategoryService`
- `list(user_id)` → `Vec<Category>` — categorias do sistema + categorias do usuário
- `find(user_id, category_id)` → `Category`
- `create(user_id, payload)` → `Category`
- `update(user_id, category_id, payload)` → `Category` — somente is_system = false
- `delete(user_id, category_id)` — soft delete; valida que não há transações ativas vinculadas
- `list_subcategories(user_id, parent_id)` → `Vec<Category>`
- **Dependências**: `CategoryRepository`, `TransactionRepository`

### `BudgetService`
- `list(user_id, month)` → `Vec<BudgetWithProgress>` — orçamentos com gasto atual vs. limite
- `upsert(user_id, category_id, month, amount)` → `Budget`
- `delete(user_id, budget_id)` — hard delete
- `copy_from_previous_month(user_id, month)` → `Vec<Budget>`
- `summary(user_id, month)` → `BudgetSummary` — total planejado vs. total gasto
- **Dependências**: `BudgetRepository`, `TransactionRepository`

### `BillService`
- `list(user_id, filters)` → `PaginatedResponse<Bill>`
- `find(user_id, bill_id)` → `Bill`
- `create(user_id, payload)` → `Bill`
- `update(user_id, bill_id, payload)` → `Bill`
- `deactivate(user_id, bill_id)` → `Bill` — seta is_active = false
- `delete(user_id, bill_id)` — hard delete
- `upcoming(user_id, days)` → `Vec<BillWithStatus>` — contas a vencer nos próximos N dias
- `pay(user_id, bill_id, amount, paid_at)` → `BillPayment`
- `payment_history(user_id, bill_id)` → `Vec<BillPayment>`
- `monthly_summary(user_id, month)` → `BillSummary` — total contas, total pago, total pendente
- **Dependências**: `BillRepository`, `BillPaymentRepository`

### `DebtService`
- `list(user_id)` → `Vec<Debt>`
- `find(user_id, debt_id)` → `Debt`
- `create(user_id, payload)` → `Debt`
- `update(user_id, debt_id, payload)` → `Debt`
- `mark_as_paid(user_id, debt_id)` → `Debt`
- `delete(user_id, debt_id)` — soft delete
- `log_payment(user_id, debt_id, amount, is_early, note)` → `DebtPayment` — registra pagamento + atualiza debt.balance
- `payment_history(user_id, debt_id)` → `Vec<DebtPayment>`
- `avalanche_plan(user_id, extra_payment)` → `PayoffPlan`
- `snowball_plan(user_id, extra_payment)` → `PayoffPlan`
- `simulate_early_payment(user_id, debt_id, amount)` → `EarlyPaymentSimulation`
- **Dependências**: `DebtRepository`, `DebtPaymentRepository`

### `SubscriptionService`
- `find(user_id)` → `Subscription`
- `get_plan(user_id)` → `Plan`
- `create_checkout(user_id, billing_period, provider)` → `CheckoutUrl`
- `cancel(user_id)` → `Subscription`
- `handle_stripe_webhook(payload, signature)` — processa eventos Stripe
- `handle_revenuecat_webhook(payload, signature)` — processa eventos RevenueCat
- **Dependências**: `SubscriptionRepository`, `PaymentGateway`

### `ReportService`
- `dashboard(user_id, month)` → `Dashboard` — resumo do mês + patrimônio líquido
- `expenses_by_category(user_id, period)` → `Vec<CategoryExpense>`
- `income_by_category(user_id, period)` → `Vec<CategoryIncome>`
- `cashflow(user_id, period)` → `Vec<MonthlyCashflow>`
- `income_vs_expense(user_id, year)` → `Vec<MonthlyBalance>`
- `spending_trends(user_id, period)` → `SpendingTrends`
- `debt_progress(user_id)` → `DebtProgress`
- `export(user_id, format)` → `Vec<u8>` — CSV ou JSON
- **Dependências**: `TransactionRepository`, `WalletRepository`, `DebtRepository`, `BudgetRepository`, `BillRepository`

### `UserSettingsService`
- `find(user_id)` → `UserSettings`
- `update(user_id, payload)` → `UserSettings`
- **Dependências**: `UserSettingsRepository`

### `NotificationSettingsService`
- `find(user_id)` → `NotificationSettings`
- `update(user_id, payload)` → `NotificationSettings`
- **Dependências**: `NotificationSettingsRepository`

### `InstitutionService`
- `list(filters)` → `Vec<Institution>` — instituições do sistema + criadas pelo usuário
- `find(institution_id)` → `Institution`
- `create(user_id, payload)` → `Institution`
- **Dependências**: `InstitutionRepository`

---

## 7. Jobs / Workers

Não há jobs/workers em background no MVP. Notificações push (budget warnings, debt due dates) são disparadas via Firebase Cloud Messaging no lado mobile, configuradas no Phase 8.

---

## 8. Common Hurdles ⚠️

> **Esta é a seção mais valiosa do documento.**
> Documente SEMPRE que resolver um problema que levou mais de 15 minutos para entender.
> A próxima sessão começa do zero — este arquivo é a memória persistente do projeto.

<!-- Adicione novos hurdles aqui conforme aparecem -->

---

## 9. Design Patterns do Projeto

### P1: Isolamento por camada no backend Rust
```
Handler → Service → Query (SQLx)
```
- **Handler**: só HTTP (deserializar, validar, chamar service, serializar)
- **Service**: lógica de negócio, orquestra queries, gerencia DB transactions
- **Query**: só SQL via SQLx — funções puras que recebem `PgPool`

### P2: Mutations de transação atualizam saldo da carteira dentro de uma DB transaction Rust
`TransactionService` abre uma `PgTransaction`, insere em `transactions` e atualiza `wallets.balance` atomicamente. **Nunca** atualizar `balance` em queries separadas sem transaction.
```rust
let mut tx = pool.begin().await?;
queries::insert_transaction(&mut tx, &payload).await?;
queries::update_wallet_balance(&mut tx, wallet_id, delta).await?;
tx.commit().await?;
```

### P3: Queries web sempre via TanStack Query — nunca fetch direto em componentes
Todo acesso a dados no Next.js passa por hooks em `web/lib/queries/` que usam `lib/api.ts`.

### P4: Mobile — providers Riverpod por feature, não globais
Cada feature tem seus próprios providers. Evitar god-providers globais.

### P5: AppError unificado no backend Rust
Todos os erros retornam `AppError` que implementa `IntoResponse`:
```rust
pub enum AppError {
    NotFound(String),
    Unauthorized,
    Validation(String),
    Database(sqlx::Error),
    Internal(anyhow::Error),
}
```
**Nunca** retornar `500` com mensagem de erro interna exposta ao cliente.

### P6: Autenticação via JWT no header Authorization
```
Authorization: Bearer <token>
```
Middleware Axum extrai e valida o token em todas as rotas protegidas. O `user_id` é injetado no handler via extractor.

### P7: Datas sempre em UTC no banco, exibidas no timezone local do usuário
Armazenar `date` e `created_at` como `TIMESTAMPTZ` (UTC). Conversão para timezone local apenas na camada de exibição.

### P8: Domain não conhece infraestrutura (Hexagonal)
O código em `domain/` nunca importa Axum, SQLx, Serde, ou qualquer crate de infraestrutura. Só Rust puro + traits.
```rust
// ✅ Correto — domain/services/wallet_service.rs
use crate::domain::ports::outbound::wallet_repository::WalletRepository;

// ❌ Errado
use sqlx::PgPool; // infraestrutura vazando para o domínio
```

### P9: DTOs separados das entities de domínio
Entities de domínio (`domain/entities/`) representam o estado interno. DTOs (`adapters/inbound/http/dto/`) são os contratos HTTP.
**Nunca** expor uma entity de domínio diretamente como resposta de API — sempre converter para DTO.
```rust
// ✅ Correto
let wallet = wallet_service.find(id).await?;
Json(WalletResponse::from(wallet)) // DTO

// ❌ Errado
Json(wallet) // entity de domínio exposta diretamente
```

### P10: Soft delete para dados financeiros, hard delete para preferências
- **Soft delete** (`deleted_at: DateTime`): `wallets`, `transactions`, `debts`, `categories` — dados financeiros nunca são perdidos permanentemente
- **Hard delete**: `user_settings`, `notification_settings`, `auth_tokens` — sem valor histórico
- Exceção: deletar conta (`DELETE /users/me`) faz hard delete em cascata de tudo

### P11: Paginação em todas as listagens
Toda query que retorna uma lista usa paginação com cursor ou offset:
```rust
pub struct PaginationParams {
    pub page: i64,    // default: 1
    pub limit: i64,   // default: 20, max: 100
}

pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub limit: i64,
}
```
**Nunca** retornar lista sem paginação — protege contra queries que retornam milhares de registros.

### P12: Validação no handler, lógica de negócio no service
- **Handler**: valida formato, tipos, campos obrigatórios (usando `validator` crate)
- **Service**: valida regras de negócio (ex: limite de 2 carteiras no plano free, saldo suficiente)
```rust
// ✅ Handler valida formato
#[derive(Validate)]
pub struct CreateWalletRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
}

// ✅ Service valida regra de negócio
if user_plan == Plan::Free && wallet_count >= 2 {
    return Err(AppError::PlanLimitReached);
}
```

### P13: Categorias do sistema via seed na migration
Categorias padrão (Alimentação, Transporte, Saúde, etc.) são inseridas via `migration/008_create_categories.sql` com `user_id = NULL` e `is_system = true`.
**Nunca** inserir categorias do sistema via código da aplicação ou seed separado.

### P14: Free tier enforced no service layer
O limite do plano free (ex: máximo 2 carteiras) é verificado **sempre** no service, antes de qualquer operação de criação.
```rust
// WalletService::create()
let count = self.wallet_repo.count_by_user(user_id).await?;
let plan = self.subscription_repo.find_plan(user_id).await?;
if plan == Plan::Free && count >= 2 {
    return Err(AppError::PlanLimitReached("wallet".into()));
}
```
**Nunca** depender do frontend para enforçar limites de plano.

---

## 10. Pipelines Principais

### Lançamento de Transação
```
1. [UI] Usuário preenche form (valor, tipo, categoria, carteira, data, nota)
2. [API — Handler] POST /transactions → valida payload
3. [API — Service] Abre DB transaction → INSERT transactions + UPDATE wallets.balance (atômico)
4. [API] Retorna transaction criada
5. [TanStack Query / Riverpod] Invalida cache de wallets + transactions
6. [UI] Saldo da carteira e lista de transações refletem o novo estado
```

### Cálculo de Budget Progress
```
1. [UI] Usuário abre tela de Budgets (mês atual)
2. [Query] Busca budgets do mês + transactions do mês agrupadas por category_id
3. [Utils] getBudgetProgress() calcula % gasto vs. limite por categoria
4. [UI] Exibe progress bars por categoria
```

### Payoff Planner (Dívidas)
```
1. [UI] Usuário informa valor extra disponível por mês
2. [Utils] calculateAvalanche() ou calculateSnowball() processa lista de debts
3. [UI] Exibe timeline de quitação mês a mês + data estimada de livre de dívidas
```

---

## 11. Checklist Pós-Implementação

> Trate como gate de qualidade, não sugestão. Feature não está pronta sem isso.
> Seguimos XP: nenhum código sem teste, nenhum merge com testes falhando.

**TDD (Red → Green → Refactor)**
- [ ] Teste unitário escrito ANTES do código de produção
- [ ] Teste de integração cobrindo o handler HTTP
- [ ] Edge cases cobertos: input inválido, recurso inexistente, permissão negada, limite de plano
- [ ] Todos os testes passando (`cargo test` / `npm test`)
- [ ] User B não consegue acessar dados do User A (teste de isolamento)

**Código**
- [ ] `cargo clippy` sem warnings
- [ ] `npm run lint` sem warnings
- [ ] Build compila sem erros (`cargo build` / `npm run build`)
- [ ] Sem `console.log` esquecido, sem `any` no TypeScript
- [ ] Domain não importa infraestrutura (P8 respeitado)
- [ ] Soft delete aplicado onde necessário (P10 respeitado)
- [ ] Todas as listagens paginadas (P11 respeitado)
- [ ] Validação no handler, lógica no service (P12 respeitado)

**Segurança**
- [ ] `cargo audit` sem vulnerabilidades críticas
- [ ] `cargo deny` sem violações
- [ ] `npm audit` sem vulnerabilidades críticas
- [ ] Free tier enforced no service layer (P14 respeitado)

**Documentação**
- [ ] Novas variáveis de ambiente documentadas na seção 3
- [ ] Novo hurdle descoberto? → seção 8
- [ ] Novo padrão estabelecido? → seção 9
- [ ] Decisão arquitetural tomada? → seção 14

**Deploy**
- [ ] CI verde antes do merge
- [ ] Variáveis de ambiente configuradas em produção
- [ ] Migration executada (`sqlx migrate run`)
- [ ] Smoke test no ambiente de staging

---

## 12. Processo de Desenvolvimento

### Metodologia: Extreme Programming (XP)

**Práticas adotadas:**
- **TDD (Test-Driven Development)**: testes são escritos ANTES do código de produção. Nenhuma feature é implementada sem teste correspondente.
- **Small releases**: cada entrega é pequena, funcional e deployável. Nada de branches gigantes.
- **Simple design**: implementar apenas o necessário para os testes passarem. Sem over-engineering.
- **Refactoring contínuo**: o código é melhorado continuamente. Toda sessão pode incluir refactoring de código existente.
- **Collective ownership**: qualquer parte do código pode ser alterada a qualquer momento sem burocracia.
- **Pair programming**: todas as implementações são feitas em conjunto (usuário + agente).

**Fluxo de trabalho por feature:**
```
1. Escrever teste que falha (Red)
2. Implementar o mínimo para o teste passar (Green)
3. Refatorar mantendo os testes passando (Refactor)
4. Repetir
```

**Regras:**
- Nunca commitar código com testes falhando
- Nunca implementar código sem teste cobrindo
- Se surgir dúvida sobre design, escolher a opção mais simples

---

### Branching: Gitflow

**Branches permanentes (protegidas no GitHub):**
- `main` — produção. Só recebe merges de `release/*` e `hotfix/*`
- `develop` — integração. Toda feature é mergeada aqui primeiro

**Branches de curta duração:**
```
feature/<descricao-curta>   ex: feature/create-wallet-endpoint
bugfix/<descricao-curta>    ex: bugfix/fix-jwt-expiry
release/<versao>            ex: release/1.0.0
hotfix/<descricao-curta>    ex: hotfix/fix-critical-auth-bug
```

**Fluxo padrão de feature:**
```
develop → feature/xxx → PR para develop → merge → deletar branch
```

**Fluxo de release:**
```
develop → release/x.y.z → ajustes finais → merge em main + develop → tag vx.y.z
```

**Fluxo de hotfix:**
```
main → hotfix/xxx → fix → merge em main + develop → tag vx.y.z
```

**Regras:**
- Nunca commitar diretamente em `main` ou `develop`
- Todo merge via Pull Request com CI verde obrigatório
- Branch names devem seguir o padrão acima (validado pelo hook pre-push)

---

## 14. Decisões Arquiteturais (ADRs)

### ADR-001: Rust + Axum + SQLx como backend
**Decisão**: API REST em Rust com Axum e SQLx (sem ORM).
**Alternativas consideradas**: Supabase (BaaS), Node/NestJS, Go.
**Motivo**: Performance, segurança de tipos em tempo de compilação (SQLx valida queries contra o DB), controle total sobre a lógica de negócio.
**Trade-offs**: Curva de aprendizado maior, mais código boilerplate vs. Supabase. Setup local requer PostgreSQL rodando separadamente.
**Data**: 2026-03-11

### ADR-002: Web e Mobile como apps separados (sem monorepo TS)
**Decisão**: `web/` (Next.js/TypeScript) e `mobile/` (Flutter/Dart) são apps independentes compartilhando apenas a API Rust.
**Alternativas consideradas**: Turborepo monorepo com pacotes compartilhados.
**Motivo**: Web e Mobile usam linguagens diferentes (TS vs Dart) — não há código a compartilhar. Monorepo adicionaria complexidade sem benefício real.
**Trade-offs**: Tipos e lógica de cálculo (avalanche/snowball) precisam ser escritos tanto em TS quanto em Dart.
**Data**: 2026-03-11

### ADR-004: JWT em cookie httpOnly (não localStorage)
**Decisão**: O backend Rust seta o JWT em cookie httpOnly na resposta de login. O frontend nunca toca no token diretamente.
**Alternativas consideradas**: localStorage, next-auth com credentials provider.
**Motivo**: Cookie httpOnly é inacessível via JavaScript — protege contra XSS. Next.js middleware lê o cookie para proteger rotas sem expor o token ao cliente.
**Trade-offs**: Requer configuração de CORS com `credentials: true` e `SameSite` adequado. Mobile não usa cookie — recebe o JWT no body e armazena em flutter_secure_storage.
**Data**: 2026-03-11

### ADR-003: Saldo de carteira atualizado atomicamente no Service Rust (DB transaction)
**Decisão**: `wallets.balance` é atualizado dentro de uma DB transaction Rust junto com o INSERT de `transactions`.
**Alternativas consideradas**: Trigger PostgreSQL, calcular via SUM no cliente.
**Motivo**: Lógica de negócio fica no Rust (testável, tipada), não espalhada em SQL. DB transaction garante atomicidade sem depender de triggers.
**Trade-offs**: Se alguém alterar `transactions` diretamente no banco (sem passar pela API), o balance ficará inconsistente.
**Data**: 2026-03-11

---

## 15. Contexto de Negócio

- **Usuário principal**: Pessoa física que quer controle financeiro pessoal — especialmente quem tem dívidas ativas e quer um plano claro para quitá-las.
- **Entrada de dados**: Manual (sem integração bancária automática no MVP). O usuário lança as transações.
- **Regras de negócio críticas**:
  - Saldo de carteira nunca pode ficar desatualizado em relação às transações.
  - Dados de um usuário NUNCA podem ser visíveis a outro usuário.
  - Payoff planner deve sempre mostrar ambos os métodos (avalanche e snowball) para o usuário comparar.
- **Terminologia do domínio**:
  - **Carteira** (wallet): conta bancária, cartão de crédito, dinheiro em espécie ou investimento.
  - **Categoria**: classificação de uma transação (ex: Alimentação, Transporte, Salário).
  - **Orçamento** (budget): limite mensal de gastos por categoria.
  - **Dívida** (debt): valor devido a um credor com juros e prazo.
  - **Avalanche**: estratégia de pagamento priorizando maior taxa de juros.
  - **Snowball**: estratégia de pagamento priorizando menor saldo.
- **Restrições legais/compliance**: LGPD — dados financeiros pessoais. Não coletar dados além do necessário.

---

---

## 16. User Stories

### Autenticação
1. Como usuário, quero me cadastrar com email e senha
2. Como usuário, quero fazer login com email e senha
3. Como usuário, quero fazer login com Google
4. Como usuário, quero fazer login com Apple
5. Como usuário, quero manter minha sessão ativa entre acessos
6. Como usuário, quero fazer logout
7. Como usuário, quero recuperar minha senha via email
8. Como usuário, quero redefinir minha senha através de um link enviado por email
9. Como usuário, quero alterar minha senha estando logado
10. Como usuário, quero atualizar meu perfil (nome, foto)
11. Como usuário, quero deletar minha conta e todos os meus dados
12. Como usuário, quero verificar meu email após o cadastro

### Planos & Assinatura
1. Como usuário, quero ver o que está incluído no plano free vs. premium
2. Como usuário, quero fazer upgrade para o plano premium
3. Como usuário, quero assinar o plano premium mensalmente ou anualmente
4. Como usuário, quero cancelar minha assinatura
5. Como usuário, quero ver o status da minha assinatura (ativa, cancelada, data de renovação)
6. Como usuário, quero ser notificado antes da renovação automática
7. Como usuário (free), quero ser informado quando atingir o limite de 2 carteiras

### Configurações
1. Como usuário, quero definir a moeda padrão do app
2. Como usuário, quero escolher o tema (claro/escuro/sistema)
3. Como usuário, quero definir o dia de início do mês financeiro (ex: dia 5 em vez de dia 1)
4. Como usuário, quero ativar ou desativar a proteção biométrica no app (mobile)
5. Como usuário, quero fazer backup dos meus dados
6. Como usuário, quero restaurar um backup

### Notificações
1. Como usuário, quero receber notificação quando atingir 80% de um orçamento
2. Como usuário, quero receber notificação quando ultrapassar o limite de um orçamento
3. Como usuário, quero receber notificação de contas próximas do vencimento
4. Como usuário, quero receber notificação no dia de vencimento de uma dívida
5. Como usuário, quero ativar ou desativar cada tipo de notificação
6. Como usuário, quero configurar com quantos dias de antecedência receber alertas de vencimento

### Perfil do Usuário
1. Como usuário, quero ver meu perfil com nome completo, email e foto
2. Como usuário, quero editar meu nome completo
3. Como usuário, quero editar meu email
4. Como usuário, quero alterar minha senha
5. Como usuário, quero escolher o idioma do app
6. Como usuário, quero configurar minha região (formato de moeda, data, separadores)
7. Como usuário, quero deletar minha conta e todos os meus dados permanentemente

### Relatórios & Insights

**Visão Geral**
1. Como usuário, quero ver um dashboard com resumo financeiro do mês (entradas, saídas, saldo)
2. Como usuário, quero ver meu patrimônio líquido total (carteiras - dívidas)
3. Como usuário, quero ver um resumo de quanto economizei no mês

**Gastos**
4. Como usuário, quero ver meus gastos por categoria em gráfico de pizza/donut
5. Como usuário, quero ver a evolução de gastos por categoria ao longo dos meses em gráfico de linha
6. Como usuário, quero ver meus maiores gastos do mês
7. Como usuário, quero ver em quais dias do mês gasto mais

**Receitas**
8. Como usuário, quero ver minhas receitas por categoria no mês
9. Como usuário, quero ver a evolução das minhas receitas ao longo dos meses

**Fluxo de Caixa**
10. Como usuário, quero ver o fluxo de caixa mensal (entradas vs. saídas em gráfico de barras)
11. Como usuário, quero ver a projeção do saldo futuro com base nas contas a pagar e receber

**Comparativo Income vs Expense**
12. Como usuário, quero ver a comparação entre receitas e despesas no mês atual
13. Como usuário, quero ver a evolução da relação receita vs. despesa ao longo dos meses
14. Como usuário, quero ver se estou no positivo ou negativo em cada mês do ano

**Insights**
15. Como usuário, quero receber insights automáticos (ex: "você gastou 30% a mais em alimentação este mês")
16. Como usuário, quero ver uma análise de tendências dos meus gastos
17. Como usuário, quero ver alertas de comportamentos financeiros incomuns

**Debt Progress**
18. Como usuário, quero ver um relatório de evolução das minhas dívidas ao longo do tempo
19. Como usuário, quero ver o quanto já paguei em juros vs. principal
20. Como usuário, quero ver a projeção de redução do total de dívidas mês a mês
21. Como usuário, quero ver o impacto dos pagamentos antecipados no total de juros pago

**Exportação**
22. Como usuário, quero exportar meus dados financeiros em CSV ou PDF

### Dívidas

**Geral**
1. Como usuário, quero cadastrar uma dívida com nome, credor, saldo, taxa de juros e parcela mínima
2. Como usuário, quero informar o tipo da dívida (cartão de crédito, empréstimo, financiamento, outro)
3. Como usuário, quero associar uma dívida a uma carteira
4. Como usuário, quero editar os dados de uma dívida
5. Como usuário, quero marcar uma dívida como quitada
6. Como usuário, quero deletar uma dívida
7. Como usuário, quero ver a lista de todas as dívidas ativas
8. Como usuário, quero ver o total de dívidas consolidado

**Pagamentos**
9. Como usuário, quero registrar um pagamento parcial ou total de uma dívida
10. Como usuário, quero ver o histórico de pagamentos de uma dívida
11. Como usuário, quero ver o saldo restante após cada pagamento

**Plano de Quitação**
12. Como usuário, quero informar um valor extra disponível por mês para quitar dívidas
13. Como usuário, quero ver o plano de quitação pelo método avalanche (maior juros primeiro)
14. Como usuário, quero ver o plano de quitação pelo método snowball (menor saldo primeiro)
15. Como usuário, quero comparar os dois métodos lado a lado
16. Como usuário, quero ver a data estimada de quitação de cada dívida
17. Como usuário, quero ver o total de juros que pagarei em cada método
18. Como usuário, quero ver a projeção de quando estarei completamente livre de dívidas

**Progresso**
19. Como usuário, quero ver o progresso de quitação de cada dívida (% pago)
20. Como usuário, quero ver a evolução do saldo total de dívidas ao longo do tempo

**Antecipação**
21. Como usuário, quero simular a antecipação de parcelas de uma dívida
22. Como usuário, quero ver quanto economizo em juros ao antecipar parcelas
23. Como usuário, quero registrar uma antecipação de parcelas

### Orçamentos

**Geral**
1. Como usuário, quero definir um limite de gastos mensal por categoria
2. Como usuário, quero editar o limite de um orçamento
3. Como usuário, quero deletar um orçamento
4. Como usuário, quero ver todos os orçamentos do mês atual
5. Como usuário, quero copiar os orçamentos do mês anterior para o mês atual

**Acompanhamento**
6. Como usuário, quero ver quanto já gastei vs. o limite de cada categoria
7. Como usuário, quero ver a porcentagem consumida de cada orçamento
8. Como usuário, quero receber um alerta quando atingir 80% de um orçamento
9. Como usuário, quero receber um alerta quando ultrapassar o limite de um orçamento
10. Como usuário, quero ver quais categorias estão dentro e fora do orçamento

**Análise**
11. Como usuário, quero ver um resumo geral do orçamento do mês (total planejado vs. total gasto)
12. Como usuário, quero ver o histórico de orçamentos de meses anteriores
13. Como usuário, quero ver se estou melhorando ou piorando no cumprimento dos orçamentos mês a mês

### Categorias

**Geral**
1. Como usuário, quero ver a lista de categorias disponíveis
2. Como usuário, quero criar uma categoria personalizada com nome, cor e ícone
3. Como usuário, quero editar uma categoria personalizada
4. Como usuário, quero deletar uma categoria personalizada
5. Como usuário, quero usar categorias do sistema pré-cadastradas (Alimentação, Transporte, etc.)

**Subcategorias**
6. Como usuário, quero criar subcategorias dentro de uma categoria (ex: Alimentação > Restaurante)
7. Como usuário, quero associar uma transação a uma subcategoria

**Análise por Categoria**
8. Como usuário, quero ver o total gasto por categoria no mês
9. Como usuário, quero ver a porcentagem de cada categoria no total de gastos
10. Como usuário, quero ver a evolução de gastos de uma categoria ao longo dos meses

### Transações

**Geral**
1. Como usuário, quero lançar uma transação de entrada (receita)
2. Como usuário, quero lançar uma transação de saída (despesa)
3. Como usuário, quero lançar uma transferência entre carteiras
4. Como usuário, quero associar uma transação a uma categoria
5. Como usuário, quero associar uma transação a uma carteira
6. Como usuário, quero adicionar uma descrição à transação (ex: "Supermercado Extra")
7. Como usuário, quero adicionar uma nota opcional com detalhes adicionais
8. Como usuário, quero informar a data da transação (não necessariamente hoje)
9. Como usuário, quero editar uma transação lançada
10. Como usuário, quero deletar uma transação
11. Como usuário, quero ver a lista de todas as transações

**Filtros & Busca**
12. Como usuário, quero filtrar transações por período
13. Como usuário, quero filtrar transações por categoria
14. Como usuário, quero filtrar transações por carteira
15. Como usuário, quero filtrar transações por tipo (entrada/saída/transferência)
16. Como usuário, quero buscar transações por descrição ou nota

**Recorrência**
17. Como usuário, quero marcar uma transação como recorrente (semanal, mensal, anual)
18. Como usuário, quero editar ou cancelar uma recorrência

**Anexos**
19. Como usuário, quero anexar um comprovante ou foto a uma transação

**Resumo & Totais**
20. Como usuário, quero ver o saldo total do período (total de entradas - total de saídas)
21. Como usuário, quero ver o total de contas a pagar do mês (ex: 10 contas, R$ 2.340,00)
22. Como usuário, quero ver o total já pago e o total pendente do mês

**Comparativo**
23. Como usuário, quero comparar meus gastos mês a mês (ex: setembro vs outubro)
24. Como usuário, quero ver se gastei mais ou menos do que no mês anterior por categoria

### Carteiras

**Geral**
1. Como usuário, quero criar uma carteira informando nome, tipo, moeda e saldo inicial
2. Como usuário, quero associar uma carteira a uma instituição (banco, fintech, operadora)
3. Como usuário, quero ver a lista de todas as minhas carteiras
4. Como usuário, quero ver o saldo atual de cada carteira
5. Como usuário, quero editar os dados de uma carteira
6. Como usuário, quero arquivar uma carteira que não uso mais
7. Como usuário, quero deletar uma carteira
8. Como usuário, quero ver o patrimônio líquido total (soma de todas as carteiras)
9. Como usuário, quero personalizar uma carteira com cor e ícone
10. Como usuário, quero reordenar minhas carteiras

**Cartão de Crédito**
11. Como usuário, quero informar o limite total do cartão
12. Como usuário, quero informar o dia de fechamento da fatura
13. Como usuário, quero informar o dia de vencimento da fatura
14. Como usuário, quero ver o limite disponível (limite total - fatura atual)
15. Como usuário, quero ver a fatura do mês atual
16. Como usuário, quero ver faturas anteriores
17. Como usuário, quero registrar o pagamento da fatura

**Conta Internacional**
18. Como usuário, quero criar uma conta com moeda estrangeira (USD, EUR, GBP, etc.)
19. Como usuário, quero ver o saldo em moeda original e o equivalente em BRL

**Investimentos**
20. Como usuário, quero registrar aportes na carteira de investimentos
21. Como usuário, quero atualizar o saldo atual dos investimentos manualmente

**Instituições**
22. Como usuário, quero selecionar uma instituição de uma lista pré-cadastrada
23. Como usuário, quero cadastrar uma instituição manualmente caso não encontre na lista

**Dívidas associadas à carteira**
24. Como usuário, quero ver o total de dívidas associadas a uma carteira
25. Como usuário, quero registrar um pagamento de dívida debitando de uma carteira específica
26. Como usuário, quero ver o histórico de pagamentos de uma carteira

**Top Expenses**
27. Como usuário, quero ver os maiores gastos de uma carteira no mês
28. Como usuário, quero ver os maiores gastos de uma carteira por categoria

**Pagamentos**
29. Como usuário, quero registrar uma transferência entre duas carteiras minhas
30. Como usuário, quero ver o histórico completo de transações de uma carteira
31. Como usuário, quero filtrar as transações de uma carteira por período, categoria e tipo

**Budget Planner por Carteira**
32. Como usuário, quero ver o orçamento mensal alocado para gastos de uma carteira
33. Como usuário, quero acompanhar quanto do orçamento já foi consumido nesta carteira

**Recovery Progress**
34. Como usuário, quero ver o progresso de quitação das dívidas associadas a uma carteira
35. Como usuário, quero ver a projeção de quando a dívida de uma carteira será quitada

**Bills & Payments**
36. Como usuário, quero cadastrar contas a pagar recorrentes (aluguel, streaming, assinaturas)
37. Como usuário, quero ver as próximas contas a vencer de uma carteira
38. Como usuário, quero registrar o pagamento de uma conta
39. Como usuário, quero receber alertas de contas próximas do vencimento

**Dívidas por Carteira**
40. Como usuário, quero ver todas as dívidas associadas a uma carteira específica
41. Como usuário, quero ver o total de dívidas de uma carteira

**Dívidas Geral**
42. Como usuário, quero ver todas as minhas dívidas consolidadas
43. Como usuário, quero adicionar uma nova dívida
44. Como usuário, quero ver o plano de quitação pelo método avalanche
45. Como usuário, quero ver o plano de quitação pelo método snowball
46. Como usuário, quero ver a projeção de quando estarei livre de todas as dívidas

---

> **Regra de ouro**: Se você resolveu um problema que levou mais de 15 minutos para
> entender, documente aqui. Se tomou uma decisão arquitetural, documente aqui.
> Este arquivo é a memória persistente do projeto entre sessões de IA.
