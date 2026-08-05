#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import http from "node:http";
import net from "node:net";
import process from "node:process";

const target = "plinth/application";
const visualRoot = "target/visual";
const jobPath = `${visualRoot}/capture_job.json`;
const captureOutput = `${visualRoot}/captures`;
const rubricCache = `${visualRoot}/rubric-cache`;
const captureManifest = `${visualRoot}/capture_manifest.json`;
const visualReport = `${visualRoot}/run_report.json`;
const viewports = [
  ["desktop", 1440, 900],
  ["tablet-landscape", 1024, 768],
  ["tablet-portrait", 768, 1024],
  ["mobile", 390, 844],
];
const routes = [
  ["home", "/"],
  ["about", "/about"],
  ["support", "/support"],
  ["posts", "/posts"],
  ["post-detail", "/posts/welcome-to-my-blog"],
  ["projects", "/projects"],
  ["project-detail", "/projects/sample-project"],
];

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
}

function output(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    ...options,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}: ${result.stderr}`);
  }
  return result.stdout.trim();
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close(() => resolve(address.port));
    });
  });
}

function waitForHttp(url, child) {
  return new Promise((resolve, reject) => {
    const deadline = Date.now() + 180_000;
    const check = () => {
      if (Date.now() >= deadline) {
        reject(new Error(`timed out waiting for ${url}`));
        return;
      }
      if (child.exitCode !== null) {
        reject(new Error(`Dioxus server exited with status ${child.exitCode}`));
        return;
      }
      const request = http.get(url, (response) => {
        response.resume();
        if (response.statusCode >= 200 && response.statusCode < 400) {
          resolve();
        } else {
          setTimeout(check, 1000);
        }
      });
      request.setTimeout(1000, () => request.destroy());
      request.on("error", () => setTimeout(check, 1000));
    };
    check();
  });
}

function stopProcess(child) {
  if (child?.pid == null || child.exitCode !== null) return;
  try {
    process.kill(-child.pid, "SIGTERM");
  } catch {
    child.kill("SIGTERM");
  }
}

function captureJob(revision, dirty) {
  const cells = routes.flatMap(([state, path]) =>
    viewports.map(([name, width, height]) => ({
      id: `${state}/${name}/default/en-US`,
      path,
      state,
      viewport: { width, height, dpr: 1 },
      theme: "default",
      locale: "en-US",
      presets: ["ui-regression"],
    })),
  );
  return {
    schema_version: 1,
    target,
    revision,
    dirty,
    cells,
  };
}

async function main() {
  const revision = output("git", ["rev-parse", "HEAD"]);
  const dirty = output("git", ["status", "--porcelain=v1"]).length > 0;
  mkdirSync(visualRoot, { recursive: true });
  writeFileSync(jobPath, `${JSON.stringify(captureJob(revision, dirty), null, 2)}\n`);

  run("./scripts/dev-db.sh", ["start"]);
  const proxyPort = await freePort();
  const appPort = await freePort();
  const appUrl = `http://127.0.0.1:${appPort}`;
  const child = spawn(
    "dx",
    [
      "serve",
      "--web",
      "--fullstack",
      "--package",
      "plinth-web",
      "--bin",
      "plinth-web",
      "--port",
      String(proxyPort),
      "--addr",
      "127.0.0.1",
      "--open",
      "false",
      "--hot-reload",
      "false",
      "--watch",
      "false",
    ],
    {
      env: {
        ...process.env,
        // The interactive wrapper currently probes C preprocessors as Rust.
        // The visual producer only needs a local application build, so keep
        // this producer independent of that cache-probe failure.
        RUSTC_WRAPPER: "",
        SCCACHE_START_SERVER: "0",
        PLINTH_SITE_ADDR: `127.0.0.1:${appPort}`,
      },
      stdio: "inherit",
      detached: true,
    },
  );

  try {
    await waitForHttp(`${appUrl}/`, child);
    run(
      "nix",
      [
        "develop",
        "--no-write-lock-file",
        "../visual-rubric",
        "-c",
        "cargo",
        "run",
        "--locked",
        "--manifest-path",
        "../visual-rubric/Cargo.toml",
        "--features",
        "audit",
        "--bin",
        "visual-rubric",
        "--",
        "capture",
        "--root",
        ".",
        "--base-url",
        appUrl,
        "--job",
        jobPath,
        "--output",
        captureOutput,
        "--manifest",
        captureManifest,
        "--report",
        visualReport,
        "--browser",
        "chromium",
        "--rubric-workers",
        "4",
        "--cache-dir",
        rubricCache,
        "--preset",
        "ui-regression",
      ],
    );
  } finally {
    stopProcess(child);
  }

  const report = JSON.parse(readFileSync(visualReport, "utf8"));
  if (report.target !== target || report.git?.sha !== revision || report.git?.dirty !== dirty) {
    throw new Error("visual-rubric report provenance does not match the producer job");
  }
  console.log(
    `Plinth application visual producer wrote ${report.summary.total_cells} cells to ${visualReport}`,
  );
}

main().catch((error) => {
  console.error(`Plinth application visual producer failed: ${error.message}`);
  process.exitCode = 1;
});
