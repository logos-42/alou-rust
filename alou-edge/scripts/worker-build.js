import { rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(__dirname, "..");

const customOutDir = process.env.CUSTOM_WORKER_BUILD_OUT_DIR?.trim();
const cleanTarget = resolve(
  projectRoot,
  customOutDir && customOutDir.length > 0 ? customOutDir : "build",
);

const cleanOnly = process.argv.includes("--clean-only");

try {
  rmSync(cleanTarget, { recursive: true, force: true });
  console.log(`[worker-build] cleaned ${cleanTarget}`);
} catch (err) {
  console.error("[worker-build] failed to clean output directory", err);
  process.exit(1);
}

if (cleanOnly) {
  process.exit(0);
}

const args = ["--release"];
if (customOutDir && customOutDir.length > 0) {
  args.push("--out-dir", customOutDir);
}

const child = spawnSync("worker-build", args, {
  cwd: projectRoot,
  stdio: "inherit",
  shell: true,
});

if (child.error) {
  console.error("[worker-build] failed to run worker-build:", child.error);
  process.exit(child.status ?? 1);
}

// 在 worker-build 完成后，应用 WASM 修复
console.log("[worker-build] applying WASM initialization fix...");
const fixChild = spawnSync("node", ["./scripts/fix-wasm-init.js"], {
  cwd: projectRoot,
  stdio: "inherit",
  shell: true,
});

if (fixChild.error) {
  console.error("[worker-build] failed to apply WASM fix:", fixChild.error);
  process.exit(fixChild.status ?? 1);
}

process.exit(child.status ?? 0);

