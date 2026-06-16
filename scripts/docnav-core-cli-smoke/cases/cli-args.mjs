import { createProject } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectStderrEmpty,
  expectStdoutIncludes,
  parseJson
} from "../assertions.mjs";
import { exitCodes } from "../config.mjs";

export function createCliArgumentFailureTasks() {
  const cases = [
    {
      name: "missing command",
      args: () => [],
      message: "missing command"
    },
    {
      name: "unknown command",
      args: () => ["unknown"],
      message: "unknown command"
    },
    {
      name: "outline missing path",
      args: () => ["outline"],
      message: "outline requires <path>"
    },
    {
      name: "read missing ref",
      args: (project) => ["read", project.normalRelPath],
      message: "read requires --ref <ref>"
    },
    {
      name: "find missing query",
      args: (project) => ["find", project.normalRelPath],
      message: "find requires --query <text>"
    },
    {
      name: "value flag missing value",
      args: (project) => ["outline", project.normalRelPath, "--limit-chars"],
      message: "flag requires a value"
    },
    {
      name: "page must be positive",
      args: (project) => ["outline", project.normalRelPath, "--page", "0"],
      message: "--page must be a positive integer"
    },
    {
      name: "limit chars must be numeric",
      args: (project) => ["outline", project.normalRelPath, "--limit-chars", "abc"],
      message: "--limit-chars must be a positive integer"
    },
    {
      name: "output mode must be known",
      args: (project) => ["outline", project.normalRelPath, "--output", "xml"],
      message: "invalid --output"
    },
    {
      name: "config missing subcommand",
      args: () => ["config"],
      message: "missing config subcommand"
    },
    {
      name: "config set missing value",
      args: () => ["config", "set", "defaults.output"],
      message: "config set requires <value>"
    },
    {
      name: "config list invalid operation",
      args: (project) => ["config", "list", "--path", project.normalRelPath, "--operation", "bad"],
      message: "expected outline, read, find, or info"
    }
  ];

  return [
    ...cases.map((testCase) => ({
      id: `core-cli-arg-failure-${slug(testCase.name)}`,
      run: async () => {
        const project = createProject(`cli-argument-failure-${testCase.name}`);
        const record = await runCli(testCase.name, testCase.args(project), { project });
        expectExit(record, exitCodes.input);
        expectStderrEmpty(record);
        expectStdoutIncludes(record, "\"$block\": \"/error\"");
        expectStdoutIncludes(record, "\"code\": \"INVALID_REQUEST\"");
        expectStdoutIncludes(record, testCase.message);
      }
    })),
    {
      id: "core-cli-arg-parse-failure-missing-ref-protocol",
      run: async () => {
        const project = createProject("cli-argument-parse-failure-missing-ref");
        const missingRef = await runCli("read missing ref protocol-json parse failure", [
          "read",
          project.normalRelPath,
          "--output",
          "protocol-json"
        ], { project });
        expectExit(missingRef, exitCodes.input);
        expectNoJsonPayloadInStderr(missingRef);
        const missingRefJson = parseJson(missingRef);
        validateSchema(missingRef, "protocolResponse", missingRefJson);
        expectProtocolFailure(missingRef, missingRefJson, "read", "INVALID_REQUEST");
      }
    },
    {
      id: "core-cli-arg-parse-failure-unknown-command-protocol",
      run: async () => {
        const project = createProject("cli-argument-parse-failure-unknown-command");
        const unknownCommand = await runCli("unknown command protocol-json parse failure", [
          "unknown",
          "--output",
          "protocol-json"
        ], { project });
        expectExit(unknownCommand, exitCodes.input);
        expectNoJsonPayloadInStderr(unknownCommand);
        const unknownCommandJson = parseJson(unknownCommand);
        validateSchema(unknownCommand, "protocolResponse", unknownCommandJson);
        expectProtocolFailure(unknownCommand, unknownCommandJson, null, "INVALID_REQUEST");
      }
    },
    {
      id: "core-cli-arg-parse-failure-missing-path-readable",
      run: async () => {
        const project = createProject("cli-argument-parse-failure-missing-path");
        const missingPath = await runCli("outline missing path readable-json parse failure", [
          "outline",
          "--output",
          "readable-json"
        ], { project });
        expectExit(missingPath, exitCodes.input);
        expectStderrEmpty(missingPath);
        const missingPathJson = parseJson(missingPath);
        validateSchema(missingPath, "readableError", missingPathJson);
        expectNoProtocolEnvelope(missingPath, missingPathJson);
      }
    }
  ];
}

function slug(value) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}
