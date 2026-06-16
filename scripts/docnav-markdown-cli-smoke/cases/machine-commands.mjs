import { fixture, getNormalRef } from "../fixtures.mjs";
import { runProtocolResponseCase, runSuccessfulJsonCase } from "../harness.mjs";
import {
  expect,
  expectIncludes,
  expectNoProtocolEnvelope,
  expectReadResultsEquivalent
} from "../assertions.mjs";

export function createManifestProbeTasks() {
  return [
    {
      id: "markdown-machine-manifest",
      run: async () => {
        await runSuccessfulJsonCase(
          "manifest protocol-json",
          ["manifest", "--output", "protocol-json"],
          {
            schema: "manifest",
            check: (record, json) => {
              expect(record, json.adapter.id === "docnav-markdown", "manifest adapter id is docnav-markdown");
              for (const capability of ["outline", "read", "find", "info"]) {
                expectIncludes(record, json.capabilities, capability, `manifest includes ${capability}`);
              }
              expect(record, json.formats[0].id === "markdown", "manifest declares markdown format");
              expectIncludes(record, json.formats[0].extensions, ".md", "manifest declares .md extension");
              expectIncludes(record, json.formats[0].extensions, ".markdown", "manifest declares .markdown extension");
              expectIncludes(
                record,
                json.formats[0].content_types,
                "text/markdown",
                "manifest declares markdown content type"
              );
              expect(record, !Object.hasOwn(json, "protocol"), "manifest omits protocol range");
              expect(record, !Object.hasOwn(json, "recommended_parameters"), "manifest omits recommended parameters");
              expect(record, !Object.hasOwn(json, "entries"), "manifest has no navigation payload");
              expect(record, !Object.hasOwn(json, "result"), "manifest is not a response envelope");
            }
          }
        );
      }
    },
    ...["uppercase.MD", "longform.markdown"].map((name) => ({
      id: `markdown-machine-probe-${slug(name)}`,
      run: async () => {
        await runSuccessfulJsonCase(
          `probe ${name} protocol-json`,
          ["probe", fixture(name), "--output", "protocol-json"],
          {
            schema: "probe",
            check: (record, json) => {
              expect(record, json.supported === true, `${name} probe is supported`);
              expect(record, json.format === "markdown", `${name} probe format is markdown`);
              expect(
                record,
                json.reasons.some((reason) => reason.code === "EXTENSION_MATCH"),
                `${name} probe records extension evidence`
              );
              expect(record, !Object.hasOwn(json, "entries"), `${name} probe has no outline payload`);
              expect(record, !Object.hasOwn(json, "result"), `${name} probe is not a response envelope`);
            }
          }
        );
      }
    }))
  ];
}

export function createValidInvokeTasks() {
  return [
    {
      id: "markdown-machine-valid-invoke",
      run: async () => {
        const normal = fixture("normal.md");
        const ref = await getNormalRef();
        const { json: readableRead } = await runSuccessfulJsonCase(
          "read normal readable-json for invoke equivalence",
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

        await runProtocolResponseCase("invoke valid read request", ["invoke"], {
          operation: "read",
          commandOptions: {
            stdin: JSON.stringify(request),
            stdinSummary: "protocol read request for normal.md"
          },
          check: (record, json) => {
            expect(record, json.request_id === "smoke-valid-read", "invoke preserves request_id");
            expect(record, json.result.ref === ref, "invoke read preserves ref");
            expectReadResultsEquivalent(
              record,
              json.result,
              readableRead,
              "invoke read result matches readable-json"
            );
          }
        });
      }
    }
  ];
}

function slug(value) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}
