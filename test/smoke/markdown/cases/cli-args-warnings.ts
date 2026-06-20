import { exitCodes } from "../config.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoWarningsField,
  expectObjectArray,
  expectProtocolSuccess,
  expectStderrEmpty,
  expectStderrWarning,
  expectStructuredWarning,
  expectStdoutIncludes,
  parseJson
} from "../assertions.ts";

type HelpOutputModeCompatibility = {
  modes: readonly [string, string, string];
  textModeAssertion: string;
};

export async function assertHelpOutputModeCompatibility(options: HelpOutputModeCompatibility) {
  const outlineHelp = await runCli("MD-WARN-001 docnav-markdown outline help", ["outline", "--help"]);
  expectExit(outlineHelp, exitCodes.success);
  expectStderrEmpty(outlineHelp);
  expectStdoutIncludes(outlineHelp, "--max-heading-level");
  expectStdoutIncludes(outlineHelp, "--output");
  // 3.5: help only lists three final output modes for document operations.
  expectStdoutIncludes(outlineHelp, options.modes[0]);
  expectStdoutIncludes(outlineHelp, options.modes[1]);
  expectStdoutIncludes(outlineHelp, options.modes[2]);
  expect(outlineHelp, !outlineHelp.stdout.includes("text"), options.textModeAssertion);
}

export async function assertReadableJsonCompatibilityWarnings(normal: string, ref: string) {
  const readable = await runCli("MD-WARN-001 outline future readable-json warning", [
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
  const readableWarnings = expectObjectArray(readable, readableJson.warnings, "readable warnings are objects");
  expectStructuredWarning(readable, readableWarnings[0], ["--future"], "future flag");

  const unused = await runCli("MD-WARN-001 read unused known flag readable-json warning", [
    "read",
    normal,
    "--ref",
    String(ref),
    "--max-heading-level",
    "3",
    "--output",
    "readable-json"
  ]);
  expectExit(unused, exitCodes.success);
  expectStderrEmpty(unused);
  const unusedJson = parseJson(unused);
  validateSchema(unused, "readableRead", unusedJson);
  const unusedWarnings = expectObjectArray(unused, unusedJson.warnings, "unused flag warnings are objects");
  expectStructuredWarning(unused, unusedWarnings[0], ["--max-heading-level", "3"], "unused native flag");
}

export async function assertProtocolJsonCompatibilityWarning(normal: string) {
  const protocol = await runCli("MD-WARN-001 outline future flag protocol-json stderr warning", [
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
