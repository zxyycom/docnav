import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeProjectConfig,
  writeRegistry
} from "../fixtures.mjs";
import { runCli } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectReadableViewFieldValue,
  expectStderrEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

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
  expect(list, valueFor(listJson, "defaults.output").value === "readable-json", "config list shows project output value");
  expect(list, valueFor(listJson, "defaults.limit_chars").value === 321, "config list shows user limit value");
  expect(list, listJson.path_context?.adapter?.selected === fake.id, "config list --path reports selected adapter");
  expect(list, listJson.path_context?.defaults?.limit_chars?.value === 321, "config list --path reports final limit");
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
  expect(doctor, Array.isArray(doctorJson.checks), "doctor output contains checks array");
  expect(doctor, doctorJson.checks.some((check) => check.status === "fail"), "doctor reports failing check");
  expect(doctor, readAdapterCalls(bad).some((call) => call.command === "manifest"), "doctor validates adapter manifest");
}

function valueFor(configListJson, key) {
  return configListJson.values.find((item) => item.key === key);
}
