import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeProjectConfig,
  writeRegistry
} from "../fixtures.ts";
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
    {
      id: "CORE-CONFIG-001",
      label: "CORE-CONFIG-001 config precedence and path context",
      run: testConfigPrecedenceAndPathContext
    }
  ];
}

export function createToolCommandTasks() {
  return [
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

  const setUser = await runCli("CORE-CONFIG-001 config set user defaults.limit_chars", [
    "config",
    "set",
    "defaults.limit_chars",
    "321",
    "--user"
  ], { project });
  expectExit(setUser, 0);
  expectStderrEmpty(setUser);
  const setUserJson = parseJson(setUser);
  expect(setUser, setUserJson.scope === "user", "user config set writes user scope");
  expect(setUser, setUserJson.value === 321, "user config set stores limit chars");

  const setRemovedOutput = await runCli("CORE-CONFIG-001 config set defaults.output text fails", [
    "config",
    "set",
    "defaults.output",
    "text"
  ], { project });
  expectExit(setRemovedOutput, exitCodes.input);
  expectStderrEmpty(setRemovedOutput);
  expectStdoutIncludes(setRemovedOutput, "\"$block\": \"/error\"");
  expectStdoutIncludes(setRemovedOutput, "\"code\": \"INVALID_REQUEST\"");
  expectReadableViewFieldValue(setRemovedOutput, setRemovedOutput.stdout, "/details/field", "defaults.output");
  expectReadableViewFieldValue(setRemovedOutput, setRemovedOutput.stdout, "/details/received", "text");
  expectReadableViewFieldValue(setRemovedOutput, setRemovedOutput.stdout, "/details/accepted", [
    "readable-view",
    "readable-json",
    "protocol-json"
  ]);
  expectStdoutIncludes(setRemovedOutput, "accepted values: readable-view, readable-json, protocol-json");

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
  expect(list, valueFor(list, listJson, "defaults.limit_chars").value === 321, "config list shows user limit value");
  const pathContext = expectJsonObject(list, listJson.path_context, "config list path_context is an object");
  const adapter = expectJsonObject(list, pathContext.adapter, "config list path_context.adapter is an object");
  const defaults = expectJsonObject(list, pathContext.defaults, "config list path_context.defaults is an object");
  const limitChars = expectJsonObject(list, defaults.limit_chars, "config list limit_chars context is an object");
  expect(list, adapter.selected === fake.id, "config list --path reports selected adapter");
  expect(list, limitChars.value === 321, "config list --path reports final limit");
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
  expectStdoutIncludes(help, "--limit-chars");
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
