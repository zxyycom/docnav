import { exitCodes } from "../config.ts";
import { fixture, getNormalRef } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoWarningsField,
  expectProtocolSuccess,
  expectStderrEmpty,
  expectStderrIncludes,
  expectStderrWarning,
  expectStructuredWarning,
  expectStdoutEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.ts";

export function createCliArgumentFailureTasks() {
  return [
    {
      id: "MD-ARGS-001",
      label: "MD-ARGS-001 strict CLI argument failure",
      run: testStrictCliArgumentFailure
    }
  ];
}

export function createCliArgumentCompatibilityWarningTasks() {
  return [
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

  const outlineHelp = await runCli("MD-WARN-001 docnav-markdown outline help", ["outline", "--help"]);
  expectExit(outlineHelp, exitCodes.success);
  expectStderrEmpty(outlineHelp);
  expectStdoutIncludes(outlineHelp, "--max-heading-level");
  expectStdoutIncludes(outlineHelp, "--output");
  // 3.5: help only lists three final output modes for document operations.
  expectStdoutIncludes(outlineHelp, "readable-view");
  expectStdoutIncludes(outlineHelp, "readable-json");
  expectStdoutIncludes(outlineHelp, "protocol-json");
  expect(outlineHelp, !outlineHelp.stdout.includes("text"), "outline help does not mention text output mode");

  const readable = await runCli("MD-WARN-001 outline unknown readable-json warning", [
    "outline",
    normal,
    "--future",
    "--output",
    "readable-json"
  ]);
  expectExit(readable, exitCodes.success);
  expectStderrEmpty(readable);
  const readableJson = parseJson(readable);
  validateSchema(readable, "readableOutline", readableJson);
  expectStructuredWarning(readable, readableJson.warnings?.[0], ["--future"], "unknown flag");

  const unused = await runCli("MD-WARN-001 read unused known flag readable-json warning", [
    "read",
    normal,
    "--ref",
    ref,
    "--max-heading-level",
    "3",
    "--output",
    "readable-json"
  ]);
  expectExit(unused, exitCodes.success);
  expectStderrEmpty(unused);
  const unusedJson = parseJson(unused);
  validateSchema(unused, "readableRead", unusedJson);
  expectStructuredWarning(unused, unusedJson.warnings?.[0], ["--max-heading-level", "3"], "unused native flag");

  const protocol = await runCli("MD-WARN-001 outline unknown flag protocol-json stderr warning", [
    "outline",
    normal,
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(protocol, exitCodes.success);
  expectStderrWarning(protocol, ["--future"]);
  expectNoJsonPayloadInStderr(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "outline");
  expectNoWarningsField(protocol, protocolJson, "protocol-json stdout");
}
