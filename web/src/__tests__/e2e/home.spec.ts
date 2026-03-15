import { test, expect } from '@playwright/test'

test('homepage has title', async ({ page }) => {
  await page.goto('/')
  await expect(page).toHaveTitle(/GuvCode/)
})

test('homepage has hero heading', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByRole('heading', { name: /Ship code faster/i }),
  ).toBeVisible()
})

test('login link works', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('link', { name: /Get Started/i }).click()
  await expect(page).toHaveURL(/\/login/)
})
