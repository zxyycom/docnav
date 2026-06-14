import {
  createFakeAdapter,
  createProject,
  readAdapterCalls,
  writeProjectConfig,
  writeRegistry
} from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolSuccess,
  expectReadableViewFieldValue,
  expectStderrEmpty,
  expectStderrWarning,
  expectStdoutIncludes,
  expectStructuredWarning,
  parseJson
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

export function testConfigContextAndCompatibility() {
  testProjectAndUserConfig();
  testConfigListPath();
  testInitVersionDoctor();
  testHelpCommands();
  testCompatibilityWarnings();
}

function testProjectAndUserConfig() {
  const project = createProject("config-project-user");

  const setProject = runCli("config set project defaults.output", [
    "config",
    "set",
    "defaults.output",
    "readable-json"
  ], { project });
  expectExit(setProject, 0);
  expectStderrEmpty(setProject);
  const setProjectJson = parseJson(setProject);
  expect(setProject, setProjectJson.scope === "project", "project config set writes project scope");
  expect(setProject, setProjectJson.value === "readable-json", "project config set stores output");

  const setRemovedOutput = runCli("config set project defaults.output text fails", [
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

  const setUser = runCli("config set user defaults.limit_chars", [
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

  const getUser = runCli("config get user defaults.limit_chars", [
    "config",
    "get",
    "defaults.limit_chars",
    "--user"
  ], { project });
  expectExit(getUser, 0);
  expectStderrEmpty(getUser);
  const getUserJson = parseJson(getUser);
  expect(getUser, getUserJson.value === 321, "config get reads user limit value");
  expect(getUser, getUserJson.source === "user", "config get reports user scope");

  const getUnsupported = runCli("config get unsupported key fails", [
    "config",
    "get",
    "defaults.unknown"
  ], { project });
  expectExit(getUnsupported, exitCodes.input);
  expectStderrEmpty(getUnsupported);
  expectStdoutIncludes(getUnsupported, "\"$block\": \"/error\"");
  expectStdoutIncludes(getUnsupported, "\"code\": \"INVALID_REQUEST\"");
  expectStdoutIncludes(getUnsupported, "unsupported docnav config key");

  const list = runCli("config list effective values", ["config", "list"], { project });
  expectExit(list, 0);
  expectStderrEmpty(list);
  const listJson = parseJson(list);
  const output = valueFor(listJson, "defaults.output");
  const limitChars = valueFor(listJson, "defaults.limit_chars");
  expect(list, output.value === "readable-json", "config list shows project output value");
  expect(list, output.source === "project", "config list shows project output source");
  expect(list, limitChars.value === 321, "config list shows user limit value");
  expect(list, limitChars.source === "user", "config list shows user limit source");

  const unsetProject = runCli("config unset project defaults.output", [
    "config",
    "unset",
    "defaults.output"
  ], { project });
  expectExit(unsetProject, 0);
  expectStderrEmpty(unsetProject);
  const unsetProjectJson = parseJson(unsetProject);
  expect(unsetProject, unsetProjectJson.scope === "project", "config unset writes project scope");
}

function testConfigListPath() {
  const project = createProject("config-list-path");
  const fake = createFakeAdapter(project, { id: "fake-config-context" });
  writeProjectConfig(project, {
    defaults: {
      adapter: fake.id,
      limit_chars: 444
    }
  });
  writeRegistry(project, [fake]);

  const record = runCli("config list --path selects adapter", [
    "config",
    "list",
    "--path",
    project.normalRelPath,
    "--operation",
    "outline"
  ], { project });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  expect(record, json.path_context?.adapter?.selected === fake.id, "config list --path reports selected adapter");
  expect(record, json.path_context?.defaults?.limit_chars?.value === 444, "config list --path reports final limit");
  expect(record, json.path_context?.defaults?.limit_chars?.source === "project", "config list --path reports limit source");
}

function testInitVersionDoctor() {
  const initProject = createProject("init-command", { docnavDir: false, normalDocument: false });
  const init = runCli("init creates project config", ["init"], { project: initProject });
  expectExit(init, 0);
  expectStderrEmpty(init);
  const initJson = parseJson(init);
  expect(init, initJson.created === true, "init creates config on first run");

  const initAgain = runCli("init is idempotent", ["init"], { project: initProject });
  expectExit(initAgain, 0);
  expectStderrEmpty(initAgain);
  const initAgainJson = parseJson(initAgain);
  expect(initAgain, initAgainJson.created === false, "init does not overwrite existing config");

  const version = runCli("version prints crate version", ["version"], { project: initProject });
  expectExit(version, 0);
  expectStderrEmpty(version);
  expectStdoutIncludes(version, "docnav ");

  const doctorProject = createProject("doctor-failing-check");
  const bad = createFakeAdapter(doctorProject, { id: "fake-doctor-invalid", mode: "manifest-invalid" });
  writeRegistry(doctorProject, [bad]);
  const doctor = runCli("doctor reports checks and fails on bad manifest", ["doctor"], { project: doctorProject });
  expectExit(doctor, exitCodes.protocolOrAdapterProcess);
  const doctorJson = parseJson(doctor);
  expect(doctor, Array.isArray(doctorJson.checks), "doctor output contains checks array");
  expect(doctor, doctorJson.checks.some((check) => check.status === "fail"), "doctor reports failing check");
}

function testHelpCommands() {
  const project = createProject("help-commands", { normalDocument: false });
  const root = runCli("docnav root help", ["--help"], { project });
  expectExit(root, 0);
  expectStderrEmpty(root);
  expectStdoutIncludes(root, "Usage:");
  expectStdoutIncludes(root, "outline");
  expectStdoutIncludes(root, "config");

  const outline = runCli("docnav outline help", ["outline", "--help"], { project });
  expectExit(outline, 0);
  expectStderrEmpty(outline);
  expectStdoutIncludes(outline, "--output");
  expectStdoutIncludes(outline, "--limit-chars");
  expectStdoutIncludes(outline, "readable-view");
  expectStdoutIncludes(outline, "readable-json");
  expectStdoutIncludes(outline, "protocol-json");
  expect(outline, !outline.stdout.includes("text"), "outline help does not mention text output mode");

  const config = runCli("docnav config help", ["config", "--help"], { project });
  expectExit(config, 0);
  expectStderrEmpty(config);
  expectStdoutIncludes(config, "get");
  expectStdoutIncludes(config, "set");

  const configGet = runCli("docnav config get help", ["config", "get", "--help"], { project });
  expectExit(configGet, 0);
  expectStderrEmpty(configGet);
  expectStdoutIncludes(configGet, "--user");
  expectStdoutIncludes(configGet, "key");
}

function testCompatibilityWarnings() {
  const project = createProject("compatibility-warnings");
  const fake = createFakeAdapter(project, { id: "fake-compat" });
  writeRegistry(project, [fake]);

  const unknownBeforePath = runCli("unknown flag before path still succeeds", [
    "outline",
    "--future",
    project.normalRelPath,
    "--output",
    "readable-json"
  ], { project });
  expectExit(unknownBeforePath, 0);
  expectStderrEmpty(unknownBeforePath);
  const unknownBeforePathJson = parseJson(unknownBeforePath);
  validateSchema(unknownBeforePath, "readableOutline", unknownBeforePathJson);
  expectStructuredWarning(unknownBeforePath, unknownBeforePathJson.warnings?.[0], ["--future"], "unknown flag");

  const readable = runCli("readable-json unknown flag and extra positional warnings", [
    "outline",
    project.normalRelPath,
    "--future",
    "extra",
    "--output",
    "readable-json"
  ], { project });
  expectExit(readable, 0);
  expectStderrEmpty(readable);
  const readableJson = parseJson(readable);
  validateSchema(readable, "readableOutline", readableJson);
  expectNoProtocolEnvelope(readable, readableJson);
  expectStructuredWarning(readable, readableJson.warnings?.[0], ["--future"], "unknown flag");
  expectStructuredWarning(readable, readableJson.warnings?.[1], ["extra"], "extra positional");

  const unused = runCli("unused known value flag warning", [
    "info",
    project.normalRelPath,
    "--page",
    "9",
    "--output",
    "readable-json"
  ], { project });
  expectExit(unused, 0);
  expectStderrEmpty(unused);
  const unusedJson = parseJson(unused);
  validateSchema(unused, "readableInfo", unusedJson);
  expectStructuredWarning(unused, unusedJson.warnings?.[0], ["--page", "9"], "unused known flag");
  const infoInvoke = readAdapterCalls(fake).findLast((call) => call.command === "invoke" && call.stdin?.operation === "info");
  expect(unused, infoInvoke && !Object.hasOwn(infoInvoke.stdin.arguments, "page"), "info invoke request omits page");
  expect(unused, infoInvoke && !Object.hasOwn(infoInvoke.stdin.arguments, "limit_chars"), "info invoke request omits limit_chars");

  const unusedInvalid = runCli("unused known invalid value warning", [
    "info",
    project.normalRelPath,
    "--limit-chars",
    "nope",
    "--output",
    "readable-json"
  ], { project });
  expectExit(unusedInvalid, 0);
  expectStderrEmpty(unusedInvalid);
  const unusedInvalidJson = parseJson(unusedInvalid);
  validateSchema(unusedInvalid, "readableInfo", unusedInvalidJson);
  expectStructuredWarning(unusedInvalid, unusedInvalidJson.warnings?.[0], ["--limit-chars", "nope"], "unused known invalid flag");

  const protocol = runCli("protocol-json warning goes to stderr", [
    "outline",
    project.normalRelPath,
    "--future",
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, 0);
  expectStderrWarning(protocol, ["--future"]);
  expectNoJsonPayloadInStderr(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "outline");
}

function valueFor(configListJson, key) {
  return configListJson.values.find((item) => item.key === key);
}
