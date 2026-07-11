import { collectionFromEntries, type CaseIdCollection } from "./case-id-index.ts";
import { normalizeCasePath } from "./case-paths.ts";

export const CASE_CATALOG_DOC = "docs/testing/cases.md";

const CASE_STATUS_PATTERN = /^Status:\s+(\S+)/u;
const CASE_CODE_PATTERN = /^Code:\s+`([^`]+)`\s*$/u;
const CASE_CODE_PREFIX_PATTERN = /^Code:/u;
const CASE_PROVES_PATTERN = /^Proves:\s*$/u;
const CASE_HEADING_PATTERN = /^###+\s+((?:BB|WB|AUX)(?:-[A-Z0-9]+){2,}-\d{3})\b/u;

type CaseStatus = "implemented" | "planned";

export interface DocumentedCase {
  id: string;
  relPath: string;
  status: CaseStatus | null;
  line: number;
  codePath: string | null;
  codeDeclarations: number;
  invalidCode: boolean;
  provesDeclarations: number;
  provesContent: boolean;
}

export interface DocumentedCaseIndex {
  allIds: string[];
  ids: string[];
  implemented: CaseIdCollection & { entries: DocumentedCase[] };
  planned: CaseIdCollection & { entries: DocumentedCase[] };
  invalid: DocumentedCase[];
}

export function collectDocumentedCasesFromText(text: string): DocumentedCaseIndex {
  const entries = extractDocumentedCases(text);
  const implemented = entries.filter((entry) => entry.status === "implemented");
  const planned = entries.filter((entry) => entry.status === "planned");
  return {
    allIds: entries.map((entry) => entry.id).sort(),
    ids: [...new Set(entries.map((entry) => entry.id))].sort(),
    implemented: {
      ...collectionFromEntries(implemented.map((entry) => [entry.id, CASE_CATALOG_DOC] as const)),
      entries: implemented
    },
    planned: {
      ...collectionFromEntries(planned.map((entry) => [entry.id, CASE_CATALOG_DOC] as const)),
      entries: planned
    },
    invalid: entries.filter((entry) => entry.status === null)
  };
}

function extractDocumentedCases(text: string): DocumentedCase[] {
  const lines = text.split(/\r?\n/u);
  const entries: DocumentedCase[] = [];
  let current: DocumentedCase | null = null;
  let collectingProves = false;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index] ?? "";
    const heading = line.match(CASE_HEADING_PATTERN);
    if (heading !== null) {
      current = {
        id: heading[1] ?? "",
        relPath: CASE_CATALOG_DOC,
        status: null,
        line: index + 1,
        codePath: null,
        codeDeclarations: 0,
        invalidCode: false,
        provesDeclarations: 0,
        provesContent: false
      };
      collectingProves = false;
      entries.push(current);
      continue;
    }

    if (current === null) {
      continue;
    }

    const status = line.match(CASE_STATUS_PATTERN);
    if (status !== null) {
      current.status = parseCaseStatus(status[1] ?? "");
      continue;
    }

    if (CASE_CODE_PREFIX_PATTERN.test(line)) {
      current.codeDeclarations += 1;
      const code = line.match(CASE_CODE_PATTERN);
      if (code === null) {
        current.invalidCode = true;
      } else if (current.codePath === null) {
        current.codePath = normalizeCasePath(code[1] ?? "");
      }
      continue;
    }

    if (CASE_PROVES_PATTERN.test(line)) {
      current.provesDeclarations += 1;
      collectingProves = true;
      continue;
    }

    if (line.startsWith("决策说明:")) {
      collectingProves = false;
      continue;
    }

    const trimmed = line.trim();
    if (collectingProves && (trimmed.startsWith("- ") || trimmed === "```mermaid")) {
      current.provesContent = true;
    }
  }

  return entries;
}

function parseCaseStatus(value: string): CaseStatus | null {
  if (value === "implemented" || value === "planned") {
    return value;
  }
  return null;
}
