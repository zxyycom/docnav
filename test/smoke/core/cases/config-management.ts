import {
  createProject,
  writeJson,
  writeProjectConfig,
} from "../fixtures.ts";
import type { SmokeProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectProtocolFailure,
  expectReadableViewFieldValue,
  expectStderrEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import { exitCodes } from "../config.ts";

export function createConfigContextTasks() {
  return [
    // @case BB-CORE-CONFIG-001
    {
      id: "CORE-CONFIG-001",
      label: "CORE-CONFIG-001 config precedence and path context",
      run: testConfigPrecedenceAndPathContext
    }
  ];
}

export function createToolCommandTasks() {
  return [
    // @case BB-CORE-TOOLS-001
    {
      id: "CORE-TOOLS-001",
      label: "CORE-TOOLS-001 init version doctor and help commands",
      run: testInitVersionDoctorAndHelp
    }
  ];
}

async function testConfigPrecedenceAndPathContext() {
  const project = createProject("config-precedence");
  writeProjectConfig(project, {
    defaults: {
      adapter: "docnav-markdown",
      output: "readable-json"
    }
  });

  await assertUserPaginationConfigSet(project);
  await assertRemovedTextOutputFails(project);
  await assertConfigListPathContext(project, "docnav-markdown");
  await assertPaginationDisabledSuccess(project);
  await assertLegacyDefaultsLimitConfigFails();
}

async function assertUserPaginationConfigSet(project: SmokeProject) {
  const setLimit = await runCli("CORE-CONFIG-001 config set user defaults.pagination.limit", [
    "config",
    "set",
    "defaults.pagination.limit",
    "321",
    "--user"
  ], { project });
  expectExit(setLimit, 0);
  expectStderrEmpty(setLimit);
  const setLimitJson = parseJson(setLimit);
  expect(setLimit, setLimitJson.scope === "user", "user config set writes user scope");
  expect(setLimit, setLimitJson.value === 321, "user config set stores pagination limit");

  const setEnabled = await runCli("CORE-CONFIG-001 config set user defaults.pagination.enabled", [
    "config",
    "set",
    "defaults.pagination.enabled",
    "disabled",
    "--user"
  ], { project });
  expectExit(setEnabled, 0);
  expectStderrEmpty(setEnabled);
  const setEnabledJson = parseJson(setEnabled);
  expect(setEnabled, setEnabledJson.value === false, "user config set stores pagination enabled state");
}

async function assertRemovedTextOutputFails(project: SmokeProject) {
  const setRemovedOutput = await runCli("CORE-CONFIG-001 config set defaults.output text fails", [
    "config",
    "set",
    "defaults.output",
    "text"
  ], { project });
  expectExit(setRemovedOutput, exitCodes.input);
  expectStderrEmpty(setRemovedOutput);
  expectInvalidOutputModeErrorShape(setRemovedOutput);
}

function expectInvalidOutputModeErrorShape(record: CommandRecord) {
  expectStdoutIncludes(record, "\"$block\": \"/error\"");
  expectStdoutIncludes(record, "\"code\": \"INVALID_REQUEST\"");
  expectReadableViewFieldValue(record, record.stdout, "/details/field", "defaults.output");
  expectReadableViewFieldValue(record, record.stdout, "/details/received", "text");
  expectReadableViewFieldValue(record, record.stdout, "/details/accepted", [
    "readable-view",
    "readable-json",
    "protocol-json"
  ]);
  expectStdoutIncludes(record, "accepted values: readable-view, readable-json, protocol-json");
}

async function assertConfigListPathContext(project: SmokeProject, adapterId: string) {
  const list = await runCli("CORE-CONFIG-001 config list --path selects adapter and defaults", [
    "config",
    "list",
    "--path",
    project.normalRelPath,
    "--operation",
    "outline"
  ], { project });
  expectExit(list, 0);
  expectStderrEmpty(list);
  const listJson = parseJson(list);
  expect(list, valueFor(list, listJson, "defaults.output").value === "readable-json", "config list shows project output value");
  expect(
    list,
    valueFor(list, listJson, "defaults.pagination.enabled").value === false,
    "config list shows user pagination enabled value"
  );
  expect(
    list,
    valueFor(list, listJson, "defaults.pagination.limit").value === 321,
    "config list shows user pagination limit value"
  );
  const pathContext = expectJsonObject(list, listJson.path_context, "config list path_context is an object");
  const adapter = expectJsonObject(list, pathContext.adapter, "config list path_context.adapter is an object");
  const defaults = expectJsonObject(list, pathContext.defaults, "config list path_context.defaults is an object");
  const pagination = expectJsonObject(list, defaults.pagination, "config list pagination context is an object");
  const enabled = expectJsonObject(list, pagination.enabled, "config list pagination enabled context is an object");
  const limit = expectJsonObject(list, pagination.limit, "config list pagination limit context is an object");
  expect(list, adapter.selected === adapterId, "config list --path reports selected adapter");
  expect(list, enabled.value === false, "config list --path reports final pagination enabled state");
  expect(list, limit.value === 321, "config list --path reports final limit");
}

async function assertPaginationDisabledSuccess(project: SmokeProject) {
  const record = await runCli("CORE-CONFIG-001 config disabled pagination uses numeric limit only", [
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
}

async function assertLegacyDefaultsLimitConfigFails() {
  const project = createProject("legacy-defaults-limit", { config: false });
  writeJson(`${project.docnavDir}/docnav.json`, {
    defaults: {
      limit: 12
    }
  });

  const record = await runCli("CORE-CONFIG-001 legacy defaults.limit config fails", [
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

async function testInitVersionDoctorAndHelp() {
  const initProject = createProject("tool-init", { docnavDir: false, normalDocument: false });
  const init = await runCli("CORE-TOOLS-001 init creates project config", ["init"], { project: initProject });
  expectExit(init, 0);
  expectStderrEmpty(init);
  const initJson = parseJson(init);
  expect(init, initJson.created === true, "init creates config on first run");

  const version = await runCli("CORE-TOOLS-001 version prints crate version", ["version"], { project: initProject });
  expectExit(version, 0);
  expectStderrEmpty(version);
  expectStdoutIncludes(version, "docnav ");

  const help = await runCli("CORE-TOOLS-001 docnav outline help", ["outline", "--help"], { project: initProject });
  expectExit(help, 0);
  expectStderrEmpty(help);
  expectStdoutIncludes(help, "--output");
  expectStdoutIncludes(help, "--pagination");
  expectStdoutIncludes(help, "--limit");
  expectStdoutIncludes(help, "readable-view");
  expectStdoutIncludes(help, "readable-json");
  expectStdoutIncludes(help, "protocol-json");
  expect(help, !help.stdout.includes("text"), "outline help does not mention text output mode");

  const doctorProject = createProject("tool-doctor-static-registry");
  const doctor = await runCli("CORE-TOOLS-001 doctor reports static registry checks", ["doctor"], {
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

  // @case BB-CORE-ADAPTER-MGMT-001
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

function valueFor(record: CommandRecord, configListJson: JsonRecord, key: string): JsonRecord {
  const values = expectObjectArray(record, configListJson.values, "config list values are objects");
  const item = values.find((entry) => entry.key === key);
  return expectJsonObject(record, item, `config list includes ${key}`);
}
