import { devNull } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { processFailed, runProcessSync } from "../tools/foundation/src/process.ts";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
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
const CARGO_WORKSPACE_MANIFESTS = [
  "Cargo.toml",
  "subrepos/cli-config-resolution/Cargo.toml"
] as const;

type Action = "check" | "setup";

export function submoduleStatusFailures(output: string): string[] {
  return output
    .split(/\r?\n/u)
    .filter((line) => line.length > 0 && !line.startsWith(" "));
}

function setupEnvironment(): void {
  runMise(["trust", "mise.toml"]);
  run("git", ["submodule", "update", "--init", "--recursive"]);
  runMise(["install", "--locked", ...MISE_TOOLS]);
  runInMise("pnpm", ["install", "--frozen-lockfile"]);

  for (const manifestPath of CARGO_WORKSPACE_MANIFESTS) {
    runInMise("cargo", ["fetch", "--locked", "--manifest-path", manifestPath]);
  }

  runInMise("codegraph", ["init", "."]);
  runInMise("codegraph", ["sync", "--quiet", "."]);
}

function checkEnvironment(): void {
  runMise(["install", "--dry-run-code", "--locked", ...MISE_TOOLS]);
  checkWorkspaces();
  runMise(["ls", "--current", ...MISE_TOOLS]);
  runInMise("lizard", ["--version"]);
  runInMise("scc", ["--version"]);
  runInMise("bun", ["run", "openspec", "--version"]);
  runInMise("bun", ["run", "jscpd", "--version"]);
  runInMise("codegraph", ["--version"]);
  runInMise("codegraph", ["status", "."]);
}

function checkWorkspaces(): void {
  const submoduleOutput = run("git", ["submodule", "status", "--recursive"], true);
  const submoduleLines = submoduleOutput.split(/\r?\n/u).filter((line) => line.length > 0);
  if (submoduleLines.length === 0) {
    throw new Error("git submodule status returned no entries");
  }

  const submoduleFailures = submoduleStatusFailures(submoduleOutput);
  if (submoduleFailures.length > 0) {
    throw new Error(
      `submodules are unavailable or not at the pinned revision:\n${submoduleFailures.join("\n")}\nRun bun run env:setup.`
    );
  }
  console.log(`submodule check ok: ${submoduleLines.length} pinned checkouts`);

  for (const manifestPath of CARGO_WORKSPACE_MANIFESTS) {
    const metadata = JSON.parse(runInMise("cargo", [
      "metadata",
      "--locked",
      "--offline",
      "--format-version",
      "1",
      "--manifest-path",
      manifestPath
    ], true)) as unknown;
    const workspaceMembers = workspaceMemberCount(metadata, manifestPath);
    console.log(`cargo workspace ok: ${manifestPath} (${workspaceMembers} members)`);
  }
}

function workspaceMemberCount(metadata: unknown, manifestPath: string): number {
  if (
    typeof metadata !== "object"
    || metadata === null
    || !("workspace_members" in metadata)
    || !Array.isArray(metadata.workspace_members)
    || metadata.workspace_members.length === 0
  ) {
    throw new Error(`cargo metadata returned no workspace members for ${manifestPath}`);
  }
  return metadata.workspace_members.length;
}

function runInMise(command: string, args: string[], capture = false): string {
  return runMise(["exec", "--", command, ...args], capture);
}

function runMise(args: string[], capture = false): string {
  return run("mise", args, capture, MISE_ENV);
}

function run(
  command: string,
  args: string[],
  capture = false,
  env: NodeJS.ProcessEnv = process.env
): string {
  const result = runProcessSync(command, args, {
    cwd: REPO_ROOT,
    env,
    stdio: capture ? "pipe" : "inherit"
  });
  if (processFailed(result)) {
    const diagnostic = result.stderr.trim() || result.stdout.trim() || `exit ${result.status ?? "unknown"}`;
    throw new Error(`${command} ${args.join(" ")} failed: ${diagnostic}`);
  }
  return result.stdout;
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
