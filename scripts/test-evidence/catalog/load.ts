import path from "node:path";

import {
  diagnostic,
  type TestEvidenceDiagnostic
} from "../model.ts";
import { parseTopicLines } from "./markdown.ts";
import type {
  SemanticTestCase,
  TestCaseTopic
} from "./model.ts";
import {
  readTopicFile,
  relativeWorkspacePath
} from "./source.ts";

export function reconcileTopicFiles(options: {
  root: string;
  workspaceRoot: string;
  topics: readonly TestCaseTopic[];
  files: readonly string[];
  diagnostics: TestEvidenceDiagnostic[];
}): string[] {
  const filesByTopic = new Map(
    options.files.map((fileName) => [path.basename(fileName, ".md"), fileName])
  );
  const orderedFiles: string[] = [];
  for (const { id } of options.topics) {
    const fileName = filesByTopic.get(id);
    if (fileName === undefined) {
      options.diagnostics.push(diagnostic(
        "topic.file-missing",
        "case",
        `topic ${id} has no ${id}.md Case file`,
        {
          path: relativeWorkspacePath(
            options.workspaceRoot,
            path.join(options.root, `${id}.md`)
          )
        }
      ));
    } else {
      orderedFiles.push(fileName);
      filesByTopic.delete(id);
    }
  }
  for (const [topic, fileName] of [...filesByTopic].sort(compareTopicPair)) {
    options.diagnostics.push(diagnostic(
      "topic.unknown",
      "case",
      `Case file ${fileName} uses unknown topic ${topic}`,
      {
        path: relativeWorkspacePath(
          options.workspaceRoot,
          path.join(options.root, fileName)
        )
      }
    ));
  }
  return orderedFiles;
}

export function loadTopicCases(options: {
  root: string;
  workspaceRoot: string;
  files: readonly string[];
  diagnostics: TestEvidenceDiagnostic[];
}): SemanticTestCase[] {
  const cases: SemanticTestCase[] = [];
  for (const fileName of options.files) {
    const source = readTopicFile({
      root: options.root,
      fileName,
      workspaceRoot: options.workspaceRoot,
      diagnostics: options.diagnostics
    });
    if (source !== null) {
      cases.push(...parseTopicLines(source, options.diagnostics));
    }
  }
  return cases;
}

export function diagnoseDuplicateCaseIds(
  cases: readonly SemanticTestCase[],
  diagnostics: TestEvidenceDiagnostic[]
): void {
  const firstById = new Map<string, SemanticTestCase>();
  for (const testCase of cases) {
    const first = firstById.get(testCase.id);
    if (first === undefined) {
      firstById.set(testCase.id, testCase);
    } else {
      diagnostics.push(diagnostic(
        "case.id-duplicate",
        "case",
        `Case ID ${testCase.id} is duplicated; first declared in ${first.sourcePath}:${first.sourceLine}`,
        {
          caseId: testCase.id,
          path: testCase.sourcePath,
          line: testCase.sourceLine
        }
      ));
    }
  }
}

function compareTopicPair(
  left: readonly [string, string],
  right: readonly [string, string]
): number {
  return left[0] < right[0] ? -1 : left[0] > right[0] ? 1 : 0;
}
