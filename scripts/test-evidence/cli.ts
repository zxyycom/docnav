import path from "node:path";
import { parseArgs } from "node:util";

import {
  listTestCaseTopics,
  loadTestCaseCatalog,
  queryTestCases,
  showTestCase,
  validateQueryWindow,
  validateTestCaseCoverage
} from "./cases.ts";
import { discoverTestEntities } from "./discover.ts";
import type { TestEntity, TestEvidenceDiagnostic } from "./model.ts";

export type ProjectTestEvidenceReport = {
  schemaVersion: 1;
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  summary: {
    entities: number;
    cargo: number;
    bun: number;
    smoke: number;
    mappedEntities: number;
    topics: number;
    cases: number;
  };
};

export async function checkTestEvidence(options: {
  workspaceRoot: string;
}): Promise<ProjectTestEvidenceReport> {
  const discovery = await discoverTestEntities(options);
  const catalog = loadTestCaseCatalog(options);
  const diagnostics = [
    ...discovery.diagnostics,
    ...catalog.diagnostics
  ];
  if (!discovery.diagnostics.some(({ blocking }) => blocking)) {
    diagnostics.push(...validateTestCaseCoverage({
      catalog,
      entities: discovery.entities
    }));
  }
  return projectReport(discovery.entities, catalog, diagnostics);
}

export async function runTestEvidenceCli(
  argv: readonly string[] = process.argv.slice(2)
): Promise<number> {
  let command: ParsedCommand;
  try {
    command = parseCommand(argv);
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    return 2;
  }

  if (command.command === "topics") {
    const result = listTestCaseTopics({
      workspaceRoot: command.workspaceRoot
    });
    writeJson(result);
    return result.status === "ok"
      ? 0
      : exitCodeForDiagnostics(result.diagnostics);
  }
  if (command.command === "list") {
    const result = queryTestCases(command);
    writeJson(result);
    return result.status === "ok"
      ? 0
      : exitCodeForDiagnostics(result.diagnostics);
  }
  if (command.command === "show") {
    const result = showTestCase({
      workspaceRoot: command.workspaceRoot,
      id: command.id
    });
    writeJson(result);
    return result.status === "ok"
      ? 0
      : exitCodeForDiagnostics(result.diagnostics);
  }

  const result = await checkTestEvidence({
    workspaceRoot: command.workspaceRoot
  });
  writeCheckResult(result, command.json);
  return result.status === "ok"
    ? 0
    : exitCodeForDiagnostics(result.diagnostics);
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
  if (origins.has("case")) {
    return 5;
  }
  return 6;
}

type ParsedCommand =
  | {
      command: "check";
      workspaceRoot: string;
      json: boolean;
    }
  | {
      command: "topics";
      workspaceRoot: string;
    }
  | {
      command: "list";
      workspaceRoot: string;
      topic?: string;
      entityKey?: string;
      ownerRef?: string;
      query?: string;
      offset?: number;
      limit?: number;
    }
  | {
      command: "show";
      workspaceRoot: string;
      id: string;
    };

function parseCommand(argv: readonly string[]): ParsedCommand {
  const command = argv[0];
  if (
    command !== "check" &&
    command !== "topics" &&
    command !== "list" &&
    command !== "show"
  ) {
    throw new Error("usage: test-evidence <check|topics|list|show>");
  }
  const args = [...argv.slice(1)];
  if (command === "show") {
    return parseShowCommand(args);
  }
  if (command === "list") {
    return parseListCommand(args);
  }
  return parseCheckOrTopicsCommand(command, args);
}

function parseShowCommand(args: string[]): ParsedCommand {
  const { positionals, values } = parseArgs({
    args,
    allowPositionals: true,
    strict: true,
    options: {
      root: { type: "string" }
    }
  });
  requireRoot(values.root);
  if (positionals.length !== 1) {
    throw new Error("show requires exactly one <CASE-ID>");
  }
  return {
    command: "show",
    workspaceRoot: path.resolve(values.root),
    id: positionals[0]
  };
}

function parseListCommand(args: string[]): ParsedCommand {
  const { values } = parseArgs({
    args,
    allowPositionals: false,
    strict: true,
    options: {
      root: { type: "string" },
      topic: { type: "string" },
      "entity-key": { type: "string" },
      "owner-ref": { type: "string" },
      query: { type: "string" },
      offset: { type: "string" },
      limit: { type: "string" }
    }
  });
  requireRoot(values.root);
  const offset = optionalInteger(values.offset);
  const limit = optionalInteger(values.limit);
  validateQueryWindow({ offset, limit });
  return {
    command: "list",
    workspaceRoot: path.resolve(values.root),
    ...(values.topic === undefined ? {} : { topic: values.topic }),
    ...(values["entity-key"] === undefined
      ? {}
      : { entityKey: values["entity-key"] }),
    ...(values["owner-ref"] === undefined
      ? {}
      : { ownerRef: values["owner-ref"] }),
    ...(values.query === undefined ? {} : { query: values.query }),
    ...(offset === undefined ? {} : { offset }),
    ...(limit === undefined ? {} : { limit })
  };
}

function parseCheckOrTopicsCommand(command: "check" | "topics", args: string[]): ParsedCommand {
  const { values } = parseArgs({
    args,
    allowPositionals: false,
    strict: true,
    options: {
      root: { type: "string" },
      json: { type: "boolean" }
    }
  });
  requireRoot(values.root);
  if (command === "topics") {
    if (values.json !== undefined) {
      throw new Error("--json is not needed: topics always writes JSON");
    }
    return {
      command,
      workspaceRoot: path.resolve(values.root)
    };
  }
  return {
    command: "check",
    workspaceRoot: path.resolve(values.root),
    json: values.json ?? false
  };
}

function projectReport(
  entities: readonly TestEntity[],
  catalog: ReturnType<typeof loadTestCaseCatalog>,
  diagnostics: TestEvidenceDiagnostic[]
): ProjectTestEvidenceReport {
  const currentKeys = new Set(entities.map(({ entityKey }) => entityKey));
  const mappedKeys = new Set(
    catalog.cases
      .flatMap(({ entityKeys }) => entityKeys)
      .filter((entityKey) => currentKeys.has(entityKey))
  );
  return {
    schemaVersion: 1,
    status: diagnostics.some(({ blocking }) => blocking) ? "error" : "ok",
    diagnostics,
    summary: {
      entities: entities.length,
      cargo: entities.filter(({ runner }) => runner === "cargo").length,
      bun: entities.filter(({ runner }) => runner === "bun").length,
      smoke: entities.filter(({ runner }) => runner === "smoke").length,
      mappedEntities: mappedKeys.size,
      topics: catalog.topics.length,
      cases: catalog.cases.length
    }
  };
}

function writeCheckResult(
  result: ProjectTestEvidenceReport,
  json: boolean
): void {
  if (json) {
    writeJson(result);
    return;
  }
  if (result.status === "ok") {
    process.stdout.write(
      `Test Case check passed: ${result.summary.entities} current test entities ` +
      `(${result.summary.cargo} Cargo, ${result.summary.bun} Bun, ` +
      `${result.summary.smoke} smoke); ${result.summary.mappedEntities} mapped by ` +
      `${result.summary.cases} semantic Cases across ${result.summary.topics} topics.\n`
    );
    return;
  }
  for (const value of result.diagnostics) {
    process.stderr.write(
      `${value.origin}:${value.code}: ${value.message}` +
      `${value.path ? ` (${value.path}${value.line ? `:${value.line}` : ""})` : ""}\n`
    );
  }
}

function writeJson(value: unknown): void {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
}

function requireRoot(value: string | undefined): asserts value is string {
  if (value === undefined || value.length === 0) {
    throw new Error("--root is required");
  }
}

function optionalInteger(value: string | undefined): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (!/^\d+$/.test(value)) {
    return Number.NaN;
  }
  return Number(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
