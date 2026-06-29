import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeJson,
  writeProjectConfig,
  writeRegistry
} from "../fixtures.ts";
import type { FakeAdapter, SmokeProject } from "../fixtures.ts";
import { runCli } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
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
  const fake = createFakeAdapter(project, { id: "fake-config-context" });
  writeProjectConfig(project, {
    defaults: {
      adapter: fake.id,
      output: "readable-json"
    }
  });
  writeRegistry(project, [fake]);

  await assertUserPaginationConfigSet(project);
  await assertRemovedTextOutputFails(project);
  await assertConfigListPathContext(project, fake.id);
  await assertPaginationDisabledRequestFinalization(project, fake);
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

async function assertPaginationDisabledRequestFinalization(project: SmokeProject, adapter: FakeAdapter) {
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
  const invokeCall = [...readAdapterCalls(adapter)].reverse().find((call) => call.command === "invoke");
  const stdin = expectJsonObject(record, invokeCall?.stdin, "fake adapter invoke stdin is an object");
  const argumentsJson = expectJsonObject(record, stdin.arguments, "fake adapter invoke arguments is an object");
  expect(record, argumentsJson.limit === 4294967295, "disabled pagination finalizes limit to max positive integer");
  expect(record, argumentsJson.page === 1, "disabled pagination still sends page");
  expect(record, !Object.hasOwn(argumentsJson, "pagination"), "protocol request does not include pagination field");
}

async function assertLegacyDefaultsLimitConfigFails() {
  const project = createProject("legacy-defaults-limit", { config: false });
  writeJson(`${project.docnavDir}/docnav.json`, {
    defaults: {
      limit: 12
    }
  });

  const record = await runCli("CORE-CONFIG-001 legacy defaults.limit config fails", ["config", "list"], { project });
  expectExit(record, exitCodes.input);
  expectStderrEmpty(record);
  expectStdoutIncludes(record, "failed to parse");
  expectStdoutIncludes(record, "unknown field");
  expectStdoutIncludes(record, "limit");
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

  const doctorProject = createProject("tool-doctor-failing-check");
  const bad = createFakeAdapter(doctorProject, { id: "fake-doctor-invalid", mode: "manifest-invalid" });
  writeRegistry(doctorProject, [bad]);
  const doctor = await runCli("CORE-TOOLS-001 doctor reports checks and fails on bad manifest", ["doctor"], {
    project: doctorProject
  });
  expectExit(doctor, exitCodes.protocolOrAdapterProcess);
  const doctorJson = parseJson(doctor);
  const checks = expectObjectArray(doctor, doctorJson.checks, "doctor output contains checks array");
  expect(doctor, checks.some((check) => check.status === "fail"), "doctor reports failing check");
  expect(doctor, readAdapterCalls(bad).some((call) => call.command === "manifest"), "doctor validates adapter manifest");
}

function valueFor(record: CommandRecord, configListJson: JsonRecord, key: string): JsonRecord {
  const values = expectObjectArray(record, configListJson.values, "config list values are objects");
  const item = values.find((entry) => entry.key === key);
  return expectJsonObject(record, item, `config list includes ${key}`);
}
