import path from "node:path";

import { exitCodes, fixturesDir } from "../config.mjs";
import { fixture } from "../fixtures.mjs";
import { runCli, validateSchema } from "../harness.mjs";
import {
  expect,
  expectExit,
  expectNoJsonPayloadInStderr,
  expectNoProtocolEnvelope,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";

export function testReadableOperationErrors() {
  const normal = fixture("normal.md");
  const missingPath = path.join(fixturesDir, "missing.md");
  const invalidRef = "L99:Missing";
  const legacyOrdinalRef = legacyBracketedOrdinalRef();
  const canonicalNotFoundRef = "H:L99:H1:I1";
  const cases = [
    {
      name: "missing file readable-json",
      args: ["outline", missingPath, "--output", "readable-json"],
      code: "DOCUMENT_NOT_FOUND",
      detailKey: "path",
      detailValue: missingPath
    },
    {
      name: "invalid ref readable-json (old format)",
      args: ["read", normal, "--ref", invalidRef, "--output", "readable-json"],
      code: "REF_INVALID",
      detailKey: "ref",
      detailValue: invalidRef
    },
    {
      name: "legacy ordinal ref readable-json (REF_INVALID)",
      args: ["read", normal, "--ref", legacyOrdinalRef, "--output", "readable-json"],
      code: "REF_INVALID",
      detailKey: "ref",
      detailValue: legacyOrdinalRef
    },
    {
      name: "canonical ref not found readable-json",
      args: ["read", normal, "--ref", canonicalNotFoundRef, "--output", "readable-json"],
      code: "REF_NOT_FOUND",
      detailKey: "ref",
      detailValue: canonicalNotFoundRef
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
    if (Object.hasOwn(item, "detailValue")) {
      expect(record, json.details[item.detailKey] === item.detailValue, `${item.name} preserves details.${item.detailKey}`);
    }
    expect(record, Array.isArray(json.guidance), `${item.name} includes guidance array`);
  }
}

export function testProtocolOperationErrors() {
  const normal = fixture("normal.md");
  const missingPath = path.join(fixturesDir, "missing.md");
  const invalidRef = "L99:Missing";
  const legacyOrdinalRef = legacyBracketedOrdinalRef();
  const canonicalNotFoundRef = "H:L99:H1:I1";
  const cases = [
    {
      name: "missing file protocol-json",
      args: ["outline", missingPath, "--output", "protocol-json"],
      operation: "outline",
      code: "DOCUMENT_NOT_FOUND",
      detailKey: "path",
      detailValue: missingPath
    },
    {
      name: "invalid ref protocol-json (old format)",
      args: ["read", normal, "--ref", invalidRef, "--output", "protocol-json"],
      operation: "read",
      code: "REF_INVALID",
      detailKey: "ref",
      detailValue: invalidRef
    },
    {
      name: "legacy ordinal ref protocol-json (REF_INVALID)",
      args: ["read", normal, "--ref", legacyOrdinalRef, "--output", "protocol-json"],
      operation: "read",
      code: "REF_INVALID",
      detailKey: "ref",
      detailValue: legacyOrdinalRef
    },
    {
      name: "canonical ref not found protocol-json",
      args: ["read", normal, "--ref", canonicalNotFoundRef, "--output", "protocol-json"],
      operation: "read",
      code: "REF_NOT_FOUND",
      detailKey: "ref",
      detailValue: canonicalNotFoundRef
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
    if (Object.hasOwn(item, "detailValue")) {
      expect(
        record,
        json.error.details[item.detailKey] === item.detailValue,
        `${item.name} preserves error.details.${item.detailKey}`
      );
    }
  }
}

function legacyBracketedOrdinalRef() {
  return ["L1:Guide ", "[", "doc", "nav:", "1", "]"].join("");
}
