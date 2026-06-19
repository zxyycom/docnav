import { fixture, getNormalRef } from "../fixtures.ts";
import { runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.ts";
import {
  expect,
  expectIncludes,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectReadResultsEquivalent,
  expectStringArray
} from "../assertions.ts";

export function createMachineProtocolTasks() {
  return [
    // @case BB-MD-MACHINE-001
    {
      id: "MD-MACHINE-001",
      label: "MD-MACHINE-001 manifest probe and invoke protocol",
      run: testMachineProtocolChain
    }
  ];
}

async function testMachineProtocolChain() {
  const normal = fixture("normal.md");

  await runSuccessfulJsonCase("MD-MACHINE-001 manifest protocol-json", ["manifest", "--output", "protocol-json"], {
    schema: "manifest",
    check: (record, json) => {
      const adapter = expectJsonObject(record, json.adapter, "manifest adapter is an object");
      const capabilities = expectStringArray(record, json.capabilities, "manifest capabilities are strings");
      const formats = expectObjectArray(record, json.formats, "manifest formats are objects");
      const firstFormat = expectJsonObject(record, formats[0], "manifest first format is an object");
      const extensions = expectStringArray(record, firstFormat.extensions, "manifest format extensions are strings");
      const contentTypes = expectStringArray(record, firstFormat.content_types, "manifest format content types are strings");
      expect(record, adapter.id === "docnav-markdown", "manifest adapter id is docnav-markdown");
      for (const capability of ["outline", "read", "find", "info"]) {
        expectIncludes(record, capabilities, capability, `manifest includes ${capability}`);
      }
      expect(record, firstFormat.id === "markdown", "manifest declares markdown format");
      expectIncludes(record, extensions, ".md", "manifest declares .md extension");
      expectIncludes(record, extensions, ".markdown", "manifest declares .markdown extension");
      expectIncludes(record, contentTypes, "text/markdown", "manifest declares markdown content type");
      expect(record, !Object.hasOwn(json, "result"), "manifest is not a response envelope");
    }
  });

  await runSuccessfulJsonCase("MD-MACHINE-001 probe normal protocol-json", ["probe", normal, "--output", "protocol-json"], {
    schema: "probe",
    check: (record, json) => {
      const reasons = expectObjectArray(record, json.reasons, "probe reasons are objects");
      expect(record, json.supported === true, "normal.md probe is supported");
      expect(record, json.format === "markdown", "normal.md probe format is markdown");
      expect(
        record,
        reasons.some((reason) => reason.code === "EXTENSION_MATCH"),
        "normal.md probe records extension evidence"
      );
      expect(record, !Object.hasOwn(json, "result"), "probe is not a response envelope");
    }
  });

  const ref = await getNormalRef();
  const { json: readableRead } = await runSuccessfulJsonCase(
    "MD-MACHINE-001 read normal readable-json for invoke equivalence",
    ["read", normal, "--ref", ref, "--output", "readable-json"],
    {
      schema: "readableRead",
      check: expectNoProtocolEnvelope
    }
  );
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

  await runProtocolResponseCase("MD-MACHINE-001 invoke valid read request", ["invoke"], {
    operation: "read",
    commandOptions: {
      stdin: JSON.stringify(request),
      stdinSummary: "protocol read request for normal.md"
    },
    check: (record, json) => {
      const result = expectJsonObject(record, json.result, "invoke result is an object");
      expect(record, json.request_id === "smoke-valid-read", "invoke preserves request_id");
      expect(record, result.ref === ref, "invoke read preserves ref");
      expectReadResultsEquivalent(record, result, readableRead, "invoke read result matches readable-json");
    }
  });
}
