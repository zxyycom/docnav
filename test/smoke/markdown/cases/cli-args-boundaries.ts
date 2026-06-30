import { exitCodes } from "../config.ts";
import { runCli, validateSchema } from "../harness.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoJsonPayloadInStderr,
  expectProtocolFailure,
  expectReadableFailure,
  expectStderrEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.ts";

type HelpOutputModeBoundary = {
  modes: readonly [string, string, string];
  textModeAssertion: string;
};

export async function assertHelpOutputModeBoundary(options: HelpOutputModeBoundary) {
  const outlineHelp = await runCli("MD-BOUNDARY-001 docnav-markdown outline help", ["outline", "--help"]);
  expectExit(outlineHelp, exitCodes.success);
  expectStderrEmpty(outlineHelp);
  expectStdoutIncludes(outlineHelp, "--max-heading-level");
  expectStdoutIncludes(outlineHelp, "--output");
  expectStdoutIncludes(outlineHelp, options.modes[0]);
  expectStdoutIncludes(outlineHelp, options.modes[1]);
  expectStdoutIncludes(outlineHelp, options.modes[2]);
  expect(outlineHelp, !outlineHelp.stdout.includes("text"), options.textModeAssertion);
}

export async function assertReadableJsonStrictInputFailures(normal: string, ref: string) {
  await assertReadableInputFailure("MD-BOUNDARY-001 outline unknown argv readable-json failure", [
    "outline",
    normal,
    "--future",
    "--output",
    "readable-json"
  ], {
    reason: "unknown_argument",
    tokens: ["--future"]
  });

  await assertReadableInputFailure("MD-BOUNDARY-001 read operation-inapplicable flag readable-json failure", [
    "read",
    normal,
    "--ref",
    String(ref),
    "--max-heading-level",
    "3",
    "--output",
    "readable-json"
  ], {
    reason: "operation_inapplicable_argument",
    tokens: ["--max-heading-level", "3"]
  });

  await assertReadableInputFailure("MD-BOUNDARY-001 read extra positional readable-json failure", [
    "read",
    normal,
    "--ref",
    String(ref),
    "extra.md",
    "--output",
    "readable-json"
  ], {
    reason: "extra_positional",
    tokens: ["extra.md"]
  });
}

export async function assertProtocolJsonStrictInputFailure(normal: string) {
  const protocol = await runCli("MD-BOUNDARY-001 outline unknown argv protocol-json failure", [
    "outline",
    normal,
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(protocol, exitCodes.input);
  expectNoJsonPayloadInStderr(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  const error = expectProtocolFailure(protocol, protocolJson, "outline", "INVALID_REQUEST");
  expectInvalidInputDiagnostic(protocol, error, {
    reason: "unknown_argument",
    tokens: ["--future"]
  });
}

type StrictInputFailure = {
  reason: string;
  tokens: readonly string[];
};

async function assertReadableInputFailure(name: string, args: string[], expected: StrictInputFailure) {
  const record = await runCli(name, args);
  expectExit(record, exitCodes.input);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "readableError", json);
  const error = expectReadableFailure(record, json, "INVALID_REQUEST");
  expectInvalidInputDiagnostic(record, error, expected);
}

function expectInvalidInputDiagnostic(record: CommandRecord, error: Record<string, unknown>, expected: StrictInputFailure) {
  const details = expectJsonObject(record, error.details, "INVALID_REQUEST details is an object");
  expect(record, details.reason === expected.reason, `INVALID_REQUEST reason is ${expected.reason}`);
  const serialized = JSON.stringify(error);
  for (const token of expected.tokens) {
    expect(record, serialized.includes(token), `INVALID_REQUEST diagnostic mentions ${token}`);
  }
}
