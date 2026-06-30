import { exitCodes } from "../config.ts";
import { fixture, getNormalRef } from "../fixtures.ts";
import { runCli } from "../harness.ts";
import {
  expectExit,
  expectStderrEmpty,
  expectStdoutIncludes
} from "../assertions.ts";
import {
  assertHelpOutputModeBoundary,
  assertProtocolJsonStrictInputFailure,
  assertReadableJsonStrictInputFailures
} from "./cli-args-boundaries.ts";

const HELP_OUTPUT_MODE_BOUNDARY = {
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

export function createCliInputBoundaryTasks() {
  return [
    // @case BB-MD-BOUNDARY-001
    {
      id: "MD-BOUNDARY-001",
      label: "MD-BOUNDARY-001 strict CLI input boundaries and help",
      run: testStrictCliInputBoundaries
    }
  ];
}

async function testStrictCliArgumentFailure() {
  const normal = fixture("normal.md");
  const record = await runCli("MD-ARGS-001 find missing required --query", ["find", normal]);
  expectExit(record, exitCodes.input);
  expectStderrEmpty(record);
  expectStdoutIncludes(record, "\"code\": \"INVALID_REQUEST\"");
  expectStdoutIncludes(record, "find requires --query <text>");
}

async function testStrictCliInputBoundaries() {
  const normal = fixture("normal.md");
  const ref = await getNormalRef();

  await assertHelpOutputModeBoundary(HELP_OUTPUT_MODE_BOUNDARY);
  await assertReadableJsonStrictInputFailures(normal, ref);
  await assertProtocolJsonStrictInputFailure(normal);
}
