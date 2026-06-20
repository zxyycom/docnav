import { exitCodes } from "../config.ts";
import { fixture, getNormalRef } from "../fixtures.ts";
import { runCli } from "../harness.ts";
import {
  expectExit,
  expectStderrIncludes,
  expectStdoutEmpty
} from "../assertions.ts";
import {
  assertHelpOutputModeCompatibility,
  assertProtocolJsonCompatibilityWarning,
  assertReadableJsonCompatibilityWarnings
} from "./cli-args-warnings.ts";

const HELP_OUTPUT_MODE_COMPATIBILITY = {
  modes: ["readable-view", "readable-json", "protocol-json"],
  textModeAssertion: "outline help does not mention text output mode"
} as const;

export function createCliArgumentFailureTasks() {
  return [
    // @case BB-MD-ARGS-001
    {
      id: "MD-ARGS-001",
      label: "MD-ARGS-001 strict CLI argument failure",
      run: testStrictCliArgumentFailure
    }
  ];
}

export function createCliArgumentCompatibilityWarningTasks() {
  return [
    // @case BB-MD-WARN-001
    {
      id: "MD-WARN-001",
      label: "MD-WARN-001 CLI compatibility warnings and help",
      run: testCliArgumentCompatibilityWarnings
    }
  ];
}

async function testStrictCliArgumentFailure() {
  const normal = fixture("normal.md");
  const record = await runCli("MD-ARGS-001 find missing required --query", ["find", normal]);
  expectExit(record, exitCodes.input);
  expectStdoutEmpty(record);
  expectStderrIncludes(record, "find requires --query <text>");
}

async function testCliArgumentCompatibilityWarnings() {
  const normal = fixture("normal.md");
  const ref = await getNormalRef();

  await assertHelpOutputModeCompatibility(HELP_OUTPUT_MODE_COMPATIBILITY);
  await assertReadableJsonCompatibilityWarnings(normal, ref);
  await assertProtocolJsonCompatibilityWarning(normal);
}
