import path from "node:path";

import { runAstGrep } from "./ast-grep.ts";
import {
  diagnostic,
  type SourceRange,
  type TestEvidenceDiagnostic
} from "./model.ts";

type AstPosition = {
  line: number;
  column: number;
};

type AstRange = {
  start: AstPosition;
  end: AstPosition;
};

export type AstMatch = {
  file: string;
  text: string;
  range: AstRange;
  ruleId: string;
  metaVariables: {
    single: Record<string, {
      text: string;
      range: AstRange;
    }>;
  };
};

export async function scanAstRule(options: {
  workspaceRoot: string;
  rulePath: string;
  paths: string[];
}): Promise<{
  matches: AstMatch[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  let result;
  try {
    result = await runAstGrep([
      "scan",
      "--rule",
      options.rulePath,
      "--json=stream",
      "--color",
      "never",
      ...options.paths
    ], {
      workspaceRoot: options.workspaceRoot
    });
  } catch (error) {
    return {
      matches: [],
      diagnostics: [
        diagnostic(
          "static-scan-failed",
          "static",
          error instanceof Error ? error.message : String(error)
        )
      ]
    };
  }

  if (result.status !== 0 && result.status !== 1) {
    return {
      matches: [],
      diagnostics: [
        diagnostic(
          "static-scan-failed",
          "static",
          `ast-grep rule ${path.basename(options.rulePath)} failed with status ${String(result.status)}: ${result.stderr.trim()}`
        )
      ]
    };
  }

  try {
    return {
      matches: parseAstMatches(result.stdout),
      diagnostics: []
    };
  } catch (error) {
    return {
      matches: [],
      diagnostics: [
        diagnostic(
          "static-scan-failed",
          "static",
          `ast-grep rule ${path.basename(options.rulePath)} returned malformed JSON: ${error instanceof Error ? error.message : String(error)}`
        )
      ]
    };
  }
}

export function astSourceRange(match: AstMatch): SourceRange {
  return {
    startLine: match.range.start.line + 1,
    startColumn: match.range.start.column + 1,
    endLine: match.range.end.line + 1,
    endColumn: match.range.end.column + 1
  };
}

export function unsupportedAstDiagnostics(
  matches: readonly AstMatch[],
  runner: string
): TestEvidenceDiagnostic[] {
  return matches.map((match) => diagnostic(
    "unsupported-entity-shape",
    "static",
    `${runner} source uses unsupported test entity shape ${match.ruleId}`,
    {
      path: match.file,
      line: match.range.start.line + 1,
      column: match.range.start.column + 1,
      runner
    }
  ));
}

function parseAstMatches(stdout: string): AstMatch[] {
  return stdout
    .split(/\r?\n/u)
    .filter((line) => line.trim().length > 0)
    .map((line) => normalizeAstMatch(JSON.parse(line) as unknown));
}

function normalizeAstMatch(value: unknown): AstMatch {
  if (
    !isRecord(value) ||
    typeof value.file !== "string" ||
    typeof value.text !== "string" ||
    typeof value.ruleId !== "string" ||
    !isAstRange(value.range) ||
    !isRecord(value.metaVariables) ||
    !isRecord(value.metaVariables.single)
  ) {
    throw new Error("invalid ast-grep match shape");
  }
  const single = Object.fromEntries(
    Object.entries(value.metaVariables.single).map(([key, candidate]) => {
      if (
        !isRecord(candidate) ||
        typeof candidate.text !== "string" ||
        !isAstRange(candidate.range)
      ) {
        throw new Error(`invalid ast-grep metavariable ${key}`);
      }
      return [key, {
        text: candidate.text,
        range: candidate.range
      }];
    })
  );
  return {
    file: value.file,
    text: value.text,
    range: value.range,
    ruleId: value.ruleId,
    metaVariables: {
      single
    }
  };
}

function isAstRange(value: unknown): value is AstRange {
  return (
    isRecord(value) &&
    isAstPosition(value.start) &&
    isAstPosition(value.end)
  );
}

function isAstPosition(value: unknown): value is AstPosition {
  return (
    isRecord(value) &&
    Number.isInteger(value.line) &&
    Number(value.line) >= 0 &&
    Number.isInteger(value.column) &&
    Number(value.column) >= 0
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
