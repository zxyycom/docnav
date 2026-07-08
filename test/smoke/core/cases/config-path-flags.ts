import {
  configFixturePath,
  copyConfigFixtureToProject,
  createProject,
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectStderrEmpty,
  parseJson,
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";
import { toSlashPath } from "../../../../scripts/tools/foundation/src/path.ts";

export async function testConfigPathFlagsSelectConfigTargets() {
  const project = createProject("config-path-flags", {
    config: {
      options: {
        max_heading_level: 3,
      },
    },
  });
  const readOnlyProjectConfig = configFixturePath("project-native-option-outline");
  const readOnlyUserConfig = configFixturePath("empty");

  const outline = await runCli(
    "CORE-CONFIG-PATH-001 outline uses explicit read-only config fixture",
    [
      "outline",
      project.normalRelPath,
      "--project-config",
      readOnlyProjectConfig,
      "--user-config",
      readOnlyUserConfig,
      "--output",
      "protocol-json",
    ],
    { project },
  );
  expectExit(outline, 0);
  expectStderrEmpty(outline);
  const outlineJson = parseJson(outline);
  validateSchema(outline, "protocolResponse", outlineJson);
  const result = expectJsonObject(outline, outlineJson.result, "outline result is an object");
  const entries = expectObjectArray(outline, result.entries, "outline entries are objects");
  expect(outline, entries.length === 1, "explicit project config fixture filters nested headings");
  expect(outline, entries[0]?.label === "Guide", "explicit project config fixture selects top heading");

  const mutableProjectConfig = copyConfigFixtureToProject(project, "empty", "selected/project.json");
  const mutableUserConfig = copyConfigFixtureToProject(project, "empty", "selected/user.json");
  const setProject = await runCli(
    "CORE-CONFIG-PATH-001 config set writes selected project config",
    [
      "config",
      "set",
      "defaults.output",
      "readable-json",
      "--project-config",
      mutableProjectConfig,
      "--user-config",
      mutableUserConfig,
    ],
    { project },
  );
  expectExit(setProject, 0);
  expectStderrEmpty(setProject);
  const setProjectJson = parseJson(setProject);
  expect(
    setProject,
    setProjectJson.path === toSlashPath(mutableProjectConfig),
    "config set writes selected project config path",
  );

  const setUser = await runCli(
    "CORE-CONFIG-PATH-001 config set --user writes selected user config",
    [
      "config",
      "set",
      "defaults.pagination.limit",
      "321",
      "--user",
      "--project-config",
      mutableProjectConfig,
      "--user-config",
      mutableUserConfig,
    ],
    { project },
  );
  expectExit(setUser, 0);
  expectStderrEmpty(setUser);
  const setUserJson = parseJson(setUser);
  expect(
    setUser,
    setUserJson.path === toSlashPath(mutableUserConfig),
    "config set --user writes selected user config path",
  );

  const list = await runCli(
    "CORE-CONFIG-PATH-001 config list uses selected config files",
    [
      "config",
      "list",
      "--path",
      project.normalRelPath,
      "--operation",
      "outline",
      "--project-config",
      mutableProjectConfig,
      "--user-config",
      mutableUserConfig,
    ],
    { project },
  );
  expectExit(list, 0);
  expectStderrEmpty(list);
  const listJson = parseJson(list);
  expect(list, listJson.project_config === toSlashPath(mutableProjectConfig), "config list reports selected project config");
  expect(list, listJson.user_config === toSlashPath(mutableUserConfig), "config list reports selected user config");
  expect(list, valueFor(list, listJson, "defaults.output").value === "readable-json", "config list reads selected project config");
  expect(list, valueFor(list, listJson, "defaults.pagination.limit").value === 321, "config list reads selected user config");
  const pathContext = expectJsonObject(list, listJson.path_context, "config list path_context is an object");
  const defaults = expectJsonObject(list, pathContext.defaults, "selected path context defaults are an object");
  const pagination = expectJsonObject(list, defaults.pagination, "selected path context pagination is an object");
  const limit = expectJsonObject(list, pagination.limit, "selected path context limit is an object");
  expect(list, limit.value === 321, "selected config files participate in document context");
}

function valueFor(record: CommandRecord, configListJson: JsonRecord, key: string): JsonRecord {
  const values = expectObjectArray(record, configListJson.values, "config list values are objects");
  const item = values.find((entry) => entry.key === key);
  return expectJsonObject(record, item, `config list includes ${key}`);
}
