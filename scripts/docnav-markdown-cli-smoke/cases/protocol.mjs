import {
  fixture,
  getNormalProtocolReadResult,
  getNormalReadableFindResult,
  getNormalReadableReadResult,
  getNormalRef,
  setNormalProtocolReadResult
} from "../fixtures.mjs";
import { runCli } from "../runner.mjs";
import {
  expect,
  expectExit,
  expectFindResultsEquivalent,
  expectIncludes,
  expectNormalFindResult,
  expectProtocolSuccess,
  expectReadResultsEquivalent,
  expectStderrEmpty,
  parseJson
} from "../assertions.mjs";
import { validateSchema } from "../schemas.mjs";

export function testProtocolOutputs() {
  const normal = fixture("normal.md");
  const ref = getNormalRef();
  const cases = [
    {
      name: "outline normal protocol-json",
      args: ["outline", normal, "--output", "protocol-json"],
      operation: "outline",
      check: (record, json) => {
        expect(record, Array.isArray(json.result.entries), "outline protocol result has entries");
        expect(record, json.result.entries.length > 0, "outline protocol result has at least one entry");
      }
    },
    {
      name: "read normal protocol-json",
      args: ["read", normal, "--ref", ref, "--output", "protocol-json"],
      operation: "read",
      check: (record, json) => {
        expect(record, json.result.ref === ref, "read protocol result preserves ref");
        expect(record, json.result.content_type === "text/markdown", "read protocol result has content_type");
        expectReadResultsEquivalent(
          record,
          json.result,
          getNormalReadableReadResult(),
          "read protocol-json result matches readable-json"
        );
        setNormalProtocolReadResult(json.result);
      }
    },
    {
      name: "find normal protocol-json",
      args: ["find", normal, "--query", "target", "--output", "protocol-json"],
      operation: "find",
      check: (record, json) => {
        expectNormalFindResult(record, json.result, "protocol find");
        expectFindResultsEquivalent(
          record,
          json.result,
          getNormalReadableFindResult(),
          "find protocol-json result matches readable-json"
        );
      }
    },
    {
      name: "info normal protocol-json",
      args: ["info", normal, "--output", "protocol-json"],
      operation: "info",
      check: (record, json) => {
        expect(record, json.result.display.includes("Markdown | text/markdown"), "info protocol result has display");
        expectIncludes(record, json.result.capabilities, "read", "info protocol result includes read capability");
      }
    }
  ];

  for (const item of cases) {
    const record = runCli(item.name, item.args);
    expectExit(record, 0);
    expectStderrEmpty(record);
    const json = parseJson(record);
    validateSchema(record, "protocolResponse", json);
    expectProtocolSuccess(record, json, item.operation);
    item.check(record, json);
  }
}

export function testManifestProbe() {
  const manifest = runCli("manifest protocol-json", ["manifest", "--output", "protocol-json"]);
  expectExit(manifest, 0);
  expectStderrEmpty(manifest);
  const manifestJson = parseJson(manifest);
  validateSchema(manifest, "manifest", manifestJson);
  expect(manifest, manifestJson.adapter.id === "docnav-markdown", "manifest adapter id is docnav-markdown");
  for (const capability of ["outline", "read", "find", "info"]) {
    expectIncludes(manifest, manifestJson.capabilities, capability, `manifest includes ${capability}`);
  }
  expect(
    manifest,
    manifestJson.recommended_parameters.outline.limit_chars === 6000,
    "manifest recommends outline limit_chars"
  );
  expect(
    manifest,
    manifestJson.recommended_parameters.outline.options.max_heading_level === 3,
    "manifest recommends max_heading_level"
  );
  expect(manifest, !Object.hasOwn(manifestJson, "entries"), "manifest has no navigation payload");
  expect(manifest, !Object.hasOwn(manifestJson, "result"), "manifest is not a response envelope");

  for (const name of ["uppercase.MD", "longform.markdown"]) {
    const probe = runCli(`probe ${name} protocol-json`, [
      "probe",
      fixture(name),
      "--output",
      "protocol-json"
    ]);
    expectExit(probe, 0);
    expectStderrEmpty(probe);
    const probeJson = parseJson(probe);
    validateSchema(probe, "probe", probeJson);
    expect(probe, probeJson.supported === true, `${name} probe is supported`);
    expect(probe, probeJson.format === "markdown", `${name} probe format is markdown`);
    expect(probe, probeJson.reasons.some((reason) => reason.code === "EXTENSION_MATCH"), `${name} probe records extension evidence`);
    expect(probe, !Object.hasOwn(probeJson, "entries"), `${name} probe has no outline payload`);
    expect(probe, !Object.hasOwn(probeJson, "result"), `${name} probe is not a response envelope`);
  }
}

export function testValidInvoke() {
  const normal = fixture("normal.md");
  const ref = getNormalRef();
  const request = {
    protocol_version: "0.1",
    request_id: "smoke-valid-read",
    operation: "read",
    document: { path: normal },
    arguments: {
      ref,
      limit_chars: 6000,
      page: 1
    }
  };

  const record = runCli("invoke valid read request", ["invoke"], {
    stdin: JSON.stringify(request),
    stdinSummary: "protocol read request for normal.md"
  });
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  expectProtocolSuccess(record, json, "read");
  expect(record, json.request_id === "smoke-valid-read", "invoke preserves request_id");
  expect(record, json.result.ref === ref, "invoke read preserves ref");
  expectReadResultsEquivalent(
    record,
    json.result,
    getNormalReadableReadResult(),
    "invoke read result matches readable-json"
  );
  expectReadResultsEquivalent(
    record,
    json.result,
    getNormalProtocolReadResult(),
    "invoke read result matches protocol-json"
  );
}
