import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { astSourceRange, scanAstRule, unsupportedAstDiagnostics, type AstMatch } from "../ast-scan.ts";
import { closeStaticAndRuntimeEntities } from "../closure.ts";
import { diagnostic, type RuntimeTestEntity, type StaticTestEntity, type TestEntity, type TestEvidenceDiagnostic } from "../model.ts";
import type { SupportedRunnerProfile } from "../profile.ts";
import { processFailureMessage, runMiseCommand } from "../runner-process.ts";
import { resolveBunTestFiles } from "./bun-files.ts";

type BunDiscoveryOptions = { workspaceRoot: string; profile: SupportedRunnerProfile };
type BunDiscoveryResult = { entities: TestEntity[]; diagnostics: TestEvidenceDiagnostic[] };
type BunRuntimeResult = { entities: RuntimeTestEntity[]; diagnostics: TestEvidenceDiagnostic[] };
type BunStaticResult = { entities: StaticTestEntity[]; diagnostics: TestEvidenceDiagnostic[] };

export type BunJUnitCase = {
  name: string;
  className: string;
  file: string;
  line: number;
};

export async function discoverBunEntities(
  options: BunDiscoveryOptions
): Promise<BunDiscoveryResult> {
  const diagnostics: TestEvidenceDiagnostic[] = [];
  let files: string[];
  try {
    files = resolveBunTestFiles({
      workspaceRoot: options.workspaceRoot,
      profile: options.profile.bun
    });
  } catch (error) {
    return {
      entities: [],
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

  const staticResult = await scanBunStaticEntities(options.workspaceRoot, files);
  diagnostics.push(...staticResult.diagnostics);
  const runtimeResult = await enumerateBunTests(options, files);
  diagnostics.push(...runtimeResult.diagnostics);
  if (diagnostics.some(({ blocking }) => blocking)) {
    return {
      entities: [],
      diagnostics
    };
  }
  const closed = closeStaticAndRuntimeEntities({
    runner: "bun",
    statics: staticResult.entities,
    runtime: runtimeResult.entities,
    createEntityKey: ({ target, selector }) => `bun|${target}|${selector}`
  });
  return {
    entities: closed.entities,
    diagnostics: [...diagnostics, ...closed.diagnostics]
  };
}

async function enumerateBunTests(
  options: BunDiscoveryOptions,
  files: readonly string[]
): Promise<BunRuntimeResult> {
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
        entities: [],
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
        entities: [],
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
        entities: [],
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
      entities: cases.map((testCase) => ({
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
    cases.push(parseBunJUnitCase(match[1]));
  }
  if (cases.length !== expectedTests) {
    throw new Error(
      `testsuites reports ${expectedTests} tests but contains ${cases.length} testcase elements`
    );
  }
  return cases;
}

async function scanBunStaticEntities(
  workspaceRoot: string,
  files: string[]
): Promise<BunStaticResult> {
  const ruleRoot = path.join(
    workspaceRoot,
    "scripts",
    "test-evidence",
    "rules"
  );
  const nativeScan = await scanAstRule({
    workspaceRoot,
    rulePath: path.join(ruleRoot, "bun-native-test.yml"),
    paths: files
  });
  const diagnostics = [...nativeScan.diagnostics];
  for (const ruleName of [
    "bun-unsupported-alias.yml",
    "bun-unsupported-dynamic.yml",
    "bun-unsupported-parameterized.yml"
  ]) {
    const scan = await scanAstRule({
      workspaceRoot,
      rulePath: path.join(ruleRoot, ruleName),
      paths: files
    });
    diagnostics.push(...scan.diagnostics);
    diagnostics.push(...unsupportedAstDiagnostics(scan.matches, "bun"));
  }
  const candidates = bunStaticCandidates(nativeScan.matches);
  return {
    entities: candidates.entities,
    diagnostics: [...diagnostics, ...candidates.diagnostics]
  };
}

function bunStaticCandidates(matches: readonly AstMatch[]): BunStaticResult {
  const entities: StaticTestEntity[] = [];
  const diagnostics: TestEvidenceDiagnostic[] = [];
  for (const match of matches) {
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
    entities.push({
      identity: bunLocationIdentity(
        match.file,
        match.range.start.line + 1,
        name
      ),
      sourcePath: match.file,
      sourceRange: astSourceRange(match)
    });
  }
  return { entities, diagnostics };
}

function parseBunJUnitCase(source: string): BunJUnitCase {
  const attributes = parseXmlAttributes(source);
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
  return {
    name: attributes.name,
    className: attributes.classname ?? "",
    file: attributes.file.replaceAll("\\", "/"),
    line
  };
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
