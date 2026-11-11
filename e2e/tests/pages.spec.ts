import {
  test,
  expect,
  type APIRequestContext,
  type Browser,
  type Page,
} from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

type NavItem = { label: string; path: string };
type SiteConfig = {
  name: string;
  tagline: string;
  default_theme: "dark" | "light";
  nav: NavItem[];
  pages: {
    blog: { title: string; subtitle: string };
    portfolio: { title: string; subtitle: string };
    about: { title: string };
  };
};
type BlogListItem = {
  slug: string;
  title: string;
  description: string;
  author: string;
  reading_time_minutes: number;
  tags: string[];
};
type PortfolioItem = {
  slug: string;
  title: string;
  tech_stack: string[];
  featured?: boolean;
};

let site: SiteConfig;
let posts: BlogListItem[];
let projects: PortfolioItem[];

test.beforeAll(async ({ request }) => {
  site = await (await request.get("/api/config")).json();
  posts = await (await request.get("/api/posts")).json();
  projects = await (await request.get("/api/portfolio")).json();
});

async function openPage(
  browser: Browser,
  baseURL: string,
  options: { width?: number; height?: number; theme?: "dark" | "light" } = {},
): Promise<{ page: Page; close: () => Promise<void> }> {
  const context = await browser.newContext({
    baseURL,
    viewport: { width: options.width ?? 1440, height: options.height ?? 900 },
  });
  // Only seed the preference; a real choice stored by the toggle must survive
  // reloads.
  await context.addInitScript((theme) => {
    if (!localStorage.getItem("theme")) localStorage.setItem("theme", theme);
  }, options.theme ?? "dark");
  const page = await context.newPage();
  return { page, close: () => context.close() };
}

async function waitForHydration(page: Page) {
  await page.waitForFunction(
    () => typeof (window as { hydration_callback?: unknown }).hydration_callback === "function",
  );
}

async function settle(page: Page, path: string) {
  await page.goto(path, { waitUntil: "networkidle" });
  await waitForHydration(page);
  await page.waitForTimeout(400);
}

type AxeViolation = { id: string; impact?: string | null; help: string };

function seriousViolations(results: { violations: AxeViolation[] }) {
  return results.violations.filter(
    (violation) => violation.impact === "serious" || violation.impact === "critical",
  );
}

test.describe("stylesheet delivery", () => {
  test("every stylesheet resolves and is not a dx placeholder", async ({
    page,
    request,
  }) => {
    await page.goto("/");
    const hrefs = await page
      .locator('link[rel="stylesheet"]')
      .evaluateAll((links) => links.map((link) => link.getAttribute("href") ?? ""));
    expect(hrefs.length).toBeGreaterThan(0);
    for (const href of hrefs) {
      expect(href).not.toContain("replaced by dx");
      const response = await request.get(href);
      expect(response.status(), `${href} status`).toBe(200);
      expect(response.headers()["content-type"], `${href} content type`).toContain(
        "text/css",
      );
    }
  });
});

test.describe("site shell", () => {
  test("header and footer use the configured site identity", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('header a[href="/"]')).toContainText(site.name);
    await expect(page.locator("footer")).toContainText(site.name);

    const desktopNav = page.locator('header nav[aria-label="Primary"]').first();
    const links = desktopNav.locator("a");
    await expect(links).toHaveCount(site.nav.length);
    for (const [index, item] of site.nav.entries()) {
      await expect(links.nth(index)).toHaveText(item.label);
      await expect(links.nth(index)).toHaveAttribute("href", item.path);
    }
  });

  test("current navigation entry is marked, others carry no aria-current", async ({
    page,
  }) => {
    await page.goto("/posts");
    const desktopNav = page.locator('header nav[aria-label="Primary"]').first();
    const current = desktopNav.locator('a[aria-current="page"]:visible');
    await expect(current).toHaveCount(1);
    await expect(current).toContainText(
      site.nav.find((item) => item.path === "/posts")?.label ?? "Posts",
    );
    await expect(desktopNav.locator("a:not([aria-current])")).toHaveCount(
      site.nav.filter((item) => item.path !== "/posts").length,
    );
  });

  test("skip link is the first focusable element and targets the main region", async ({
    page,
  }) => {
    await page.goto("/");
    await page.keyboard.press("Tab");
    await expect(page.locator('a[href="#content"]')).toBeFocused();
    await page.keyboard.press("Enter");
    await expect(page).toHaveURL(/#content$/);
    await expect(page.locator("main#content")).toHaveCount(1);
    await expect(page.locator("#main main#content")).toHaveCount(1);
  });
});

test.describe("mobile navigation", () => {
  test.use({ viewport: { width: 390, height: 844 } });

  test("desktop row is hidden and the menu lists the same configured labels", async ({
    page,
  }) => {
    await page.goto("/");
    const labels = site.nav.map((item) => item.label);
    await expect(page.locator('header nav[aria-label="Primary"] a:visible')).toHaveCount(0);

    const summary = page.locator('summary[aria-label="Menu"]');
    await expect(summary).toBeVisible();
    await summary.click();

    const panel = page.locator('header nav[aria-label="Primary"]').nth(1);
    await expect(panel.locator("a")).toHaveText(labels);
    for (const label of labels) {
      await expect(panel.locator("a", { hasText: label })).toBeVisible();
    }
  });

  test("choosing a menu link navigates with the menu closed", async ({ page }) => {
    await page.goto("/");
    await page.locator('summary[aria-label="Menu"]').click();
    const target = site.nav.find((item) => item.path !== "/") ?? site.nav[0];
    const panel = page.locator('header nav[aria-label="Primary"]').nth(1);
    await panel.locator("a", { hasText: target.label }).click();
    await expect(page).toHaveURL((url) => url.pathname === target.path);
    await expect(page.locator("details[open]")).toHaveCount(0);
    await expect(page.locator("main h1").first()).toBeVisible();
  });

  test("menu opens without JavaScript", async ({ browser }) => {
    const context = await browser.newContext({
      baseURL: test.info().project.use.baseURL,
      viewport: { width: 390, height: 844 },
      javaScriptEnabled: false,
    });
    const page = await context.newPage();
    await page.goto("/");

    const labels = site.nav.map((item) => item.label);
    const panel = page.locator('header nav[aria-label="Primary"]').nth(1);
    await expect(panel.locator("a")).toHaveText(labels);
    await expect(panel.locator("a").first()).toBeHidden();

    await page.locator('summary[aria-label="Menu"]').click();
    await expect(panel.locator("a").first()).toBeVisible();
    for (const label of labels) {
      await expect(panel.locator("a", { hasText: label })).toBeVisible();
    }
    await context.close();
  });
});

test.describe("theme toggle", () => {
  test("flips html.dark, stores the choice and restores it after reload", async ({
    page,
  }) => {
    await settle(page, "/");
    const isDark = () =>
      page.locator("html").evaluate((el) => el.classList.contains("dark"));
    const before = await isDark();

    await page
      .getByRole("button", {
        name: before ? "Switch to light theme" : "Switch to dark theme",
      })
      .click();
    expect(await isDark()).toBe(!before);
    expect(await page.evaluate(() => localStorage.getItem("theme"))).toBe(
      before ? "light" : "dark",
    );

    await page.reload({ waitUntil: "networkidle" });
    expect(await isDark()).toBe(!before);
    await expect(
      page.getByRole("button", {
        name: !before ? "Switch to light theme" : "Switch to dark theme",
      }),
    ).toBeVisible();
  });
});

test.describe("home page", () => {
  test("hero is the configured identity", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("main h1").first()).toHaveText(site.name);
    await expect(page.locator("main")).toContainText(site.tagline);
  });

  test("shows recent posts and featured projects", async ({ page }) => {
    await page.goto("/");
    const recent = page.locator("main article").first();
    await expect(recent).toContainText(posts[0].title, { timeout: 10_000 });
    await expect(recent).toContainText("min read");
    await expect(page.locator("main a", { hasText: posts[0].title })).toHaveAttribute(
      "href",
      `/posts/${posts[0].slug}`,
    );

    const featured = projects.filter((project) => project.featured);
    const shown = featured.length > 0 ? featured[0] : projects[0];
    await expect(page.locator("main")).toContainText(shown.title);
  });
});

test.describe("/posts page", () => {
  test("loads without error and uses the configured heading", async ({ page }) => {
    await page.goto("/posts");
    await expect(page.locator("text=Error")).not.toBeVisible();
    await expect(page.locator("main h1")).toHaveText(site.pages.blog.title);
  });

  test("lists posts with title, description, date, reading time and tags", async ({
    page,
  }) => {
    await page.goto("/posts");
    const post = posts[0];
    const row = page.locator("main article").first();
    await expect(row).toContainText(post.title, { timeout: 10_000 });
    await expect(row).toContainText(post.description);
    await expect(row.locator("time")).toHaveAttribute("datetime", /^\d{4}-\d{2}-\d{2}T/);
    await expect(row).toContainText("min read");
    if (post.tags.length > 0) {
      await expect(row.locator(`a[href="/posts/tag/${post.tags[0]}"]`)).toBeVisible();
    }
  });

  test("post detail page renders metadata, prose and tag links", async ({ page }) => {
    const post = posts[0];
    await page.goto(`/posts/${post.slug}`);
    await expect(page).toHaveURL((url) => url.pathname === `/posts/${post.slug}`);
    await expect(page.locator("main h1").first()).toHaveText(post.title);
    const article = page.locator("main article");
    await expect(article).toContainText(post.author);
    await expect(article).toContainText("min read");
    await expect(article.locator("time")).toBeVisible();
    if (post.tags.length > 0) {
      await expect(article.locator(`a[href="/posts/tag/${post.tags[0]}"]`)).toBeVisible();
    }
  });
});

test.describe("/projects page", () => {
  test("loads without error and uses the configured heading", async ({ page }) => {
    await page.goto("/projects");
    await expect(page.locator("text=Error loading projects")).not.toBeVisible();
    await expect(page.locator("main h1")).toHaveText(site.pages.portfolio.title);
  });

  test("lists portfolio items with tech stack chips", async ({ page }) => {
    await page.goto("/projects");
    const project = projects[0];
    const card = page.locator("main a", { hasText: project.title }).first();
    await expect(card).toBeVisible({ timeout: 10_000 });
    for (const tech of project.tech_stack) {
      await expect(card).toContainText(tech);
    }
  });

  test("project detail page renders", async ({ page }) => {
    const project = projects[0];
    await page.goto(`/projects/${project.slug}`);
    await expect(page.locator("main h1").first()).toHaveText(project.title);
  });
});

test.describe("responsive overflow", () => {
  test.use({ viewport: { width: 320, height: 700 } });

  test("no horizontal scrolling at 320px", async ({ page }) => {
    const longest = [...posts].sort((a, b) => b.title.length - a.title.length)[0];
    for (const path of ["/", "/posts", `/posts/${longest.slug}`]) {
      await page.goto(path);
      await page.waitForTimeout(300);
      const metrics = await page.evaluate(() => ({
        doc: document.documentElement.scrollWidth,
        body: document.body.scrollWidth,
        viewport: window.innerWidth,
      }));
      expect(metrics.doc, `${path} document scrollWidth`).toBeLessThanOrEqual(
        metrics.viewport + 1,
      );
      expect(metrics.body, `${path} body scrollWidth`).toBeLessThanOrEqual(
        metrics.viewport + 1,
      );
    }
  });
});

const axeTargets = [
  { name: "home", resolve: () => "/" },
  { name: "posts", resolve: () => "/posts" },
  {
    name: "post detail",
    resolve: async (request: APIRequestContext) => {
      const list: BlogListItem[] = await (await request.get("/api/posts")).json();
      return `/posts/${list[0].slug}`;
    },
  },
  { name: "about", resolve: () => "/about" },
];

for (const theme of ["dark", "light"] as const) {
  test.describe(`accessibility (${theme})`, () => {
    for (const target of axeTargets) {
      test(`axe has no serious or critical violations on ${target.name}`, async ({
        browser,
        request,
      }) => {
        const path = await target.resolve(request);
        const { page, close } = await openPage(
          browser,
          test.info().project.use.baseURL ?? "http://localhost:3000",
          { theme },
        );
        await settle(page, path);
        const results = await new AxeBuilder({ page })
          .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
          .analyze();
        expect(seriousViolations(results).map((v) => `${v.id}: ${v.help}`)).toEqual([]);
        await close();
      });
    }
  });
}

test.describe("client-side navigation", () => {
  test("content links keep working after hydration", async ({ page }) => {
    const failed: string[] = [];
    page.on("response", (r) => {
      if (r.status() >= 500) failed.push(`${r.status()} ${r.url()}`);
    });
    await page.goto("/posts");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    const title = page.locator("main a[href^='/posts/']:not([href^='/posts/tag/'])").first();
    const text = (await title.innerText()).trim();
    await title.click();
    await expect(page.locator("main h1")).toHaveText(text);
    await expect(page.locator("header")).toBeVisible();
    await page.locator("main a[href^='/posts/tag/']").first().click();
    await expect(page.locator("main h1")).toContainText("#");
    await page.goto("/projects");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    const project = page.locator("main a[href^='/projects/']").first();
    const projectTitle = (await project.locator("h2, h3").first().innerText()).trim();
    await project.click();
    await expect(page).toHaveURL(/\/projects\/.+/);
    await expect(page.locator("main h1")).toHaveText(projectTitle);
    await expect(page.locator("header")).toBeVisible();
    expect(failed).toEqual([]);
  });

  test("Back restores the scroll position and navigation moves focus", async ({ page }) => {
    await page.goto("/posts");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    const titles = page.locator("main a[href^='/posts/']:not([href^='/posts/tag/'])");
    const target = titles.nth((await titles.count()) - 1);
    await target.scrollIntoViewIfNeeded();
    const before = await page.evaluate(() => window.scrollY);
    expect(before).toBeGreaterThan(200);
    await page.evaluate(() => ((window as any).__sameDocument = true));
    await target.click();
    await expect(page).toHaveURL(/\/posts\/.+/);
    await expect(page.locator("main")).toBeFocused();
    // Header navigation stays in the same document too.
    await page.locator('header nav[aria-label="Primary"] a:visible').first().click();
    await expect(page.locator("main")).toBeFocused();
    expect(await page.evaluate(() => (window as any).__sameDocument)).toBe(true);
    await page.goBack();
    await expect(page).toHaveURL(/\/posts\/.+/);
    await page.goBack();
    await expect(page.locator("main h1")).toBeVisible();
    await expect
      .poll(() => page.evaluate(() => window.scrollY))
      .toBeGreaterThan(before - 50);
  });

  test("a failed loader shows a recoverable page, on a clean endpoint path", async ({ page }) => {
    const calls: string[] = [];
    await page.goto("/");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    await page.route("**/api/**", (route) => {
      calls.push(new URL(route.request().url()).pathname);
      return route.abort();
    });
    await page.locator('header nav[aria-label="Primary"] a:visible').first().click();
    await expect(page.getByRole("alert")).toContainText("could not be loaded");
    expect(calls.length).toBeGreaterThan(0);
    expect(calls.filter((path) => path.includes("//"))).toEqual([]);
    await page.unroute("**/api/**");
    await page.getByRole("link", { name: "Try again" }).click();
    await expect(page.locator("main h1")).toBeVisible();
    await expect(page.getByRole("banner")).toBeVisible();
  });

  test("a stalled loader ends in a full reload, not a blank page", async ({ page }) => {
    await page.goto("/");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    await page.evaluate(() => ((window as any).__sameDocument = true));
    let hold = true;
    await page.route("**/api/**", (route) => (hold ? undefined : route.continue()));
    await page.locator('header nav[aria-label="Primary"] a:visible').first().click();
    await page.waitForFunction(() => !(window as any).__sameDocument, undefined, { timeout: 15000 });
    hold = false;
    await expect(page.getByRole("banner")).toBeVisible();
    await expect(page.locator("main h1")).toBeVisible();
  });

  test("the header stays while the next page's data loads", async ({ page }) => {
    await page.goto("/");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    let release!: () => void;
    const released = new Promise<void>((resolve) => (release = resolve));
    await page.route("**/api/**", async (route) => {
      await released;
      await route.continue();
    });
    await page.locator('header nav[aria-label="Primary"] a:visible').first().click();
    await expect(page.locator('main[aria-busy="true"]')).toBeVisible();
    await expect(page.getByRole("banner")).toBeVisible();
    await expect(page.getByRole("contentinfo")).toBeVisible();
    release();
    await expect(page.locator("main h1")).toBeVisible();
  });

  test("navigation keeps focus, link previews and canonical URL current", async ({ page }) => {
    await page.goto("/posts");
    await page.waitForFunction(() => typeof (window as any).hydration_callback === "function");
    // The first page keeps the browser's own focus start.
    expect(await page.evaluate(() => document.activeElement?.tagName)).toBe("BODY");
    const meta = (selector: string, attribute = "content") =>
      page.locator(selector).first().getAttribute(attribute);
    await page.locator("main a[href^='/posts/']:not([href^='/posts/tag/'])").first().click();
    await expect(page).toHaveURL(/\/posts\/.+/);
    const lede = (await page.locator("main article > p").nth(1).innerText()).trim();
    await expect.poll(() => meta('meta[name="description"]')).toBe(lede);
    await expect.poll(() => meta('meta[property="og:title"]')).toBe(await page.title());
    const path = new URL(page.url()).pathname;
    await expect.poll(() => meta('link[rel="canonical"]', "href")).toMatch(new RegExp(`${path}$`));
    await page.getByRole("banner").getByRole("link").first().click();
    await expect(page).toHaveURL(/\/$/);
    await expect(page.locator("main")).toBeFocused();
    await expect.poll(() => meta('link[rel="canonical"]', "href")).toMatch(/\/$/);
  });

  test("a post links to the posts before and after it", async ({ page }) => {
    await page.goto("/posts");
    const links = page.locator("main a[href^='/posts/']:not([href^='/posts/tag/'])");
    const titles = await links.allInnerTexts();
    expect(titles.length).toBeGreaterThan(2);
    await page.goto((await links.nth(1).getAttribute("href"))!);
    const more = page.getByRole("navigation", { name: "More posts" });
    await expect(more.getByRole("link", { name: /Previous/ })).toContainText(titles[2].trim());
    await expect(more.getByRole("link", { name: /Next/ })).toContainText(titles[0].trim());
  });

  test("without JavaScript the theme toggle is hidden", async ({ browser }) => {
    const context = await browser.newContext({ javaScriptEnabled: false });
    const page = await context.newPage();
    await page.goto(new URL("/", test.info().project.use.baseURL).toString());
    await expect(page.locator("[data-theme-toggle]")).toBeHidden();
    await expect(page.locator("main h1")).toBeVisible();
    await context.close();
  });

  test("theme toggle works before WebAssembly loads", async ({ page }) => {
    await page.route("**/*.wasm", (route) => route.abort());
    await page.goto("/");
    const dark = await page.evaluate(() => document.documentElement.classList.contains("dark"));
    await page.locator("[data-theme-toggle]").click();
    expect(await page.evaluate(() => document.documentElement.classList.contains("dark"))).toBe(
      !dark,
    );
  });
});
