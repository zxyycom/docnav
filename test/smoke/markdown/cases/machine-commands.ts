import { fixture, getNormalRef } from "../fixtures.ts";
import { runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.ts";
import {
  expect,
  expectIncludes,
  expectNoProtocolEnvelope,
  expectReadResultsEquivalent
} from "../assertions.ts";

export function createMachineProtocolTasks() {
  return [
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
    check: (record: any, json: any) => {
      expect(record, json.adapter.id === "docnav-markdown", "manifest adapter id is docnav-markdown");
      for (const capability of ["outline", "read", "find", "info"]) {
        expectIncludes(record, json.capabilities, capability, `manifest includes ${capability}`);
      }
      expect(record, json.formats[0].id === "markdown", "manifest declares markdown format");
      expectIncludes(record, json.formats[0].extensions, ".md", "manifest declares .md extension");
      expectIncludes(record, json.formats[0].extensions, ".markdown", "manifest declares .markdown extension");
      expectIncludes(record, json.formats[0].content_types, "text/markdown", "manifest declares markdown content type");
      expect(record, !Object.hasOwn(json, "result"), "manifest is not a response envelope");
    }
  });

  await runSuccessfulJsonCase("MD-MACHINE-001 probe normal protocol-json", ["probe", normal, "--output", "protocol-json"], {
    schema: "probe",
    check: (record: any, json: any) => {
      expect(record, json.supported === true, "normal.md probe is supported");
      expect(record, json.format === "markdown", "normal.md probe format is markdown");
      expect(
        record,
        json.reasons.some((reason: any) => reason.code === "EXTENSION_MATCH"),
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
    check: (record: any, json: any) => {
      expect(record, json.request_id === "smoke-valid-read", "invoke preserves request_id");
      expect(record, json.result.ref === ref, "invoke read preserves ref");
      expectReadResultsEquivalent(record, json.result, readableRead, "invoke read result matches readable-json");
    }
  });
}
