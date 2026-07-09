import { readFileSync } from "node:fs";
import path from "node:path";

import {
  configFixturePath,
  createProject,
  writeJson,
} from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectObjectArray,
  expectNoJsonPayloadInStderr,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson,
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";
import { exitCodes } from "../config.ts";
import type { CommandRecord } from "../../../tools/smoke-harness.ts";

export async function testConfigPathFlagsSelectConfigTargets() {
  const project = createProject("config-path-flags", {
    config: {
      options: {
        "docnav-markdown": {
          max_heading_level: 3,
        },
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

  const mutableProjectConfig = path.join(project.root, "selected", "project.json");
  const mutableUserConfig = path.join(project.root, "selected", "user.json");
  writeJson(mutableProjectConfig, {
    defaults: {
      output: "readable-json",
    },
  });
  writeJson(mutableUserConfig, {
    defaults: {
      pagination: {
        limit: 321,
      },
    },
  });

  const inspect = await runCli(
    "CORE-CONFIG-PATH-001 config inspect uses selected config files",
    [
      "config",
      "inspect",
      "--project-config",
      mutableProjectConfig,
      "--user-config",
      mutableUserConfig,
    ],
    { project },
  );
  expectExit(inspect, 0);
  expectStderrEmpty(inspect);
  const inspectJson = parseJson(inspect);
  const inspection = expectJsonObject(inspect, inspectJson.inspection, "config inspect reports inspection");
  const projectSource = sourceFor(inspect, inspection, "project");
  const userSource = sourceFor(inspect, inspection, "user");
  expect(inspect, projectSource.origin === "explicit_cli", "config inspect reports selected project config origin");
  expect(inspect, userSource.origin === "explicit_cli", "config inspect reports selected user config origin");
  expect(inspect, projectSource.path === mutableProjectConfig, "config inspect reports selected project config path");
  expect(inspect, userSource.path === mutableUserConfig, "config inspect reports selected user config path");
  expect(inspect, parameterFact(inspect, inspection, "docnav.defaults.output").value === "readable-json", "config inspect reads selected project config");
  expect(inspect, parameterFact(inspect, inspection, "docnav.defaults.pagination.limit").value === 321, "config inspect reads selected user config");
  expect(
    inspect,
    projectionHasPath(inspect, inspection, "defaults.pagination.limit"),
    "config inspect exposes config-source projection for selected config fields",
  );

  const selectedConfigSnapshot = snapshotSelectedConfigFiles(mutableProjectConfig, mutableUserConfig);
  for (const legacyCommand of legacyConfigCommands()) {
    const legacy = await runCli(
      `CORE-CONFIG-PATH-001 legacy config ${legacyCommand.subcommand} rejects selected config files`,
      [
        "config",
        legacyCommand.subcommand,
        ...legacyCommand.args,
        "--project-config",
        mutableProjectConfig,
        "--user-config",
        mutableUserConfig,
        "--output",
        "protocol-json",
      ],
      { project },
    );
    expectExit(legacy, exitCodes.input);
    expectNoJsonPayloadInStderr(legacy);
    const legacyJson = parseJson(legacy);
    validateSchema(legacy, "protocolResponse", legacyJson);
    const error = expectProtocolFailure(legacy, legacyJson, null, "INVALID_REQUEST");
    expect(legacy, error.owner === "core_cli", "legacy config command is rejected at core CLI boundary");
    const details = expectJsonObject(legacy, error.details, "legacy config command error details are an object");
    expect(legacy, details.field === "config", "legacy config command reports config field");
    expect(
      legacy,
      details.reason === `unknown config subcommand "${legacyCommand.subcommand}"`,
      "legacy config command reports removed subcommand",
    );
    expectSelectedConfigFilesUnchanged(legacy, selectedConfigSnapshot);
  }
}

type SelectedConfigSnapshot = {
  projectPath: string;
  projectBytes: Buffer;
  userPath: string;
  userBytes: Buffer;
};

function snapshotSelectedConfigFiles(projectPath: string, userPath: string): SelectedConfigSnapshot {
  return {
    projectPath,
    projectBytes: readFileSync(projectPath),
    userPath,
    userBytes: readFileSync(userPath),
  };
}

function expectSelectedConfigFilesUnchanged(record: CommandRecord, snapshot: SelectedConfigSnapshot) {
  expect(record, readFileSync(snapshot.projectPath).equals(snapshot.projectBytes), "selected project config is unchanged");
  expect(record, readFileSync(snapshot.userPath).equals(snapshot.userBytes), "selected user config is unchanged");
}

function legacyConfigCommands() {
  return [
    { subcommand: "get", args: ["defaults.output"] },
    { subcommand: "set", args: ["defaults.output", "protocol-json"] },
    { subcommand: "unset", args: ["defaults.output"] },
    { subcommand: "list", args: [] },
  ] as const;
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
