import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  astSourceFingerprint,
  astSourceRange,
  scanAstRule,
  unsupportedAstDiagnostics
} from "../ast-scan.ts";
import { closeStaticAndRuntimeEntries } from "../closure.ts";
import {
  diagnostic,
  type NativeTestEntry,
  type RuntimeTestEntry,
  type StaticTestCandidate,
  type TestEvidenceDiagnostic
} from "../model.ts";
import type { SupportedRunnerProfile } from "../profile.ts";
import {
  processFailureMessage,
  runMiseCommand
} from "../runner-process.ts";
import { resolveBunTestFiles } from "./bun-files.ts";

export type BunJUnitCase = {
  name: string;
  className: string;
  file: string;
  line: number;
};

export async function discoverBunEntries(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}): Promise<{
  entries: NativeTestEntry[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  let files: string[];
  try {
    files = resolveBunTestFiles({
      workspaceRoot: options.workspaceRoot,
      profile: options.profile.bun
    });
  } catch (error) {
    return {
      entries: [],
      diagnostics: [
        diagnostic(
          "runner-profile-invalid",
          "profile",
          error instanceof Error ? error.message : String(error),
          { runner: "bun" }
        )
      ]
    };
  }

  const ruleRoot = path.join(
    options.workspaceRoot,
    "scripts",
    "test-evidence",
    "rules"
  );
  const nativeScan = await scanAstRule({
    workspaceRoot: options.workspaceRoot,
    rulePath: path.join(ruleRoot, "bun-native-test.yml"),
    paths: files
  });
  diagnostics.push(...nativeScan.diagnostics);
  for (const ruleName of [
    "bun-unsupported-alias.yml",
    "bun-unsupported-dynamic.yml",
    "bun-unsupported-parameterized.yml"
  ]) {
    const scan = await scanAstRule({
      workspaceRoot: options.workspaceRoot,
      rulePath: path.join(ruleRoot, ruleName),
      paths: files
    });
    diagnostics.push(...scan.diagnostics);
    diagnostics.push(...unsupportedAstDiagnostics(scan.matches, "bun"));
  }

  const statics: StaticTestCandidate[] = [];
  for (const match of nativeScan.matches) {
    const name = match.metaVariables.single.NAME?.text;
    if (!name) {
      diagnostics.push(diagnostic(
        "static-scan-failed",
        "static",
        "Bun native test rule did not capture NAME",
        {
          path: match.file,
          line: match.range.start.line + 1,
          runner: "bun"
        }
      ));
      continue;
    }
    statics.push({
      identity: bunLocationIdentity(
        match.file,
        match.range.start.line + 1,
        name
      ),
      sourcePath: match.file,
      sourceRange: astSourceRange(match),
      sourceFingerprint: astSourceFingerprint(match)
    });
  }

  const runtimeResult = await enumerateBunTests(options, files);
  diagnostics.push(...runtimeResult.diagnostics);
  if (diagnostics.some(({ blocking }) => blocking)) {
    return {
      entries: [],
      diagnostics
    };
  }
  const closed = closeStaticAndRuntimeEntries({
    runner: "bun",
    statics,
    runtime: runtimeResult.entries,
    createEntryKey: ({ target, selector }) => `bun|${target}|${selector}`
  });
  return {
    entries: closed.entries,
    diagnostics: [...diagnostics, ...closed.diagnostics]
  };
}

async function enumerateBunTests(options: {
  workspaceRoot: string;
  profile: SupportedRunnerProfile;
}, files: readonly string[]): Promise<{
  entries: RuntimeTestEntry[];
  diagnostics: TestEvidenceDiagnostic[];
}> {
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-bun-report-"));
  const reportPath = path.join(temporaryRoot, "junit.xml");
  try {
    const result = await runMiseCommand({
      workspaceRoot: options.workspaceRoot,
      command: "bun",
      args: [
        "test",
        ...files,
        "--reporter=junit",
        `--reporter-outfile=${reportPath}`
      ],
      label: "Bun test report"
    });
    if (result.status !== 0) {
      return {
        entries: [],
        diagnostics: [
          diagnostic(
            "runner-report-failed",
            "runner",
            processFailureMessage(result, "Bun test report"),
            { runner: "bun" }
          )
        ]
      };
    }
    if (!fs.existsSync(reportPath)) {
      return {
        entries: [],
        diagnostics: [
          diagnostic(
            "runner-report-invalid",
            "runner",
            "Bun test did not create the requested JUnit report",
            { runner: "bun" }
          )
        ]
      };
    }
    let cases;
    try {
      cases = parseBunJUnit(fs.readFileSync(reportPath, "utf8"));
    } catch (error) {
      return {
        entries: [],
        diagnostics: [
          diagnostic(
            "runner-report-invalid",
            "runner",
            `Bun JUnit report is malformed: ${error instanceof Error ? error.message : String(error)}`,
            { runner: "bun" }
          )
        ]
      };
    }
    return {
      entries: cases.map((testCase) => ({
        identity: bunLocationIdentity(
          testCase.file,
          testCase.line,
          testCase.name
        ),
        target: testCase.file,
        selector: testCase.className
          ? `${testCase.className} > ${testCase.name}`
          : testCase.name
      })),
      diagnostics: []
    };
  } finally {
    fs.rmSync(temporaryRoot, { force: true, recursive: true });
  }
}

export function parseBunJUnit(source: string): BunJUnitCase[] {
  const rootMatch = /<testsuites\b([^>]*)>/u.exec(source);
  if (!rootMatch) {
    throw new Error("testsuites root is missing");
  }
  const rootAttributes = parseXmlAttributes(rootMatch[1]);
  const expectedTests = parseNonNegativeInteger(rootAttributes.tests, "tests");
  const failures = parseNonNegativeInteger(rootAttributes.failures, "failures");
  if (failures !== 0) {
    throw new Error(`report contains ${failures} failure(s)`);
  }

  const cases: BunJUnitCase[] = [];
  for (const match of source.matchAll(/<testcase\b([^>]*)\/?>/gu)) {
    const attributes = parseXmlAttributes(match[1]);
    if (
      attributes.name === undefined ||
      attributes.file === undefined ||
      attributes.line === undefined
    ) {
      throw new Error("testcase is missing name, file or line");
    }
    const line = parseNonNegativeInteger(attributes.line, "testcase line");
    if (line < 1) {
      throw new Error("testcase line must be 1-based");
    }
    cases.push({
      name: attributes.name,
      className: attributes.classname ?? "",
      file: attributes.file.replaceAll("\\", "/"),
      line
    });
  }
  if (cases.length !== expectedTests) {
    throw new Error(
      `testsuites reports ${expectedTests} tests but contains ${cases.length} testcase elements`
    );
  }
  return cases;
}

function parseXmlAttributes(source: string): Record<string, string> {
  const attributes: Record<string, string> = {};
  for (const match of source.matchAll(/([A-Za-z_:][A-Za-z0-9_.:-]*)="([^"]*)"/gu)) {
    attributes[match[1]] = decodeXml(match[2]);
  }
  return attributes;
}

function decodeXml(value: string): string {
  return value.replace(
    /&(?:amp|lt|gt|quot|apos|#\d+|#x[0-9a-fA-F]+);/gu,
    (entity) => {
      switch (entity) {
        case "&amp;":
          return "&";
        case "&lt;":
          return "<";
        case "&gt;":
          return ">";
        case "&quot;":
          return "\"";
        case "&apos;":
          return "'";
        default:
          if (entity.startsWith("&#x")) {
            return String.fromCodePoint(Number.parseInt(entity.slice(3, -1), 16));
          }
          return String.fromCodePoint(Number.parseInt(entity.slice(2, -1), 10));
      }
    }
  );
}

function parseNonNegativeInteger(value: string | undefined, label: string): number {
  if (value === undefined || !/^\d+$/u.test(value)) {
    throw new Error(`${label} must be a non-negative integer`);
  }
  return Number.parseInt(value, 10);
}

function bunLocationIdentity(
  sourcePath: string,
  line: number,
  name: string
): string {
  return `${sourcePath}\0${line}\0${name}`;
}
