import fs from "node:fs";
import path from "node:path";

import {
  configFixtureProject,
  createProject,
  mutableConfigFixtureProject,
  writeJson,
} from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolFailure,
  expectStderrEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import { exitCodes } from "../config.ts";
import {
  assertProjectNativeOptionConfigAffectsOutline,
  assertUserNativeOptionConfigRejectedForRead
} from "./config-native-options.ts";
import {
  testConfigPathFlagsSelectConfigTargets,
  testLegacyConfigSetRejectsSelectedConfigFiles
} from "./config-path-flags.ts";

export function createConfigContextTasks() {
  return [
    {
      id: "CORE-CONFIG-001",
      label: "CORE-CONFIG-001 config source inspection",
      run: testConfigSourceInspection
    },
    {
      id: "CORE-CONFIG-002",
      label: "CORE-CONFIG-002 removed readable-json config value rejected",
      run: testRemovedOutputConfigRejected
    },
    {
      id: "CORE-CONFIG-003",
      label: "CORE-CONFIG-003 legacy defaults.limit rejected",
      run: assertLegacyDefaultsLimitConfigFails
    },
    {
      id: "CORE-CONFIG-004",
      label: "CORE-CONFIG-004 native option config behavior",
      run: testNativeOptionConfigBehavior
    },
    {
      id: "CORE-CONFIG-005",
      label: "CORE-CONFIG-005 disabled pagination ignores numeric limit",
      run: testPaginationDisabledBehavior
    },
    {
      id: "CORE-CONFIG-PATH-001",
      label: "CORE-CONFIG-PATH-001 config path flags select config files",
      run: testConfigPathFlagsSelectConfigTargets
    },
    {
      id: "CORE-CONFIG-PATH-002",
      label: "CORE-CONFIG-PATH-002 removed config set rejects selected files",
      run: testLegacyConfigSetRejectsSelectedConfigFiles
    }
  ];
}

export function createToolCommandTasks() {
  return [
    {
      id: "CORE-TOOLS-001",
      label: "CORE-TOOLS-001 init creates project config",
      run: testInitCommand
    },
    {
      id: "CORE-TOOLS-002",
      label: "CORE-TOOLS-002 version reports crate version",
      run: testVersionCommand
    },
    {
      id: "CORE-TOOLS-003",
      label: "CORE-TOOLS-003 outline help documents output options",
      run: testOutlineHelp
    },
    {
      id: "CORE-ADAPTER-MGMT-001",
      label: "CORE-ADAPTER-MGMT-001 doctor and adapter list commands",
      run: testAdapterManagementCommands
    }
  ];
}

async function testConfigSourceInspection() {
  const project = configContextProject("config-inspection");
  await assertConfigInspectSourceAndFacts(project, "docnav-markdown");
}

async function testPaginationDisabledBehavior() {
  const project = configContextProject("pagination-disabled");
  await assertPaginationDisabledSuccess(project);
}

function configContextProject(name: string): SmokeProject {
  const project = mutableConfigFixtureProject("config-precedence-base", name);
  writeUserConfig(project, {
    defaults: {
      pagination: {
        enabled: false,
        limit: 321
      }
    }
  });
  return project;
}

async function testRemovedOutputConfigRejected() {
  const project = createProject("removed-output-mode", {
    config: {
      defaults: {
        output: "readable-json"
      }
    }
  });
  await assertRemovedReadableJsonOutputInspectDiagnostic(project);
}

async function testNativeOptionConfigBehavior() {
  await assertProjectNativeOptionConfigAffectsOutline();
  await assertUserNativeOptionConfigRejectedForRead();
}

async function assertConfigInspectSourceAndFacts(project: SmokeProject, adapterId: string) {
  const projectConfigPath = path.join(project.docnavDir, "docnav.json");
  const userConfigPath = path.join(project.root, ".user-config", "docnav.json");
  const projectConfigBefore = fs.readFileSync(projectConfigPath, "utf8");
  const userConfigBefore = fs.readFileSync(userConfigPath, "utf8");
  const record = await runCli("CORE-CONFIG-001 config inspect reports source facts", [
    "config",
    "inspect"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  const inspection = expectJsonObject(record, json.inspection, "config inspect reports inspection");
  const projectSource = sourceFor(record, inspection, "project");
  const userSource = sourceFor(record, inspection, "user");
  expect(record, projectSource.origin === "default", "config inspect reports default project config origin");
  expect(record, userSource.origin === "default", "config inspect reports default user config origin");
  expect(record, projectSource.load_state === "loaded", "config inspect loads project config");
  expect(record, userSource.load_state === "loaded", "config inspect loads user config");
  expect(record, expectObjectArray(record, projectSource.diagnostics, "project diagnostics are objects").length === 0, "project config has no inspect diagnostics");
  expect(record, expectObjectArray(record, userSource.diagnostics, "user diagnostics are objects").length === 0, "user config has no inspect diagnostics");
  expect(record, parameterFact(record, inspection, "docnav.defaults.adapter").value === adapterId, "config inspect reports adapter fact from project config");
  expect(record, parameterFact(record, inspection, "docnav.defaults.output").value === "readable-view", "config inspect reports output fact from project config");
  expect(record, parameterFact(record, inspection, "docnav.defaults.pagination.enabled").value === false, "config inspect reports pagination enabled fact from user config");
  expect(record, parameterFact(record, inspection, "docnav.defaults.pagination.limit").value === 321, "config inspect reports pagination limit fact from user config");
  expect(
    record,
    projectionHasPath(record, inspection, "defaults.pagination.limit"),
    "config inspect exposes config-source projection",
  );
  expect(record, fs.readFileSync(projectConfigPath, "utf8") === projectConfigBefore, "config inspect does not modify project config");
  expect(record, fs.readFileSync(userConfigPath, "utf8") === userConfigBefore, "config inspect does not modify user config");
}

async function assertRemovedReadableJsonOutputInspectDiagnostic(project: SmokeProject) {
  const record = await runCli("CORE-CONFIG-002 config inspect rejects removed readable-json", [
    "config",
    "inspect"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  const inspection = expectJsonObject(record, json.inspection, "config inspect reports inspection");
  const projectSource = sourceFor(record, inspection, "project");
  const diagnostics = expectObjectArray(record, projectSource.diagnostics, "project diagnostics are objects");
  const diagnostic = expectJsonObject(record, diagnostics[0], "defaults.output diagnostic is an object");
  expect(record, diagnostic.field === "defaults.output", "config inspect reports invalid defaults.output field");
  expect(record, diagnostic.reason === "enum_invalid", "config inspect reports invalid output enum reason");
}

async function assertPaginationDisabledSuccess(project: SmokeProject) {
  const record = await runCli("CORE-CONFIG-005 config disabled pagination uses numeric limit only", [
    "outline",
    project.normalRelPath,
    "--limit",
    "12",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  expect(record, json.ok === true, "disabled pagination still dispatches through built-in adapter");
  const result = expectJsonObject(record, json.result, "disabled pagination outline result is an object");
  const entries = expectObjectArray(record, result.entries, "disabled pagination outline entries are objects");
  expect(record, entries.length === 3, "disabled pagination returns the complete outline despite the numeric limit");
  expect(record, result.page === null, "disabled pagination has no continuation page");
}

async function assertLegacyDefaultsLimitConfigFails() {
  const project = configFixtureProject("legacy-defaults-limit");

  const record = await runCli("CORE-CONFIG-003 legacy defaults.limit config fails", [
    "outline",
    project.normalRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(record, exitCodes.input);
  expectStderrEmpty(record);
  expectUnknownConfigFieldErrorShape(record, "defaults.limit", "project");
}

function expectUnknownConfigFieldErrorShape(record: CommandRecord, field: string, sourceLevel: string) {
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, "outline", "INVALID_REQUEST");
  const details = expectJsonObject(record, error.details, "unknown config error details are an object");
  expect(record, error.owner === "docnav_config", "unknown config field owner is config boundary");
  expect(record, details.field === field, "unknown config field reports field path");
  expect(record, details.reason === "unknown_config_field", "unknown config field reports stable reason");

  const location = expectJsonObject(record, error.location, "unknown config error has location");
  expect(record, location.field === field, "unknown config location reports field path");
  expect(record, typeof location.config_path === "string", "unknown config location reports config path");

  const issues = expectObjectArray(record, details.config_issues, "unknown config error has config issues");
  const issue = expectJsonObject(record, issues[0], "unknown config issue is an object");
  expect(record, issue.field === field, "unknown config issue reports field path");
  expect(record, issue.source_level === sourceLevel, "unknown config issue reports source level");
  expect(record, issue.path_origin === "default", "unknown config issue reports path origin");
  expect(record, typeof issue.path === "string", "unknown config issue reports config path");
  expect(record, issue.reason_code === "unknown_config_field", "unknown config issue reports reason code");
}

function writeUserConfig(project: SmokeProject, config: unknown) {
  writeJson(path.join(project.root, ".user-config", "docnav.json"), config);
}

function sourceFor(record: CommandRecord, inspection: JsonRecord, scope: string): JsonRecord {
  const sources = expectObjectArray(record, inspection.sources, "config inspect sources are objects");
  const source = sources.find((entry) => entry.scope === scope);
  return expectJsonObject(record, source, `config inspect includes ${scope} source`);
}

function parameterFact(record: CommandRecord, inspection: JsonRecord, identity: string): JsonRecord {
  const facts = expectObjectArray(record, inspection.parameter_facts, "config inspect parameter facts are objects");
  const fact = facts.find((entry) => entry.identity === identity);
  return expectJsonObject(record, fact, `config inspect includes ${identity}`);
}

function projectionHasPath(record: CommandRecord, inspection: JsonRecord, fieldPath: string): boolean {
  const projection = expectObjectArray(record, inspection.config_source_projection, "config inspect projection fields are objects");
  return projection.some((field) => field.path === fieldPath);
}

async function testInitCommand() {
  const initProject = createProject("tool-init", { docnavDir: false, normalDocument: false });
  const init = await runCli("CORE-TOOLS-001 init creates project config", ["init"], { project: initProject });
  expectExit(init, 0);
  expectStderrEmpty(init);
  const initJson = parseJson(init);
  expect(init, initJson.created === true, "init creates config on first run");
}

async function testVersionCommand() {
  const project = createProject("tool-version", { docnavDir: false, normalDocument: false });
  const version = await runCli("CORE-TOOLS-002 version prints crate version", ["version"], { project });
  expectExit(version, 0);
  expectStderrEmpty(version);
  expectStdoutIncludes(version, "docnav ");
}

async function testOutlineHelp() {
  const project = createProject("tool-outline-help", { docnavDir: false, normalDocument: false });
  const help = await runCli("CORE-TOOLS-003 docnav outline help", ["outline", "--help"], { project });
  expectExit(help, 0);
  expectStderrEmpty(help);
  expectStdoutIncludes(help, "--output");
  expectStdoutIncludes(help, "--pagination");
  expectStdoutIncludes(help, "--limit");
  expectStdoutIncludes(help, "--output <mode>");
  expectStdoutIncludes(help, "possible values: readable-view, protocol-json");
  expectStdoutIncludes(help, "default: readable-view");
}

async function testAdapterManagementCommands() {
  const doctorProject = configFixtureProject("empty");
  const doctor = await runCli("CORE-ADAPTER-MGMT-001 doctor reports static registry checks", ["doctor"], {
    project: doctorProject
  });
  expectExit(doctor, 0);
  const doctorJson = parseJson(doctor);
  const checks = expectObjectArray(doctor, doctorJson.checks, "doctor output contains checks array");
  expect(
    doctor,
    checks.some((check) => check.name === "core_static_adapter_registry" && check.status === "pass"),
    "doctor reports static adapter registry check"
  );
  expect(
    doctor,
    checks.some((check) => check.name === "adapter_layer" && check.adapter_id === "docnav-markdown"),
    "doctor reports built-in markdown adapter layer"
  );

  const adapterList = await runCli("CORE-ADAPTER-MGMT-001 adapter list reports built-in registry", ["adapter", "list"], {
    project: doctorProject
  });
  expectExit(adapterList, 0);
  const adapterListJson = parseJson(adapterList);
  const adapters = expectObjectArray(adapterList, adapterListJson.adapters, "adapter list returns adapters array");
  expect(
    adapterList,
    adapters.some((adapter) => adapter.id === "docnav-markdown"),
    "adapter list includes built-in markdown adapter"
  );
}
