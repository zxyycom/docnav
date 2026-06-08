import { createProject } from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expectExit,
  expectStderrEmpty,
  expectStdoutIncludes
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

export function testCliArgumentFailures() {
  const project = createProject("cli-argument-failures");
  const cases = [
    {
      name: "missing command",
      args: [],
      message: "missing command"
    },
    {
      name: "unknown command",
      args: ["unknown"],
      message: "unknown command"
    },
    {
      name: "outline missing path",
      args: ["outline"],
      message: "outline requires <path>"
    },
    {
      name: "read missing ref",
      args: ["read", project.normalRelPath],
      message: "read requires --ref <ref>"
    },
    {
      name: "find missing query",
      args: ["find", project.normalRelPath],
      message: "find requires --query <text>"
    },
    {
      name: "value flag missing value",
      args: ["outline", project.normalRelPath, "--limit-chars"],
      message: "flag requires a value"
    },
    {
      name: "page must be positive",
      args: ["outline", project.normalRelPath, "--page", "0"],
      message: "--page must be a positive integer"
    },
    {
      name: "limit chars must be numeric",
      args: ["outline", project.normalRelPath, "--limit-chars", "abc"],
      message: "--limit-chars must be a positive integer"
    },
    {
      name: "output mode must be known",
      args: ["outline", project.normalRelPath, "--output", "xml"],
      message: "invalid --output"
    },
    {
      name: "config missing subcommand",
      args: ["config"],
      message: "missing config subcommand"
    },
    {
      name: "config set missing value",
      args: ["config", "set", "defaults.output"],
      message: "config set requires <value>"
    },
    {
      name: "config list invalid operation",
      args: ["config", "list", "--path", project.normalRelPath, "--operation", "bad"],
      message: "expected outline, read, find, or info"
    }
  ];

  for (const testCase of cases) {
    const record = runCli(testCase.name, testCase.args, { project });
    expectExit(record, exitCodes.input);
    expectStderrEmpty(record);
    expectStdoutIncludes(record, "error: INVALID_REQUEST");
    expectStdoutIncludes(record, testCase.message);
  }
}
