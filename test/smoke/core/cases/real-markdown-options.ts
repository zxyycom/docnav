import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoJsonPayloadInStderr,
  expectObjectArray,
  expectProtocolFailure,
  parseJson
} from "../assertions.ts";
import { exitCodes } from "../config.ts";
import { createProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";

export async function testMaxHeadingLevelCliOption() {
  const project = createProject("real-markdown-max-heading-level-option");
  const record = await runCli("CORE-MD-OPTIONS-001 outline max heading level native option", [
    "outline",
    project.normalRelPath,
    "--max-heading-level",
    "1",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const result = expectJsonObject(record, json.result, "outline result is an object");
  const entries = expectObjectArray(record, result.entries, "outline entries are objects");
  expect(record, entries.length === 1, "max heading level filters nested Markdown headings");
}

export async function testMaxHeadingLevelAdapterValidation() {
  const project = createProject("real-markdown-max-heading-level-validation");
  const record = await runCli("CORE-MD-OPTIONS-002 outline invalid max heading level is adapter-owned", [
    "outline",
    project.normalRelPath,
    "--max-heading-level",
    "7",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  expectNoJsonPayloadInStderr(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, "outline", "INVALID_REQUEST");
  const details = expectJsonObject(record, error.details, "protocol error details is an object");
  expect(
    record,
    details.field === "arguments.options.max_heading_level",
    "adapter option validation reports operation argument field"
  );
  expect(
    record,
    details.reason === "range_invalid",
    "adapter option validation reports Markdown range reason"
  );
  expect(
    record,
    error.owner === "adapter_options",
    "adapter option validation is adapter-owned"
  );
  expect(
    record,
    error.received === "7",
    "adapter option validation reports received value"
  );
  expect(
    record,
    error.expected === "integer in range 1..6",
    "adapter option validation reports expected value"
  );
  const issues = expectObjectArray(
    record,
    details.option_issues,
    "adapter option validation reports option issues"
  );
  const issue = expectJsonObject(
    record,
    issues[0],
    "adapter option validation issue is an object"
  );
  expect(
    record,
    issue.owner === "docnav-markdown",
    "adapter option validation issue reports owner"
  );
  expect(
    record,
    issue.namespace === "options",
    "adapter option validation issue reports namespace"
  );
  expect(
    record,
    issue.key === "max_heading_level",
    "adapter option validation issue reports key"
  );
  expect(
    record,
    issue.reason_code === "range_invalid",
    "adapter option validation issue reports range reason code"
  );
}
