import { createProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";
import {
  expect,
  expectExit,
  expectProtocolFailure,
  expectReadableFailure,
  expectStderrEmpty,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import { exitCodes } from "../config.ts";

export async function assertConfiguredProtocolEarlyFailure(missingPath: string) {
  const project = createProject("output-boundary-configured-protocol", {
    config: {
      defaults: {
        output: "protocol-json"
      }
    }
  });
  const record = await runCli("CORE-OUTPUT-001 configured early failure protocol-json", [
    "read",
    missingPath,
    "--ref",
    "H:L1:H1"
  ], { project });

  expectExit(record, exitCodes.documentRefFormat);
  expectStderrEmpty(record);
  const response = parseJson(record);
  validateSchema(record, "protocolResponse", response);
  expectProtocolFailure(record, response, "read", "DOCUMENT_NOT_FOUND");
  expect(
    record,
    !record.stdout.includes("[block "),
    "config-selected protocol early failure has no readable block framing"
  );

  const invalidCliOutput = await runCli(
    "CORE-OUTPUT-001 invalid CLI output does not fall through to config",
    ["outline", project.normalRelPath, "--output", "readable-json"],
    { project }
  );
  expectExit(invalidCliOutput, exitCodes.input);
  expectStderrEmpty(invalidCliOutput);
  expectReadableFailure(
    invalidCliOutput,
    parseReadableViewHeader(invalidCliOutput),
    "INVALID_REQUEST"
  );
}
