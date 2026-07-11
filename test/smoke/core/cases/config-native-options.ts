import path from "node:path";

import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.ts";
import { exitCodes } from "../config.ts";
import { configFixtureProject, mutableConfigFixtureProject, writeJson } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";

export async function assertProjectNativeOptionConfigAffectsOutline() {
  const project = configFixtureProject("project-native-option-outline");

  const record = await runCli("CORE-CONFIG-004 project config max heading level affects outline", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expect(record, json.ok === true, "project native option config dispatches successfully");
  const result = expectJsonObject(record, json.result, "outline result is an object");
  const entries = expectObjectArray(record, result.entries, "outline entries are objects");
  expect(record, entries.length === 1, "project config max heading level hides nested headings");
  expect(record, entries[0]?.label === "Guide", "project config max heading level preserves the top heading");
}

export async function assertUserNativeOptionConfigRejectedForRead() {
  const project = mutableConfigFixtureProject("empty", "user-native-option-read");
  const userConfigPath = path.join(project.root, ".user-config", "docnav.json");
  writeJson(userConfigPath, {
    options: {
      "docnav-markdown": {
        max_heading_level: 1
      }
    }
  });

  const record = await runCli("CORE-CONFIG-004 read rejects config max heading level", [
    "read",
    project.normalRelPath,
    "--ref",
    "doc:full",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  expectStderrEmpty(record);
  expectUnsupportedNativeOptionErrorShape(
    record,
    {
      operation: "read",
      source: "user",
      received: "1",
      expected: "no native options",
      configPath: userConfigPath
    }
  );
}

function expectUnsupportedNativeOptionErrorShape(
  record: CommandRecord,
  expectation: {
    configPath: string;
    expected: string;
    operation: string;
    received: string;
    source: string;
  }
) {
  const { configPath, expected, operation, received, source } = expectation;
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, operation, "INVALID_REQUEST");
  const details = expectJsonObject(record, error.details, "unsupported native option details are an object");
  expect(record, error.owner === "adapter_options", "unsupported native option owner is adapter options");
  expect(record, details.field === "arguments.options.max_heading_level", "unsupported native option reports field");
  expect(record, details.reason === "unsupported", "unsupported native option reports stable reason");
  expect(record, error.received === received, "unsupported native option reports received value");
  expect(record, error.expected === expected, "unsupported native option reports expected value");

  const configIssues = expectObjectArray(record, details.config_issues, "unsupported native option has config issues");
  const configIssue = expectJsonObject(record, configIssues[0], "unsupported native option config issue is an object");
  expect(record, configIssue.field === "options.docnav-markdown.max_heading_level", "config issue reports source field");
  expect(record, configIssue.source_level === source, "config issue reports source level");
  expect(record, configIssue.path === configPath, "config issue reports selected config path");
  expect(record, configIssue.path_origin === "default", "config issue reports default path origin");
  expect(record, configIssue.reason_code === "unsupported", "config issue reports stable reason code");

  const issues = expectObjectArray(record, details.option_issues, "unsupported native option has option issues");
  const issue = expectJsonObject(record, issues[0], "unsupported native option issue is an object");
  expect(record, issue.owner === "docnav-markdown", "unsupported native option issue reports owner");
  expect(record, issue.namespace === "options", "unsupported native option issue reports namespace");
  expect(record, issue.key === "max_heading_level", "unsupported native option issue reports key");
  expect(record, issue.source === source, "unsupported native option issue reports source");
  expect(record, issue.reason_code === "unsupported", "unsupported native option issue reports reason code");
  const issueLocation = expectJsonObject(record, issue.location, "unsupported native option issue has location");
  expect(
    record,
    issueLocation.field === "arguments.options.max_heading_level",
    "unsupported native option issue reports field"
  );
  expect(record, issue.received === received, "unsupported native option issue reports received value");
  expect(record, issue.expected === expected, "unsupported native option issue reports expected value");
}
