import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const CHANGE_PLAN_CLI = resolve(
  REPO_ROOT,
  ".codex/skills/change-plan/scripts/change-plan.mjs"
);

type CatalogEntry = {
  changeName: string;
  stage: "draft" | "implementation" | "plan" | "shelved" | null;
  status: "active" | "archived";
  valid: boolean;
};

type Catalog = {
  entries: CatalogEntry[];
  errors: string[];
};

const result = spawnSync(
  process.execPath,
  [CHANGE_PLAN_CLI, "list", "changes", "--all", "--json"],
  {
    cwd: REPO_ROOT,
    encoding: "utf8",
    env: process.env,
    maxBuffer: 16 * 1024 * 1024,
    windowsHide: true
  }
);

if (result.error || result.status !== 0) {
  const diagnostic = result.stderr.trim()
    || result.stdout.trim()
    || result.error?.message
    || `exit ${result.status ?? "unknown"}`;
  throw new Error(`change-plan catalog failed: ${diagnostic}`);
}

const catalog = parseCatalog(result.stdout);
if (catalog.errors.length > 0) {
  throw new Error(`change-plan catalog errors: ${catalog.errors.join("; ")}`);
}

const invalid = catalog.entries.filter((entry) => !entry.valid);
if (invalid.length > 0) {
  throw new Error(
    `invalid change plans: ${invalid.map((entry) => entry.changeName).join(", ")}`
  );
}

const active = catalog.entries.filter((entry) => entry.status === "active").length;
const archived = catalog.entries.length - active;
const stages = ["draft", "plan", "implementation", "shelved"] as const;
const stageSummary = stages
  .map((stage) => `${stage}=${catalog.entries.filter((entry) => entry.stage === stage).length}`)
  .join(", ");

console.log(
  `Change plans check passed (${active} active, ${archived} archived; ${stageSummary}).`
);

function parseCatalog(text: string): Catalog {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch (error) {
    throw new Error(
      `change-plan catalog returned invalid JSON: ${error instanceof Error ? error.message : String(error)}`,
      { cause: error }
    );
  }

  if (!isRecord(value) || !Array.isArray(value.entries) || !isStringArray(value.errors)) {
    throw new Error("change-plan catalog returned an unexpected result shape");
  }

  const entries = value.entries.map((entry, index) => parseEntry(entry, index));
  return { entries, errors: value.errors };
}

function parseEntry(value: unknown, index: number): CatalogEntry {
  if (
    !isRecord(value)
    || typeof value.changeName !== "string"
    || typeof value.valid !== "boolean"
    || (value.status !== "active" && value.status !== "archived")
    || !isStage(value.stage)
  ) {
    throw new Error(`change-plan catalog entry ${index} has an unexpected result shape`);
  }
  return {
    changeName: value.changeName,
    stage: value.stage,
    status: value.status,
    valid: value.valid
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isStage(value: unknown): value is CatalogEntry["stage"] {
  return value === null
    || value === "draft"
    || value === "plan"
    || value === "implementation"
    || value === "shelved";
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === "string");
}
