import { test, expect } from "@playwright/test";

test("homepage displays correct title and content", async ({ page }) => {
  await page.goto("http://localhost:3000/");

  // Check page title
  await expect(page).toHaveTitle("Rusty notes");

  // Check main heading and subtitle
  await expect(page.locator("h1")).toHaveText("Rusty notes");
  await expect(page.locator(".subtitle")).toHaveText("Digital Zettelkasten");

  // Check for notes grid
  await expect(page.locator(".notes-grid")).toBeVisible();

  // Check for navigation
  await expect(page.locator("nav.sidebar")).toBeVisible();
  await expect(page.locator(".nav-links a")).toHaveText("Notes");
});
