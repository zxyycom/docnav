import {
  diagnoseDuplicateCaseIds,
  loadTopicCases,
  reconcileTopicFiles
} from "./catalog/load.ts";
import type {
  SemanticTestCase,
  TestCaseCatalog,
  TestCaseTopic
} from "./catalog/model.ts";
import { diagnoseOwnerRefs } from "./catalog/owner-ref.ts";
import {
  readTopicFiles,
  readTopics,
  resolveCaseDirectory
} from "./catalog/source.ts";
import {
  diagnostic,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "./model.ts";

export type {
  SemanticTestCase,
  TestCaseCatalog,
  TestCaseTopic
};

const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 100;

export function loadTestCaseCatalog(options: {
  workspaceRoot: string;
}): TestCaseCatalog {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const root = resolveCaseDirectory(options.workspaceRoot, diagnostics);
  if (root === null) {
    return emptyCatalog(diagnostics);
  }

  const topicResult = readTopics(root, options.workspaceRoot);
  diagnostics.push(...topicResult.diagnostics);
  const orderedFiles = reconcileTopicFiles({
    root,
    workspaceRoot: options.workspaceRoot,
    topics: topicResult.topics,
    files: readTopicFiles(root, options.workspaceRoot, diagnostics),
    diagnostics
  });
  const cases = loadTopicCases({
    root,
    workspaceRoot: options.workspaceRoot,
    files: orderedFiles,
    diagnostics
  });
  diagnoseDuplicateCaseIds(cases, diagnostics);
  diagnoseOwnerRefs(cases, options.workspaceRoot, diagnostics);

  return {
    schemaVersion: 1,
    topics: topicResult.topics,
    cases,
    diagnostics
  };
}

export function validateTestCaseCoverage(options: {
  catalog: TestCaseCatalog;
  entities: readonly TestEntity[];
}): TestEvidenceDiagnostic[] {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const entitiesByKey = new Map(
    options.entities.map((entity) => [entity.entityKey, entity])
  );
  const mapped = new Set<string>();
  for (const testCase of options.catalog.cases) {
    for (const entityKey of testCase.entityKeys) {
      if (entitiesByKey.has(entityKey)) {
        mapped.add(entityKey);
      } else {
        diagnostics.push(diagnostic(
          "case.entity-unknown",
          "case",
          `Case ${testCase.id} references unknown test entity ${entityKey}`,
          {
            caseId: testCase.id,
            entityKey,
            path: testCase.sourcePath,
            line: testCase.sourceLine
          }
        ));
      }
    }
  }
  for (const entity of options.entities) {
    if (!mapped.has(entity.entityKey)) {
      diagnostics.push(diagnostic(
        "entity.case-missing",
        "case",
        `current test entity has no semantic Case ${entity.entityKey}`,
        {
          entityKey: entity.entityKey,
          runner: entity.runner,
          target: entity.target,
          selector: entity.selector,
          path: entity.sourcePath,
          line: entity.sourceRange.startLine,
          column: entity.sourceRange.startColumn
        }
      ));
    }
  }
  return diagnostics;
}

export function listTestCaseTopics(options: {
  workspaceRoot: string;
}): {
  schemaVersion: 1;
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  topics: Array<TestCaseTopic & { cases: number }>;
} {
  const catalog = loadTestCaseCatalog(options);
  return {
    schemaVersion: 1,
    status: status(catalog.diagnostics),
    diagnostics: catalog.diagnostics,
    topics: catalog.topics.map((topic) => ({
      ...topic,
      cases: catalog.cases.filter((testCase) => testCase.topic === topic.id).length
    }))
  };
}

export function queryTestCases(options: {
  workspaceRoot: string;
  topic?: string;
  entityKey?: string;
  ownerRef?: string;
  query?: string;
  offset?: number;
  limit?: number;
}): {
  schemaVersion: 1;
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  offset: number;
  limit: number;
  total: number;
  items: SemanticTestCase[];
} {
  const catalog = loadTestCaseCatalog(options);
  const offset = options.offset ?? 0;
  const limit = options.limit ?? DEFAULT_LIMIT;
  const query = options.query?.toLowerCase();
  const matches = catalog.cases.filter((testCase) => (
    (options.topic === undefined || testCase.topic === options.topic) &&
    (options.entityKey === undefined || testCase.entityKeys.includes(options.entityKey)) &&
    (options.ownerRef === undefined || testCase.ownerRef === options.ownerRef) &&
    (
      query === undefined ||
      [
        testCase.id,
        testCase.title,
        testCase.topic,
        testCase.ownerRef,
        ...testCase.entityKeys,
        ...testCase.proves
      ].some((value) => value.toLowerCase().includes(query))
    )
  ));
  return {
    schemaVersion: 1,
    status: status(catalog.diagnostics),
    diagnostics: catalog.diagnostics,
    offset,
    limit,
    total: matches.length,
    items: matches.slice(offset, offset + limit)
  };
}

export function showTestCase(options: {
  workspaceRoot: string;
  id: string;
}): {
  schemaVersion: 1;
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  item: SemanticTestCase | null;
} {
  const catalog = loadTestCaseCatalog(options);
  const diagnostics = [...catalog.diagnostics];
  const matches = catalog.cases.filter(({ id }) => id === options.id);
  if (matches.length !== 1) {
    diagnostics.push(diagnostic(
      matches.length === 0 ? "query.case-not-found" : "query.case-ambiguous",
      "query",
      matches.length === 0
        ? `no semantic Case has ID ${options.id}`
        : `semantic Case ID ${options.id} is duplicated`,
      { caseId: options.id }
    ));
  }
  return {
    schemaVersion: 1,
    status: status(diagnostics),
    diagnostics,
    item: matches.length === 1 ? matches[0] : null
  };
}

export function validateQueryWindow(options: {
  offset?: number;
  limit?: number;
}): void {
  if (
    options.offset !== undefined &&
    (!Number.isInteger(options.offset) || options.offset < 0)
  ) {
    throw new Error("--offset must be a non-negative integer");
  }
  if (
    options.limit !== undefined &&
    (!Number.isInteger(options.limit) || options.limit < 1 || options.limit > MAX_LIMIT)
  ) {
    throw new Error(`--limit must be an integer from 1 to ${MAX_LIMIT}`);
  }
}

function emptyCatalog(
  diagnostics: TestEvidenceDiagnostic[]
): TestCaseCatalog {
  return {
    schemaVersion: 1,
    topics: [],
    cases: [],
    diagnostics
  };
}

function status(
  diagnostics: readonly TestEvidenceDiagnostic[]
): "ok" | "error" {
  return diagnostics.some(({ blocking }) => blocking) ? "error" : "ok";
}
