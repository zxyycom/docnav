import { spawnSync } from "node:child_process";
import { devNull } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const PROCESS_MAX_BUFFER = 64 * 1024 * 1024;
const PLAIN_TEXT_ENV = {
  CARGO_TERM_COLOR: "never",
  CLICOLOR: "0",
  CLICOLOR_FORCE: "0",
  FORCE_COLOR: "0",
  NO_COLOR: "1",
  PNPM_CONFIG_COLOR: "false",
  PY_COLORS: "0",
  TERM: "dumb",
  UV_NO_COLOR: "1",
  npm_config_color: "false"
} satisfies NodeJS.ProcessEnv;
const MISE_ENV = {
  ...process.env,
  MISE_GLOBAL_CONFIG_FILE: devNull
};
const MISE_TOOLS = [
  "node",
  "bun",
  "pnpm",
  "uv",
  "go",
  "rust",
  "pipx:lizard",
  "go:github.com/boyter/scc/v3",
  "npm:@colbymchenry/codegraph"
] as const;

type Action = "check" | "setup";

function setupEnvironment(): void {
  runMise(["trust", "mise.toml"]);
  runMise(["install", "--locked", ...MISE_TOOLS]);
  runInMise("pnpm", ["install", "--frozen-lockfile"]);
  runInMise("cargo", ["fetch", "--locked"]);

  runInMise("codegraph", ["init", "."]);
  runInMise("codegraph", ["sync", "--quiet", "."]);
}

function checkEnvironment(): void {
  runMise(["install", "--dry-run-code", "--locked", ...MISE_TOOLS]);
  checkWorkspace();
  runMise(["ls", "--current", ...MISE_TOOLS]);
  runInMise("lizard", ["--version"]);
  runInMise("scc", ["--version"]);
  runInMise("bun", ["run", "openspec", "--version"]);
  runInMise("bun", ["run", "jscpd", "--version"]);
  runInMise("codegraph", ["--version"]);
  runInMise("codegraph", ["status", "."]);
}

function checkWorkspace(): void {
  const metadata = JSON.parse(runInMise("cargo", [
    "metadata",
    "--locked",
    "--offline",
    "--format-version",
    "1"
  ], true)) as unknown;
  const workspaceMembers = workspaceMemberCount(metadata);
  console.log(`cargo workspace ok: Cargo.toml (${workspaceMembers} members)`);
}

function workspaceMemberCount(metadata: unknown): number {
  if (
    typeof metadata !== "object"
    || metadata === null
    || !("workspace_members" in metadata)
    || !Array.isArray(metadata.workspace_members)
    || metadata.workspace_members.length === 0
  ) {
    throw new Error("cargo metadata returned no workspace members for Cargo.toml");
  }
  return metadata.workspace_members.length;
}

function runInMise(command: string, args: string[], capture = false): string {
  return runMise(["exec", "--", command, ...args], capture);
}

function runMise(args: string[], capture = false): string {
  const result = spawnSync("mise", args, {
    cwd: REPO_ROOT,
    encoding: "utf8",
    env: { ...MISE_ENV, ...PLAIN_TEXT_ENV },
    maxBuffer: PROCESS_MAX_BUFFER,
    stdio: capture ? "pipe" : "inherit",
    windowsHide: true
  });
  const stderr = typeof result.stderr === "string" ? result.stderr : "";
  const stdout = typeof result.stdout === "string" ? result.stdout : "";
  if (result.error || result.status !== 0) {
    const diagnostic = stderr.trim()
      || stdout.trim()
      || result.error?.message
      || `exit ${result.status ?? "unknown"}`;
    throw new Error(`mise ${args.join(" ")} failed: ${diagnostic}`);
  }
  return stdout;
}

function parseAction(value: string | undefined): Action {
  if (value === "check" || value === "setup") {
    return value;
  }
  throw new Error("usage: bun scripts/project-environment/index.ts <check|setup>");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const action = parseAction(process.argv[2]);
    if (action === "setup") {
      setupEnvironment();
    } else {
      checkEnvironment();
    }
  } catch (error) {
    console.error(`project environment failed: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  }
}
