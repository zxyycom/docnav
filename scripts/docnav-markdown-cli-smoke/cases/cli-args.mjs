import { exitCodes } from "../config.mjs";
import { fixture, getNormalRef } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
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
  expectStdoutWarning,
  parseJson
} from "../assertions.mjs";

export function testCliArgumentFailures() {
  const normal = fixture("normal.md");
  const cases = [
    {
      name: "outline missing path before flag",
      args: ["outline", "--output", "text"],
      stderr: "outline requires <path>"
    },
    {
      name: "read missing required --ref",
      args: ["read", normal],
      stderr: "read requires --ref <ref>"
    },
    {
      name: "find missing required --query",
      args: ["find", normal],
      stderr: "find requires --query <text>"
    },
    {
      name: "outline missing value --page",
      args: ["outline", normal, "--page"],
      stderr: "--page requires a value"
    },
    {
      name: "outline invalid --page zero",
      args: ["outline", normal, "--page", "0"],
      stderr: "--page must be a positive integer"
    },
    {
      name: "outline invalid --max-heading-level too high",
      args: ["outline", normal, "--max-heading-level", "7"],
      stderr: "--max-heading-level must be an integer from 1 to 6"
    },
    {
      name: "outline invalid --output",
      args: ["outline", normal, "--output", "bogus"],
      stderr: 'invalid --output "bogus"'
    },
    {
      name: "read empty --ref",
      args: ["read", normal, "--ref", ""],
      stderr: "--ref must not be empty"
    },
    {
      name: "find empty --query",
      args: ["find", normal, "--query", ""],
      stderr: "--query must not be empty"
    },
    {
      name: "manifest protocol-only --output text",
      args: ["manifest", "--output", "text"],
      stderr: "only --output protocol-json is supported for this command"
    },
    {
      name: "probe missing path before flag",
      args: ["probe", "--output", "protocol-json"],
      stderr: "probe requires <path>"
    },
    {
      name: "probe protocol-only --output text",
      args: ["probe", normal, "--output", "text"],
      stderr: "only --output protocol-json is supported for this command"
    },
    {
      name: "invoke positional unexpected",
      args: ["invoke", "unexpected"],
      stderr: "invoke does not accept positional arguments"
    }
  ];

  for (const item of cases) {
    const record = runCli(item.name, item.args);
    expectExit(record, exitCodes.input);
    expectStdoutEmpty(record);
    expectStderrIncludes(record, item.stderr);
  }
}

export function testCliArgumentCompatibilityWarnings() {
  const normal = fixture("normal.md");
  const ref = getNormalRef();

  const rootHelp = runCli("docnav-markdown root help", ["--help"]);
  expectExit(rootHelp, exitCodes.success);
  expectStderrEmpty(rootHelp);
  expectStdoutIncludes(rootHelp, "Usage:");
  expectStdoutIncludes(rootHelp, "outline");

  const outlineHelp = runCli("docnav-markdown outline help", ["outline", "--help"]);
  expectExit(outlineHelp, exitCodes.success);
  expectStderrEmpty(outlineHelp);
  expectStdoutIncludes(outlineHelp, "--max-heading-level");
  expectStdoutIncludes(outlineHelp, "--output");

  const text = runCli("outline unknown equals flag text warning", [
    "outline",
    normal,
    "--unknown=value",
    "--output",
    "text"
  ]);
  expectExit(text, exitCodes.success);
  expectStderrEmpty(text);
  expectStdoutIncludes(text, "page:");
  expectStdoutWarning(text, ["--unknown=value"]);

  const unknownBeforePath = runCli("outline unknown before path readable-json warning", [
    "outline",
    "--future",
    normal,
    "--output",
    "readable-json"
  ]);
  expectExit(unknownBeforePath, exitCodes.success);
  expectStderrEmpty(unknownBeforePath);
  const unknownBeforePathJson = parseJson(unknownBeforePath);
  validateSchema(unknownBeforePath, "readableOutline", unknownBeforePathJson);
  expectStructuredWarning(
    unknownBeforePath,
    unknownBeforePathJson.warnings?.[0],
    ["--future"],
    "unknown flag"
  );

  const readable = runCli("outline unknown and extra readable-json warnings", [
    "outline",
    normal,
    "--future",
    "extra",
    "--output",
    "readable-json"
  ]);
  expectExit(readable, exitCodes.success);
  expectStderrEmpty(readable);
  const readableJson = parseJson(readable);
  validateSchema(readable, "readableOutline", readableJson);
  expectStructuredWarning(readable, readableJson.warnings?.[0], ["--future"], "unknown flag");
  expectStructuredWarning(readable, readableJson.warnings?.[1], ["extra"], "extra positional");

  const unused = runCli("read unused known flag readable-json warning", [
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
  expectStructuredWarning(
    unused,
    unusedJson.warnings?.[0],
    ["--max-heading-level", "3"],
    "unused native flag"
  );

  const unusedInvalid = runCli("info unused invalid limit readable-json warning", [
    "info",
    normal,
    "--limit-chars",
    "nope",
    "--output",
    "readable-json"
  ]);
  expectExit(unusedInvalid, exitCodes.success);
  expectStderrEmpty(unusedInvalid);
  const unusedInvalidJson = parseJson(unusedInvalid);
  validateSchema(unusedInvalid, "readableInfo", unusedInvalidJson);
  expectStructuredWarning(
    unusedInvalid,
    unusedInvalidJson.warnings?.[0],
    ["--limit-chars", "nope"],
    "unused known invalid flag"
  );

  const protocol = runCli("outline unknown flag protocol-json stderr warning", [
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

  const manifest = runCli("manifest unknown flag stderr warning", [
    "manifest",
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(manifest, exitCodes.success);
  expectStderrWarning(manifest, ["--future"]);
  expectNoJsonPayloadInStderr(manifest);
  const manifestJson = parseJson(manifest);
  validateSchema(manifest, "manifest", manifestJson);
  expectNoWarningsField(manifest, manifestJson, "manifest stdout");

  const probe = runCli("probe unknown flag stderr warning", [
    "probe",
    normal,
    "--future",
    "--output",
    "protocol-json"
  ]);
  expectExit(probe, exitCodes.success);
  expectStderrWarning(probe, ["--future"]);
  expectNoJsonPayloadInStderr(probe);
  const probeJson = parseJson(probe);
  validateSchema(probe, "probe", probeJson);
  expectNoWarningsField(probe, probeJson, "probe stdout");

  const refLikeFlag = runCli("read ref value looks like flag", [
    "read",
    normal,
    "--ref",
    "--future-value",
    "--output",
    "text"
  ]);
  expectExit(refLikeFlag, exitCodes.documentRefFormat);
  expectStderrEmpty(refLikeFlag);
  expectStdoutIncludes(refLikeFlag, "REF_NOT_FOUND");
  expectStdoutIncludes(refLikeFlag, "ref=--future-value");
}
