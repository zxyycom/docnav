import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

import {
  parseNativeTestInventory,
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
    baseline = parseNativeTestInventory(
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
  if (["topics", "list", "show"].includes(String(argv[0]))) {
    return runTestEvidenceCatalogCli([...argv]);
  }

  let options;
  try {
    options = parseProjectArgs(argv);
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return 2;
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

function parseProjectArgs(argv: readonly string[]): {
  command: "check" | "sync" | "changes";
  workspaceRoot: string;
  baselinePath?: string;
  json: boolean;
} {
  const command = argv[0];
  if (!["check", "sync", "changes"].includes(command)) {
    throw new Error("usage: test-evidence <check|sync|changes|topics|list|show>");
  }
  const { values } = parseArgs({
    args: [...argv.slice(1)],
    allowPositionals: false,
    strict: true,
    options: {
      root: { type: "string" },
      baseline: { type: "string" },
      json: { type: "boolean" },
      write: { type: "boolean" }
    }
  });
  if (!values.root) {
    throw new Error("--root is required");
  }
  if (command === "sync" && !values.write) {
    throw new Error("sync requires --write");
  }
  if (command !== "sync" && values.write) {
    throw new Error("--write is only valid with sync");
  }
  if (command === "changes" && !values.baseline) {
    throw new Error("changes requires --baseline");
  }
  return {
    command: command as "check" | "sync" | "changes",
    workspaceRoot: path.resolve(values.root),
    baselinePath: values.baseline,
    json: values.json ?? false
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

function toRelativePath(workspaceRoot: string, targetPath: string): string {
  return path.relative(workspaceRoot, targetPath).split(path.sep).join("/");
}
