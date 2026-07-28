import fs from "node:fs";
import path from "node:path";

import {
  diagnostic,
  type TestEntity,
  type TestEvidenceDiagnostic
} from "./model.ts";
import {
  isSafeRelativePosixPath,
  resolveExistingWorkspacePath
} from "./relative-path.ts";

const CASES_SOURCE_PATH = "docs/testing/cases";
const CASE_HEADING_PATTERN =
  /^## Case ([A-Za-z0-9][A-Za-z0-9._-]*): (\S.*)$/;
const DEFAULT_LIMIT = 50;
const MAX_LIMIT = 100;

export type TestCaseTopic = {
  id: string;
  description: string;
};

export type SemanticTestCase = {
  id: string;
  title: string;
  topic: string;
  ownerRef: string;
  entityKeys: string[];
  proves: string[];
  sourcePath: string;
  sourceLine: number;
};

export type TestCaseCatalog = {
  schemaVersion: 1;
  topics: TestCaseTopic[];
  cases: SemanticTestCase[];
  diagnostics: TestEvidenceDiagnostic[];
};

export function loadTestCaseCatalog(options: {
  workspaceRoot: string;
}): TestCaseCatalog {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const root = resolveCaseDirectory(options.workspaceRoot, diagnostics);
  if (root === null) {
    return {
      schemaVersion: 1,
      topics: [],
      cases: [],
      diagnostics
    };
  }
  const topicResult = readTopics(root, options.workspaceRoot);
  diagnostics.push(...topicResult.diagnostics);
  const files = readTopicFiles(root, options.workspaceRoot, diagnostics);
  const filesByTopic = new Map(
    files.map((fileName) => [path.basename(fileName, ".md"), fileName])
  );
  const orderedFiles: string[] = [];

  for (const { id } of topicResult.topics) {
    const fileName = filesByTopic.get(id);
    if (fileName === undefined) {
      diagnostics.push(diagnostic(
        "topic.file-missing",
        "case",
        `topic ${id} has no ${id}.md Case file`,
        { path: relative(options.workspaceRoot, path.join(root, `${id}.md`)) }
      ));
    } else {
      orderedFiles.push(fileName);
      filesByTopic.delete(id);
    }
  }
  for (const [topic, fileName] of [...filesByTopic].sort(compareTopicPair)) {
    diagnostics.push(diagnostic(
      "topic.unknown",
      "case",
      `Case file ${fileName} uses unknown topic ${topic}`,
      { path: relative(options.workspaceRoot, path.join(root, fileName)) }
    ));
  }

  const cases = orderedFiles.flatMap((fileName) => parseTopicFile({
    root,
    fileName,
    workspaceRoot: options.workspaceRoot,
    diagnostics
  }));
  const firstById = new Map<string, SemanticTestCase>();
  for (const testCase of cases) {
    const first = firstById.get(testCase.id);
    if (first === undefined) {
      firstById.set(testCase.id, testCase);
    } else {
      diagnostics.push(caseDiagnostic(
        "case.id-duplicate",
        `Case ID ${testCase.id} is duplicated; first declared in ${first.sourcePath}:${first.sourceLine}`,
        testCase
      ));
    }
  }
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

function readTopics(
  root: string,
  workspaceRoot: string
): {
  topics: TestCaseTopic[];
  diagnostics: TestEvidenceDiagnostic[];
} {
  const sourcePath = path.join(root, "topics.json");
  const displayPath = relative(workspaceRoot, sourcePath);
  let value: unknown;
  try {
    const resolved = resolveExistingWorkspacePath(
      workspaceRoot,
      `${CASES_SOURCE_PATH}/topics.json`,
      "semantic Case topic catalog"
    );
    if (!resolved.stats.isFile()) {
      throw new Error("semantic Case topic catalog must be a regular file");
    }
    value = JSON.parse(
      fs.readFileSync(resolved.absolutePath, "utf8")
    ) as unknown;
  } catch (error) {
    const exists = pathEntryExists(sourcePath);
    return {
      topics: [],
      diagnostics: [
        diagnostic(
          exists ? "topics.invalid" : "topics.missing",
          "case",
          exists
            ? `semantic Case topic catalog is invalid: ${errorMessage(error)}`
            : `semantic Case topic catalog is missing ${displayPath}`,
          { path: displayPath }
        )
      ]
    };
  }
  if (!isRecord(value) || value.schemaVersion !== 1 || !Array.isArray(value.topics)) {
    return {
      topics: [],
      diagnostics: [
        diagnostic(
          "topics.invalid",
          "case",
          "semantic Case topic catalog must have schemaVersion 1 and a topics array",
          { path: displayPath }
        )
      ]
    };
  }

  const topics: TestCaseTopic[] = [];
  const diagnostics: TestEvidenceDiagnostic[] = [];
  const seen = new Set<string>();
  for (const [index, topic] of value.topics.entries()) {
    if (
      !isRecord(topic) ||
      typeof topic.id !== "string" ||
      !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(topic.id) ||
      typeof topic.description !== "string" ||
      topic.description.trim().length === 0
    ) {
      diagnostics.push(diagnostic(
        "topic.invalid",
        "case",
        `topic at index ${index} must have a stable id and non-empty description`,
        { path: displayPath }
      ));
    } else if (seen.has(topic.id)) {
      diagnostics.push(diagnostic(
        "topic.duplicate",
        "case",
        `topic catalog repeats topic ${topic.id}`,
        { path: displayPath }
      ));
    } else {
      seen.add(topic.id);
      topics.push({ id: topic.id, description: topic.description });
    }
  }
  return { topics, diagnostics };
}

function readTopicFiles(
  root: string,
  workspaceRoot: string,
  diagnostics: TestEvidenceDiagnostic[]
): string[] {
  try {
    const files: string[] = [];
    for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
      const sourcePath = relative(workspaceRoot, path.join(root, entry.name));
      if (entry.isSymbolicLink()) {
        diagnostics.push(diagnostic(
          "cases.symlink-unsupported",
          "case",
          `semantic Case directory member must not be a symbolic link: ${entry.name}`,
          { path: sourcePath }
        ));
      } else if (entry.isDirectory()) {
        diagnostics.push(diagnostic(
          "cases.nested-directory",
          "case",
          `semantic Case directory must not contain nested directory ${entry.name}`,
          { path: sourcePath }
        ));
      } else if (entry.isFile()) {
        if (entry.name.endsWith(".md")) {
          files.push(entry.name);
        }
      } else if (entry.name.endsWith(".md")) {
        diagnostics.push(diagnostic(
          "topic.file-invalid",
          "case",
          `Case topic source must be a regular file: ${entry.name}`,
          { path: sourcePath }
        ));
      }
    }
    return files.sort();
  } catch (error) {
    diagnostics.push(diagnostic(
      "cases.directory-invalid",
      "case",
      `cannot read semantic Case directory: ${errorMessage(error)}`,
      { path: relative(workspaceRoot, root) }
    ));
    return [];
  }
}

function parseTopicFile(options: {
  root: string;
  fileName: string;
  workspaceRoot: string;
  diagnostics: TestEvidenceDiagnostic[];
}): SemanticTestCase[] {
  const topic = path.basename(options.fileName, ".md");
  const absolutePath = path.join(options.root, options.fileName);
  const sourcePath = relative(options.workspaceRoot, absolutePath);
  let lines: string[];
  try {
    const resolved = resolveExistingWorkspacePath(
      options.workspaceRoot,
      sourcePath,
      `Case topic file ${options.fileName}`
    );
    if (!resolved.stats.isFile()) {
      throw new Error(`Case topic file ${options.fileName} must be a regular file`);
    }
    lines = fs.readFileSync(resolved.absolutePath, "utf8")
      .replaceAll("\r\n", "\n")
      .split("\n");
  } catch (error) {
    options.diagnostics.push(diagnostic(
      "topic.file-invalid",
      "case",
      `cannot read Case topic file: ${errorMessage(error)}`,
      { path: sourcePath }
    ));
    return [];
  }
  if (lines[0]?.replace(/^\uFEFF/, "").trimEnd() !== `# ${topic}`) {
    options.diagnostics.push(diagnostic(
      "topic.heading-invalid",
      "case",
      `Case topic file ${options.fileName} must start with H1 "# ${topic}"`,
      { path: sourcePath, line: 1 }
    ));
  }

  const cases: SemanticTestCase[] = [];
  let cursor = 1;
  while (cursor < lines.length) {
    const line = lines[cursor] ?? "";
    if (line.trim().length === 0) {
      cursor += 1;
      continue;
    }
    if (!isH2(line)) {
      options.diagnostics.push(diagnostic(
        "topic.content-unexpected",
        "case",
        "Case topic files may contain only blank lines and Case H2 blocks after the H1",
        { path: sourcePath, line: cursor + 1 }
      ));
      cursor += 1;
      continue;
    }

    const end = findNextH2(lines, cursor + 1);
    const match = CASE_HEADING_PATTERN.exec(line);
    if (match !== null) {
      cases.push(parseCaseBlock({
        lines,
        start: cursor,
        end,
        topic,
        sourcePath,
        diagnostics: options.diagnostics
      }, match));
    } else {
      options.diagnostics.push(diagnostic(
        line.startsWith("## Case")
          ? "case.heading-invalid"
          : "topic.heading-unexpected",
        "case",
        line.startsWith("## Case")
          ? "Case heading must use \"## Case <CASE-ID>: <title>\""
          : "Case topic files allow only \"## Case <CASE-ID>: <title>\" H2 headings",
        { path: sourcePath, line: cursor + 1 }
      ));
    }
    cursor = end;
  }
  return cases;
}

function parseCaseBlock(options: {
  lines: readonly string[];
  start: number;
  end: number;
  topic: string;
  sourcePath: string;
  diagnostics: TestEvidenceDiagnostic[];
}, match: RegExpExecArray): SemanticTestCase {
  const [, id, title] = match;
  const content = options.lines
    .slice(options.start + 1, options.end)
    .map((text, index) => ({ text: text.trim(), line: options.start + index + 2 }))
    .filter(({ text }) => text.length > 0);
  let cursor = 0;
  const current = (): { text: string; line: number } | undefined => content[cursor];
  const report = (code: string, message: string, line = options.start + 1): void => {
    options.diagnostics.push(diagnostic(code, "case", message, {
      caseId: id,
      path: options.sourcePath,
      line
    }));
  };

  let ownerRef = "";
  if (!current()?.text.startsWith("Owner:")) {
    report("case.owner-missing", `Case ${id} has no Owner field`);
  } else {
    const owner = /^Owner: `([^`]+)`$/.exec(current()?.text ?? "");
    if (owner === null || !isOwnerRef(owner[1])) {
      report(
        "case.owner-invalid",
        `Case ${id} Owner must be a backticked workspace-relative .md#heading reference`,
        current()?.line
      );
    } else {
      ownerRef = owner[1];
    }
    cursor += 1;
  }

  const entityKeys: string[] = [];
  if (current()?.text !== "Entities:") {
    report("case.entities-missing", `Case ${id} has no Entities field`, current()?.line);
  } else {
    cursor += 1;
    const seen = new Set<string>();
    while (current() !== undefined && current()?.text !== "Proves:") {
      const item = current();
      const entity = /^- `([^`]+)`$/.exec(item?.text ?? "");
      if (entity === null || entity[1].trim() !== entity[1]) {
        report(
          "case.entity-invalid",
          `Case ${id} Entities must contain exact backticked entity key bullets`,
          item?.line
        );
      } else if (seen.has(entity[1])) {
        options.diagnostics.push(diagnostic(
          "case.entity-duplicate",
          "case",
          `Case ${id} repeats test entity ${entity[1]}`,
          {
            caseId: id,
            entityKey: entity[1],
            path: options.sourcePath,
            line: item?.line
          }
        ));
      } else {
        seen.add(entity[1]);
        entityKeys.push(entity[1]);
      }
      cursor += 1;
    }
    if (entityKeys.length === 0) {
      report(
        "case.entities-empty",
        `implemented Case ${id} must reference at least one test entity`
      );
    }
  }

  const proves: string[] = [];
  if (current()?.text !== "Proves:") {
    report("case.proves-missing", `Case ${id} has no Proves field`, current()?.line);
  } else {
    cursor += 1;
    while (current() !== undefined) {
      const item = current();
      const proof = /^- (\S.*)$/.exec(item?.text ?? "");
      if (proof === null) {
        report(
          "case.proves-invalid",
          `Case ${id} Proves must contain non-empty semantic bullets`,
          item?.line
        );
      } else {
        proves.push(proof[1]);
      }
      cursor += 1;
    }
    if (proves.length === 0) {
      report(
        "case.proves-empty",
        `Case ${id} must have at least one non-empty Proves bullet`
      );
    }
  }

  return {
    id,
    title,
    topic: options.topic,
    ownerRef,
    entityKeys,
    proves,
    sourcePath: options.sourcePath,
    sourceLine: options.start + 1
  };
}

function isOwnerRef(value: string): boolean {
  const separator = value.indexOf("#");
  const sourcePath = value.slice(0, separator);
  const heading = value.slice(separator + 1);
  return (
    separator > 0 &&
    separator === value.lastIndexOf("#") &&
    sourcePath.endsWith(".md") &&
    isSafeRelativePosixPath(sourcePath) &&
    heading.length > 0 &&
    heading.trim() === heading &&
    !/\s/.test(heading)
  );
}

function diagnoseOwnerRefs(
  cases: readonly SemanticTestCase[],
  workspaceRoot: string,
  diagnostics: TestEvidenceDiagnostic[]
): void {
  const anchorsByPath = new Map<string, Set<string> | Error>();
  for (const testCase of cases) {
    if (testCase.ownerRef.length === 0) {
      continue;
    }
    const [sourcePath, heading] = testCase.ownerRef.split("#");
    let anchors = anchorsByPath.get(sourcePath);
    if (anchors === undefined) {
      try {
        const resolved = resolveExistingWorkspacePath(
          workspaceRoot,
          sourcePath,
          `Case Owner ${sourcePath}`
        );
        if (!resolved.stats.isFile()) {
          throw new Error(`Case Owner ${sourcePath} must be a regular file`);
        }
        anchors = markdownHeadingAnchors(
          fs.readFileSync(resolved.absolutePath, "utf8")
        );
      } catch (error) {
        anchors = error instanceof Error ? error : new Error(String(error));
      }
      anchorsByPath.set(sourcePath, anchors);
    }
    if (anchors instanceof Error) {
      diagnostics.push(caseDiagnostic(
        "case.owner-unknown",
        `Case ${testCase.id} Owner cannot be resolved: ${anchors.message}`,
        testCase
      ));
    } else if (!anchors.has(heading)) {
      diagnostics.push(caseDiagnostic(
        "case.owner-heading-unknown",
        `Case ${testCase.id} Owner heading does not exist: ${testCase.ownerRef}`,
        testCase
      ));
    }
  }
}

function markdownHeadingAnchors(source: string): Set<string> {
  const anchors = new Set<string>();
  const repetitions = new Map<string, number>();
  const lines = source.split(/\r?\n/u);
  let cursor = skipDocumentFrontmatter(lines);
  let fence: { marker: "`" | "~"; length: number } | undefined;
  for (; cursor < lines.length; cursor += 1) {
    const line = cursor === 0
      ? (lines[cursor] ?? "").replace(/^\uFEFF/u, "")
      : (lines[cursor] ?? "");
    if (fence !== undefined) {
      if (closesFence(line, fence)) {
        fence = undefined;
      }
      continue;
    }
    const openingFence = readOpeningFence(line);
    if (openingFence !== undefined) {
      fence = openingFence;
      continue;
    }
    const match = /^#{1,6}[ \t]+(.+?)[ \t]*#*[ \t]*$/u.exec(line);
    if (match === null) {
      continue;
    }
    const base = match[1]
      .toLowerCase()
      .replace(/[^\p{Letter}\p{Mark}\p{Number}\s_-]/gu, "")
      .replace(/\s/gu, "-");
    const occurrence = repetitions.get(base) ?? 0;
    anchors.add(occurrence === 0 ? base : `${base}-${occurrence}`);
    repetitions.set(base, occurrence + 1);
  }
  return anchors;
}

function resolveCaseDirectory(
  workspaceRoot: string,
  diagnostics: TestEvidenceDiagnostic[]
): string | null {
  try {
    const resolved = resolveExistingWorkspacePath(
      workspaceRoot,
      CASES_SOURCE_PATH,
      "semantic Case directory"
    );
    if (!resolved.stats.isDirectory()) {
      throw new Error("semantic Case directory must be a directory");
    }
    return resolved.absolutePath;
  } catch (error) {
    diagnostics.push(diagnostic(
      "cases.directory-invalid",
      "case",
      `cannot read semantic Case directory: ${errorMessage(error)}`,
      { path: CASES_SOURCE_PATH }
    ));
    return null;
  }
}

function findNextH2(lines: readonly string[], start: number): number {
  for (let index = start; index < lines.length; index += 1) {
    if (isH2(lines[index] ?? "")) {
      return index;
    }
  }
  return lines.length;
}

function isH2(line: string): boolean {
  return /^##(?:[ \t]|$)/u.test(line);
}

function skipDocumentFrontmatter(lines: readonly string[]): number {
  if ((lines[0] ?? "").replace(/^\uFEFF/u, "").trim() !== "---") {
    return 0;
  }
  for (let index = 1; index < lines.length; index += 1) {
    const line = (lines[index] ?? "").trim();
    if (line === "---" || line === "...") {
      return index + 1;
    }
  }
  return lines.length;
}

function readOpeningFence(
  line: string
): { marker: "`" | "~"; length: number } | undefined {
  const match = /^ {0,3}(`{3,}|~{3,})/u.exec(line);
  if (match === null) {
    return undefined;
  }
  return {
    marker: match[1][0] as "`" | "~",
    length: match[1].length
  };
}

function closesFence(
  line: string,
  fence: { marker: "`" | "~"; length: number }
): boolean {
  const match = /^ {0,3}(`+|~+)[ \t]*$/u.exec(line);
  return (
    match !== null &&
    match[1][0] === fence.marker &&
    match[1].length >= fence.length
  );
}

function caseDiagnostic(
  code: string,
  message: string,
  testCase: SemanticTestCase
): TestEvidenceDiagnostic {
  return diagnostic(code, "case", message, {
    caseId: testCase.id,
    path: testCase.sourcePath,
    line: testCase.sourceLine
  });
}

function status(
  diagnostics: readonly TestEvidenceDiagnostic[]
): "ok" | "error" {
  return diagnostics.some(({ blocking }) => blocking) ? "error" : "ok";
}

function compareTopicPair(
  left: readonly [string, string],
  right: readonly [string, string]
): number {
  return left[0] < right[0] ? -1 : left[0] > right[0] ? 1 : 0;
}

function relative(workspaceRoot: string, targetPath: string): string {
  return path.relative(workspaceRoot, targetPath).split(path.sep).join("/");
}

function pathEntryExists(targetPath: string): boolean {
  try {
    fs.lstatSync(targetPath);
    return true;
  } catch {
    return false;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
