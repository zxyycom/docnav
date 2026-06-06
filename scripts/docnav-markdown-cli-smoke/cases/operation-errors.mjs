import path from "node:path";

import { exitCodes, fixturesDir } from "../config.mjs";
import { fixture } from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../schemas.mjs";

export function testReadableOperationErrors() {
  const normal = fixture("normal.md");
  const cases = [
    {
      name: "missing file readable-json",
      args: ["outline", path.join(fixturesDir, "missing.md"), "--output", "readable-json"],
      code: "DOCUMENT_NOT_FOUND",
      detailKey: "path"
    },
    {
      name: "invalid ref readable-json",
      args: ["read", normal, "--ref", "L99:Missing [docnav:1]", "--output", "readable-json"],
      code: "REF_NOT_FOUND",
      detailKey: "ref"
    },
    {
      name: "non UTF-8 readable-json",
      args: ["outline", fixture("non-utf8.md"), "--output", "readable-json"],
      code: "DOCUMENT_ENCODING_UNSUPPORTED",
      detailKey: "encoding"
    }
  ];

  for (const item of cases) {
    const record = runCli(item.name, item.args);
    expectExit(record, exitCodes.documentRefFormat);
    expectStderrEmpty(record);
    const json = parseJson(record);
    validateSchema(record, "readableError", json);
    expectNoProtocolEnvelope(record, json);
    expect(record, json.code === item.code, `${item.name} returns ${item.code}`);
    expect(record, Object.hasOwn(json.details, item.detailKey), `${item.name} includes details.${item.detailKey}`);
    expect(record, Array.isArray(json.guidance), `${item.name} includes guidance array`);
  }
}

export function testProtocolOperationErrors() {
  const normal = fixture("normal.md");
  const cases = [
    {
      name: "invalid ref protocol-json",
      args: ["read", normal, "--ref", "L99:Missing [docnav:1]", "--output", "protocol-json"],
      operation: "read",
      code: "REF_NOT_FOUND",
      detailKey: "ref"
    },
    {
      name: "non UTF-8 protocol-json",
      args: ["outline", fixture("non-utf8.md"), "--output", "protocol-json"],
      operation: "outline",
      code: "DOCUMENT_ENCODING_UNSUPPORTED",
      detailKey: "encoding"
    }
  ];

  for (const item of cases) {
    const record = runCli(item.name, item.args);
    expectExit(record, exitCodes.documentRefFormat);
    expectNoJsonPayloadInStderr(record);
    const json = parseJson(record);
    validateSchema(record, "protocolResponse", json);
    expectProtocolFailure(record, json, item.operation, item.code);
    expect(record, Object.hasOwn(json.error.details, item.detailKey), `${item.name} includes error.details.${item.detailKey}`);
  }
}

