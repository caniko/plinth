import { test, expect } from "@playwright/test";

test.describe("/posts page", () => {
  test("loads without errors", async ({ page }) => {
    await page.goto("/posts");
    // Should not show error text
    await expect(page.locator("text=Error")).not.toBeVisible();
  });

  test("displays page heading", async ({ page }) => {
    await page.goto("/posts");
    await expect(page.locator("h1")).toContainText("Posts");
  });

  test("displays seeded blog post", async ({ page }) => {
    await page.goto("/posts");
    // Wait for the post to load (suspense resolves)
    const postLink = page.locator('a[href="/posts/welcome-to-my-blog"]');
    await expect(postLink).toBeVisible({ timeout: 10_000 });
    await expect(postLink).toContainText("Welcome to My Blog");
  });

  test("blog post shows metadata", async ({ page }) => {
    await page.goto("/posts");
    const postLink = page.locator('a[href="/posts/welcome-to-my-blog"]');
    await expect(postLink).toBeVisible({ timeout: 10_000 });
    // Should show reading time
    await expect(postLink).toContainText("min read");
    // Should show tags
    await expect(postLink).toContainText("meta");
  });

  test("blog post detail page renders", async ({ page }) => {
    await page.goto("/posts/welcome-to-my-blog");
    await expect(page).toHaveURL(/\/posts\/welcome-to-my-blog/);
    await expect(page.locator("h1").first()).toContainText("Welcome to My Blog");
  });
});

test.describe("/projects page", () => {
  test("loads without errors", async ({ page }) => {
    await page.goto("/projects");
    // Should not show error text
    await expect(page.locator("text=Error loading projects")).not.toBeVisible();
  });

  test("displays page heading", async ({ page }) => {
    await page.goto("/projects");
    await expect(page.locator("h1")).toContainText("Projects");
  });

  test("displays seeded portfolio item", async ({ page }) => {
    await page.goto("/projects");
    const projectLink = page.locator('a[href="/projects/sample-project"]');
    await expect(projectLink).toBeVisible({ timeout: 10_000 });
    await expect(projectLink).toContainText("Sample Project");
  });

  test("portfolio item shows tech stack", async ({ page }) => {
    await page.goto("/projects");
    const projectLink = page.locator('a[href="/projects/sample-project"]');
    await expect(projectLink).toBeVisible({ timeout: 10_000 });
    // Should show tech stack badges
    await expect(projectLink).toContainText("Rust");
    await expect(projectLink).toContainText("Leptos");
    await expect(projectLink).toContainText("SurrealDB");
  });

  test("portfolio item shows description", async ({ page }) => {
    await page.goto("/projects");
    const projectLink = page.locator('a[href="/projects/sample-project"]');
    await expect(projectLink).toBeVisible({ timeout: 10_000 });
    await expect(projectLink).toContainText(
      "A sample portfolio project to demonstrate the system",
    );
  });

  test("project detail page renders", async ({ page }) => {
    await page.goto("/projects/sample-project");
    await expect(page).toHaveURL(/\/projects\/sample-project/);
    await expect(page.locator("h1").first()).toContainText("Sample Project");
  });
});

test.describe("home page", () => {
  test("loads without errors", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Error")).not.toBeVisible();
  });

  test("shows recent posts section", async ({ page }) => {
    await page.goto("/");
    // The home page shows recent posts
    await expect(
      page.locator("text=Welcome to My Blog").first(),
    ).toBeVisible({ timeout: 10_000 });
  });

  test("shows projects section", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Sample Project").first()).toBeVisible({
      timeout: 10_000,
    });
  });
});
