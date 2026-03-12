CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email               TEXT NOT NULL,
    password_hash       TEXT,                        -- NULL for social-only accounts
    full_name           TEXT NOT NULL,
    avatar_url          TEXT,
    email_verified_at   TIMESTAMPTZ,
    plan                TEXT NOT NULL DEFAULT 'free'
                            CHECK (plan IN ('free', 'premium')),
    failed_attempts     INTEGER NOT NULL DEFAULT 0,
    locked_until        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

-- Unique email only among active (non-deleted) users
CREATE UNIQUE INDEX idx_users_email_active ON users (email) WHERE deleted_at IS NULL;
