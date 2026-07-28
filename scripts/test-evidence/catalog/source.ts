import fs from "node:fs";
import path from "node:path";

import {
  diagnostic,
  type TestEvidenceDiagnostic
} from "../model.ts";
import { resolveExistingWorkspacePath } from "../relative-path.ts";
import type { TestCaseTopic } from "./model.ts";

export const CASES_SOURCE_PATH = "docs/testing/cases";

export type TopicFileSource = {
  fileName: string;
  topic: string;
  sourcePath: string;
  lines: string[];
};

export function resolveCaseDirectory(
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

export function readTopics(
  root: string,
  workspaceRoot: string
): {
  topics: TestCaseTopic[];
  diagnostics: TestEvidenceDiagnostic[];
} {
  const sourcePath = path.join(root, "topics.json");
  const displayPath = relativeWorkspacePath(workspaceRoot, sourcePath);
  try {
    return parseTopics(readTopicsValue(workspaceRoot), displayPath);
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
}

export function readTopicFiles(
  root: string,
  workspaceRoot: string,
  diagnostics: TestEvidenceDiagnostic[]
): string[] {
  try {
    return collectTopicFiles(
      fs.readdirSync(root, { withFileTypes: true }),
      root,
      workspaceRoot,
      diagnostics
    );
  } catch (error) {
    diagnostics.push(diagnostic(
      "cases.directory-invalid",
      "case",
      `cannot read semantic Case directory: ${errorMessage(error)}`,
      { path: relativeWorkspacePath(workspaceRoot, root) }
    ));
    return [];
  }
}

function collectTopicFiles(
  entries: readonly fs.Dirent[],
  root: string,
  workspaceRoot: string,
  diagnostics: TestEvidenceDiagnostic[]
): string[] {
  const files: string[] = [];
  for (const entry of entries) {
    const sourcePath = relativeWorkspacePath(
      workspaceRoot,
      path.join(root, entry.name)
    );
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
}

export function readTopicFile(options: {
  root: string;
  fileName: string;
  workspaceRoot: string;
  diagnostics: TestEvidenceDiagnostic[];
}): TopicFileSource | null {
  const topic = path.basename(options.fileName, ".md");
  const absolutePath = path.join(options.root, options.fileName);
  const sourcePath = relativeWorkspacePath(
    options.workspaceRoot,
    absolutePath
  );
  try {
    const resolved = resolveExistingWorkspacePath(
      options.workspaceRoot,
      sourcePath,
      `Case topic file ${options.fileName}`
    );
    if (!resolved.stats.isFile()) {
      throw new Error(`Case topic file ${options.fileName} must be a regular file`);
    }
    return {
      fileName: options.fileName,
      topic,
      sourcePath,
      lines: fs.readFileSync(resolved.absolutePath, "utf8")
        .replaceAll("\r\n", "\n")
        .split("\n")
    };
  } catch (error) {
    options.diagnostics.push(diagnostic(
      "topic.file-invalid",
      "case",
      `cannot read Case topic file: ${errorMessage(error)}`,
      { path: sourcePath }
    ));
    return null;
  }
}

export function relativeWorkspacePath(
  workspaceRoot: string,
  targetPath: string
): string {
  return path.relative(workspaceRoot, targetPath).split(path.sep).join("/");
}

function readTopicsValue(workspaceRoot: string): unknown {
  const resolved = resolveExistingWorkspacePath(
    workspaceRoot,
    `${CASES_SOURCE_PATH}/topics.json`,
    "semantic Case topic catalog"
  );
  if (!resolved.stats.isFile()) {
    throw new Error("semantic Case topic catalog must be a regular file");
  }
  return JSON.parse(fs.readFileSync(resolved.absolutePath, "utf8")) as unknown;
}

function parseTopics(
  value: unknown,
  displayPath: string
): {
  topics: TestCaseTopic[];
  diagnostics: TestEvidenceDiagnostic[];
} {
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
  for (const [index, valueTopic] of value.topics.entries()) {
    const topic = parseTopic(valueTopic);
    if (topic === null) {
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
      topics.push(topic);
    }
  }
  return { topics, diagnostics };
}

function parseTopic(value: unknown): TestCaseTopic | null {
  if (
    !isRecord(value) ||
    typeof value.id !== "string" ||
    !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(value.id) ||
    typeof value.description !== "string" ||
    value.description.trim().length === 0
  ) {
    return null;
  }
  return {
    id: value.id,
    description: value.description
  };
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
