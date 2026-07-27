import fs from "node:fs";
import path from "node:path";

import {
  runTestEvidenceCatalogCli,
  syncTestEvidenceIndex,
  validateTestEvidence
} from "../../.codex/skills/test-evidence-review/scripts/test-evidence-catalog.mjs";
import { compareInventoryBaseline } from "./change-report.ts";
import { discoverNativeTestEntries } from "./discover.ts";
import {
  compareNativeTestInventory,
  createNativeTestInventory,
  inventoryPath,
  readCommittedInventory,
  writeNativeTestInventory
} from "./inventory.ts";
import {
  diagnostic,
  type NativeTestInventory,
  type TestEvidenceDiagnostic
} from "./model.ts";

export async function checkTestEvidence(options: {
  workspaceRoot: string;
}): Promise<ProjectTestEvidenceReport> {
  const discovery = await discoverNativeTestEntries(options);
  const diagnostics = [...discovery.diagnostics];
  const expected = createNativeTestInventory(discovery);
  if (!diagnostics.some(({ blocking }) => blocking)) {
    const actual = readCommittedInventory(options.workspaceRoot);
    diagnostics.push(...compareNativeTestInventory({
      expected,
      actual,
      sourcePath: toRelativePath(
        options.workspaceRoot,
        inventoryPath(options.workspaceRoot)
      )
    }));
  }

  const catalog = validateTestEvidence(options);
  diagnostics.push(...catalog.diagnostics as TestEvidenceDiagnostic[]);
  return projectReport(expected, catalog.summary, diagnostics);
}

export async function syncProjectTestEvidence(options: {
  workspaceRoot: string;
}): Promise<ProjectTestEvidenceReport> {
  const discovery = await discoverNativeTestEntries(options);
  const inventory = createNativeTestInventory(discovery);
  if (discovery.diagnostics.some(({ blocking }) => blocking)) {
    return projectReport(
      inventory,
      { topics: 0, entries: 0, claims: 0 },
      discovery.diagnostics
    );
  }

  writeNativeTestInventory(options.workspaceRoot, inventory);
  const catalog = syncTestEvidenceIndex({
    workspaceRoot: options.workspaceRoot,
    mode: "write"
  });
  return projectReport(
    inventory,
    catalog.summary,
    catalog.diagnostics as TestEvidenceDiagnostic[]
  );
}

export async function createCurrentChangeReport(options: {
  workspaceRoot: string;
  baselinePath: string;
}): Promise<{
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  report: ReturnType<typeof compareInventoryBaseline> | null;
}> {
  const discovery = await discoverNativeTestEntries(options);
  if (discovery.diagnostics.some(({ blocking }) => blocking)) {
    return {
      status: "error",
      diagnostics: discovery.diagnostics,
      report: null
    };
  }
  let baseline;
  try {
    baseline = parseInventory(
      JSON.parse(fs.readFileSync(options.baselinePath, "utf8")) as unknown
    );
  } catch (error) {
    return {
      status: "error",
      diagnostics: [
        diagnostic(
          "baseline-invalid",
          "inventory",
          `cannot read baseline inventory: ${error instanceof Error ? error.message : String(error)}`,
          {
            path: toRelativePath(options.workspaceRoot, options.baselinePath)
          }
        )
      ],
      report: null
    };
  }
  return {
    status: "ok",
    diagnostics: [],
    report: compareInventoryBaseline(
      baseline,
      createNativeTestInventory(discovery)
    )
  };
}

export async function runTestEvidenceCli(
  argv: readonly string[] = process.argv.slice(2)
): Promise<number> {
  let options;
  try {
    options = parseArgs(argv);
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return 2;
  }

  if (["topics", "list", "show"].includes(options.command)) {
    return runTestEvidenceCatalogCli([...argv]);
  }

  if (options.command === "changes") {
    const result = await createCurrentChangeReport({
      workspaceRoot: options.workspaceRoot,
      baselinePath: path.resolve(
        options.workspaceRoot,
        String(options.baselinePath)
      )
    });
    writeResult(result, options.json);
    return result.status === "ok"
      ? 0
      : exitCodeForDiagnostics(result.diagnostics);
  }

  const result = options.command === "sync"
    ? await syncProjectTestEvidence({
        workspaceRoot: options.workspaceRoot
      })
    : await checkTestEvidence({
        workspaceRoot: options.workspaceRoot
      });
  writeResult(result, options.json);
  return result.status === "ok"
    ? 0
    : exitCodeForDiagnostics(result.diagnostics);
}

type ProjectTestEvidenceReport = {
  schemaVersion: 1;
  status: "ok" | "error";
  sourceRevision: string;
  diagnostics: TestEvidenceDiagnostic[];
  summary: {
    entries: number;
    cargo: number;
    bun: number;
    smoke: number;
    topics: number;
    claims: number;
  };
};

function projectReport(
  inventory: NativeTestInventory,
  catalogSummary: {
    topics: number;
    entries: number;
    claims: number;
  },
  diagnostics: TestEvidenceDiagnostic[]
): ProjectTestEvidenceReport {
  return {
    schemaVersion: 1,
    status: diagnostics.some(({ blocking }) => blocking) ? "error" : "ok",
    sourceRevision: inventory.sourceRevision,
    diagnostics,
    summary: {
      entries: inventory.entries.length,
      cargo: inventory.entries.filter(({ runner }) => runner === "cargo").length,
      bun: inventory.entries.filter(({ runner }) => runner === "bun").length,
      smoke: inventory.entries.filter(({ runner }) => runner === "smoke").length,
      topics: catalogSummary.topics,
      claims: catalogSummary.claims
    }
  };
}

function parseArgs(argv: readonly string[]): {
  command: "check" | "sync" | "changes" | "topics" | "list" | "show";
  workspaceRoot: string;
  baselinePath?: string;
  json: boolean;
} {
  const command = argv[0];
  if (!["check", "sync", "changes", "topics", "list", "show"].includes(command)) {
    throw new Error("usage: test-evidence <check|sync|changes|topics|list|show>");
  }
  let workspaceRoot: string | undefined;
  let baselinePath: string | undefined;
  let json = false;
  for (let index = 1; index < argv.length; index += 1) {
    const token = argv[index];
    if (token === "--json") {
      json = true;
    } else if (token === "--write" && command === "sync") {
      // sync is write-only at the project boundary.
    } else if (token === "--root" || token === "--baseline") {
      const value = argv[index + 1];
      if (!value || value.startsWith("--")) {
        throw new Error(`${token} requires a value`);
      }
      if (token === "--root") {
        workspaceRoot = path.resolve(value);
      } else {
        baselinePath = value;
      }
      index += 1;
    } else if (["topics", "list", "show"].includes(command)) {
      // The generic CLI validates the remaining query flags.
      if (!token.startsWith("--") && command === "show") {
        continue;
      }
      if (token.startsWith("--")) {
        index += token === "--json" ? 0 : 1;
      }
    } else {
      throw new Error(`unknown option ${token}`);
    }
  }
  if (!workspaceRoot) {
    throw new Error("--root is required");
  }
  if (command === "sync" && !argv.includes("--write")) {
    throw new Error("sync requires --write");
  }
  if (command === "changes" && !baselinePath) {
    throw new Error("changes requires --baseline");
  }
  return {
    command: command as "check" | "sync" | "changes" | "topics" | "list" | "show",
    workspaceRoot,
    baselinePath,
    json
  };
}

function writeResult(
  result: ProjectTestEvidenceReport | Awaited<ReturnType<typeof createCurrentChangeReport>>,
  json: boolean
): void {
  if (json) {
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    return;
  }
  if ("summary" in result && result.status === "ok") {
    process.stdout.write(
      `Test evidence check passed: ${result.summary.entries} native entry/entries ` +
      `(${result.summary.cargo} Cargo, ${result.summary.bun} Bun, ${result.summary.smoke} smoke), ` +
      `${result.summary.claims} claim(s).\n`
    );
    return;
  }
  if ("report" in result && result.status === "ok") {
    process.stdout.write(`${JSON.stringify(result.report, null, 2)}\n`);
    return;
  }
  for (const value of result.diagnostics) {
    process.stderr.write(
      `${value.origin}:${value.code}: ${value.message}` +
      `${value.path ? ` (${value.path}${value.line ? `:${value.line}` : ""})` : ""}\n`
    );
  }
}

export function exitCodeForDiagnostics(
  diagnostics: readonly TestEvidenceDiagnostic[]
): number {
  const origins = new Set(
    diagnostics.filter(({ blocking }) => blocking).map(({ origin }) => origin)
  );
  if (origins.has("profile") || origins.has("static")) {
    return 3;
  }
  if (origins.has("runner")) {
    return 4;
  }
  if (origins.has("inventory")) {
    return 5;
  }
  return 6;
}

function parseInventory(value: unknown): NativeTestInventory {
  if (
    !isRecord(value) ||
    value.schemaVersion !== 1 ||
    !isRecord(value.profile) ||
    typeof value.profile.id !== "string" ||
    !Number.isInteger(value.profile.version) ||
    typeof value.sourceRevision !== "string" ||
    !Array.isArray(value.entries)
  ) {
    throw new Error("baseline does not use NativeTestInventory v1");
  }
  return value as NativeTestInventory;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function toRelativePath(workspaceRoot: string, targetPath: string): string {
  return path.relative(workspaceRoot, targetPath).split(path.sep).join("/");
}
