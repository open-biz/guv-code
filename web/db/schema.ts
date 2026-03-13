import { pgTable, text, timestamp, integer, uuid, boolean } from 'drizzle-orm/pg-core'

export const users = pgTable('users', {
  id: uuid('id').primaryKey().defaultRandom(),
  githubId: text('github_id').unique().notNull(),
  name: text('name'),
  email: text('email').unique(),
  image: text('image'),
  plan: text('plan').default('free').notNull(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
  updatedAt: timestamp('updated_at').defaultNow().notNull(),
})

export const apiKeys = pgTable('api_keys', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id')
    .references(() => users.id)
    .notNull(),
  name: text('name').notNull(),
  keyHash: text('key_hash').notNull(),
  keyPrefix: text('key_prefix').notNull(),
  isActive: boolean('is_active').default(true).notNull(),
  lastUsedAt: timestamp('last_used_at'),
  createdAt: timestamp('created_at').defaultNow().notNull(),
})

export const usageRecords = pgTable('usage_records', {
  id: uuid('id').primaryKey().defaultRandom(),
  userId: uuid('user_id')
    .references(() => users.id)
    .notNull(),
  model: text('model').notNull(),
  provider: text('provider').notNull(),
  tokensInput: integer('tokens_input').default(0).notNull(),
  tokensOutput: integer('tokens_output').default(0).notNull(),
  costCents: integer('cost_cents').default(0).notNull(),
  createdAt: timestamp('created_at').defaultNow().notNull(),
})
