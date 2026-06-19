import { collectionFromEntries, type CaseIdCollection } from "./collection.ts";
import { normalizeCasePath } from "./paths.ts";

export const CASE_CATALOG_DOC = "docs/testing/cases.md";

const CASE_STATUS_PATTERN = /^Status:\s+(\S+)/u;
const CASE_CODE_PATTERN = /^Code:\s+`([^`]+)`/u;
const CASE_HEADING_PATTERN = /^###+\s+((?:BB|WB|AUX)(?:-[A-Z0-9]+){2,}-\d{3})\b/u;

type CaseStatus = "implemented" | "planned";

export interface DocumentedCase {
  id: string;
  relPath: string;
  status: CaseStatus | null;
  line: number;
  codePath: string | null;
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

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index] ?? "";
    const heading = line.match(CASE_HEADING_PATTERN);
    if (heading !== null) {
      current = {
        id: heading[1] ?? "",
        relPath: CASE_CATALOG_DOC,
        status: null,
        line: index + 1,
        codePath: null
      };
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

    const code = line.match(CASE_CODE_PATTERN);
    if (code !== null) {
      current.codePath = normalizeCasePath(code[1] ?? "");
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

