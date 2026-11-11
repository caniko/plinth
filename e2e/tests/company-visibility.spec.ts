import { test, expect } from "@playwright/test";
import { setTimeout as delay } from "node:timers/promises";

type Response = {
  id: number;
  result?: { result?: { value?: unknown }; exceptionDetails?: unknown };
  error?: { message: string };
};

// Playwright's page attachment forces focus emulation and keeps background tabs
// "visible". Use an isolated raw CDP context in headed Chromium instead.
test("real background tabs suspend motion and preserve explicit pause", async ({ browser }) => {
  const root = await browser.newBrowserCDPSession();
  const { browserContextId } = await root.send("Target.createBrowserContext");
  let sequence = 0;
  const pending = new Map<number, (response: Response) => void>();
  root.on("Target.receivedMessageFromTarget", ({ message }) => {
    const response: Response = JSON.parse(message);
    pending.get(response.id)?.(response);
    pending.delete(response.id);
  });
  try {
    const { targetId } = await root.send("Target.createTarget", { url: "http://127.0.0.1:1122/", browserContextId });
    const { sessionId } = await root.send("Target.attachToTarget", { targetId, flatten: false });
    async function request(method: string, params: Record<string, unknown> = {}) {
      const id = ++sequence;
      const response = new Promise<Response>(resolve => pending.set(id, resolve));
      await root.send("Target.sendMessageToTarget", { sessionId, message: JSON.stringify({ id, method, params }) });
      const result = await response;
      if (result.error) throw new Error(result.error.message);
      if (result.result?.exceptionDetails) throw new Error(JSON.stringify(result.result.exceptionDetails));
      return result.result;
    }
    async function evaluate<T>(expression: string): Promise<T> {
      return (await request("Runtime.evaluate", { expression, returnByValue: true }))?.result?.value as T;
    }
    // Raw CDP tabs have no fixed viewport: activating a tab can resize its
    // window, and a resize between the press and the release moves the
    // toggle out from under the pointer. Pin the viewport like the project's.
    await request("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
    const state = () => evaluate<string>("document.body?.dataset.motionState");
    await root.send("Target.activateTarget", { targetId });
    await expect.poll(state).toBe("playing");
    const { targetId: foreground } = await root.send("Target.createTarget", { url: "about:blank", browserContextId });
    await root.send("Target.activateTarget", { targetId: foreground });
    await expect.poll(() => evaluate<boolean>("document.hidden")).toBe(true);
    await expect.poll(state).toBe("hidden");
    await expect.poll(() => evaluate<string>("getComputedStyle(document.querySelector('.company-motion-layer')).animationPlayState")).toBe("paused");
    // Let the compositor commit the pause, then sample the real animation clock.
    await delay(100);
    const time = await evaluate<number>("document.getAnimations()[0].currentTime");
    await delay(150);
    expect(await evaluate<number>("document.getAnimations()[0].currentTime")).toBe(time);
    await root.send("Target.activateTarget", { targetId });
    await expect.poll(() => evaluate<boolean>("document.hidden")).toBe(false);
    await expect.poll(state).toBe("playing");
    const point = await evaluate<{ x: number; y: number }>("(() => { const r = document.querySelector('.company-motion-toggle').getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()");
    await request("Input.dispatchMouseEvent", { type: "mousePressed", ...point, button: "left", clickCount: 1 });
    await request("Input.dispatchMouseEvent", { type: "mouseReleased", ...point, button: "left", clickCount: 1 });
    await expect.poll(state).toBe("paused");
    await root.send("Target.activateTarget", { targetId: foreground });
    await expect.poll(() => evaluate<boolean>("document.hidden")).toBe(true);
    await root.send("Target.activateTarget", { targetId });
    await expect.poll(() => evaluate<boolean>("document.hidden")).toBe(false);
    await expect.poll(state).toBe("paused");
    expect(await evaluate<string>("document.querySelector('.company-motion-toggle').getAttribute('aria-pressed')")).toBe("true");
  } finally {
    await root.send("Target.disposeBrowserContext", { browserContextId });
    await root.detach();
  }
});
