import {
  diagnostic,
  type TestEvidenceDiagnostic
} from "../model.ts";
import type { SemanticTestCase } from "./model.ts";
import { isOwnerRef } from "./owner-ref.ts";
import type { TopicFileSource } from "./source.ts";

const CASE_HEADING_PATTERN =
  /^## Case ([A-Za-z0-9][A-Za-z0-9._-]*): (\S.*)$/;

export function parseTopicLines(
  source: TopicFileSource,
  diagnostics: TestEvidenceDiagnostic[]
): SemanticTestCase[] {
  diagnoseTopicHeading(source, diagnostics);
  return parseTopicBlocks(source, diagnostics);
}

function diagnoseTopicHeading(
  source: TopicFileSource,
  diagnostics: TestEvidenceDiagnostic[]
): void {
  if (
    source.lines[0]?.replace(/^\uFEFF/, "").trimEnd() ===
    `# ${source.topic}`
  ) {
    return;
  }
  diagnostics.push(diagnostic(
    "topic.heading-invalid",
    "case",
    `Case topic file ${source.fileName} must start with H1 "# ${source.topic}"`,
    { path: source.sourcePath, line: 1 }
  ));
}

function parseTopicBlocks(
  source: TopicFileSource,
  diagnostics: TestEvidenceDiagnostic[]
): SemanticTestCase[] {
  const cases: SemanticTestCase[] = [];
  let cursor = 1;
  while (cursor < source.lines.length) {
    const line = source.lines[cursor] ?? "";
    if (line.trim().length === 0) {
      cursor += 1;
      continue;
    }
    if (!isH2(line)) {
      diagnostics.push(diagnostic(
        "topic.content-unexpected",
        "case",
        "Case topic files may contain only blank lines and Case H2 blocks after the H1",
        { path: source.sourcePath, line: cursor + 1 }
      ));
      cursor += 1;
      continue;
    }

    const end = findNextH2(source.lines, cursor + 1);
    const match = CASE_HEADING_PATTERN.exec(line);
    if (match !== null) {
      cases.push(parseCaseBlock({
        lines: source.lines,
        start: cursor,
        end,
        topic: source.topic,
        sourcePath: source.sourcePath,
        diagnostics
      }, match));
    } else {
      diagnostics.push(invalidCaseHeading(
        line,
        source.sourcePath,
        cursor + 1
      ));
    }
    cursor = end;
  }
  return cases;
}

function invalidCaseHeading(
  line: string,
  sourcePath: string,
  sourceLine: number
): TestEvidenceDiagnostic {
  const malformedCase = line.startsWith("## Case");
  return diagnostic(
    malformedCase ? "case.heading-invalid" : "topic.heading-unexpected",
    "case",
    malformedCase
      ? "Case heading must use \"## Case <CASE-ID>: <title>\""
      : "Case topic files allow only \"## Case <CASE-ID>: <title>\" H2 headings",
    { path: sourcePath, line: sourceLine }
  );
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
