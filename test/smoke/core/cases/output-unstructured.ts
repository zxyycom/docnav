import fs from "node:fs";
import path from "node:path";

import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import { createProject } from "../fixtures.ts";
import { runCli, validateSchema } from "../harness.ts";

export async function testUnstructuredOutlineOutputModes() {
  await assertPathRuleUnstructuredOutline();
  await assertCostThresholdUnstructuredOutline();
}

async function assertPathRuleUnstructuredOutline() {
  const content = "raw note\nsecond line";
  const project = createProject("output-boundary-unstructured-outline", {
    config: {
      outline: {
        mode_rules: [
          { path: "docs/raw\\.md", mode: "unstructured_full" }
        ]
      }
    }
  });
  const rawRelPath = "docs/raw.md";
  fs.writeFileSync(path.join(project.root, rawRelPath), content, "utf8");

  const protocol = await runCli("CORE-OUTPUT-004 outline unstructured protocol-json", [
    "outline",
    rawRelPath,
    "--output",
    "protocol-json"
  ], { project });
  expectExit(protocol, 0);
  expectStderrEmpty(protocol);
  const protocolJson = parseJson(protocol);
  validateSchema(protocol, "protocolResponse", protocolJson);
  expectProtocolSuccess(protocol, protocolJson, "outline");
  const protocolResult = expectJsonObject(
    protocol,
    protocolJson.result,
    "protocol unstructured result is an object"
  );
  expect(protocol, protocolResult.kind === "unstructured", "protocol-json outline uses unstructured kind");
  expect(protocol, protocolResult.reason === "path_rule", "protocol-json outline preserves path_rule reason");
  expect(protocol, protocolResult.content === content, "protocol-json outline contains full content");
  expect(protocol, protocolResult.content_type === "text/markdown", "protocol-json outline preserves Markdown content_type");
  const protocolCost = expectJsonObject(protocol, protocolResult.cost, "protocol unstructured cost is an object");
  const protocolMeasurements = expectObjectArray(
    protocol,
    protocolCost.measurements,
    "protocol unstructured cost measurements are objects"
  );
  expect(protocol, protocolMeasurements.length > 0, "protocol unstructured cost facts are non-empty for Markdown hook");
  expect(protocol, !Object.hasOwn(protocolResult, "display"), "protocol-json unstructured outline omits readable display");
  expect(protocol, !Object.hasOwn(protocolResult, "entries"), "protocol-json unstructured outline omits entries");
  expect(protocol, !Object.hasOwn(protocolResult, "ref"), "protocol-json unstructured outline omits ref");
  expect(protocol, !Object.hasOwn(protocolResult, "page"), "protocol-json unstructured outline omits page");
  expect(protocol, !Object.hasOwn(protocolResult, "continuation"), "protocol-json unstructured outline omits continuation");

  const readableView = await runCli("CORE-OUTPUT-004 outline unstructured readable-view", [
    "outline",
    rawRelPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readableView, 0);
  expectStderrEmpty(readableView);
  const header = parseReadableViewHeader(readableView);
  expectNoProtocolEnvelope(readableView, header);
  expect(readableView, header.kind === "unstructured", "readable-view outline uses unstructured kind");
  expect(readableView, header.reason === "path_rule", "readable-view outline preserves path_rule reason");
  expect(readableView, header.content_type === protocolResult.content_type, "readable-view preserves raw content_type");
  const readableCost = expectJsonObject(readableView, header.cost, "readable-view unstructured cost is an object");
  const readableMeasurements = expectObjectArray(
    readableView,
    readableCost.measurements,
    "readable-view unstructured cost measurements are objects"
  );
  expect(
    readableView,
    readableMeasurements.length === protocolMeasurements.length,
    "readable-view derives cost facts from the protocol result"
  );
  expect(readableView, !Object.hasOwn(header, "entries"), "readable-view unstructured outline omits entries");
  expect(readableView, !Object.hasOwn(header, "ref"), "readable-view unstructured outline omits ref");
  expect(readableView, !Object.hasOwn(header, "page"), "readable-view unstructured outline omits page");
  expect(readableView, !Object.hasOwn(header, "continuation"), "readable-view unstructured outline omits continuation");
  expectReadableViewBlockRestoresField(readableView, readableView.stdout, "/content", content);
}

async function assertCostThresholdUnstructuredOutline() {
  const content = "small operational note";
  const project = createProject("output-boundary-unstructured-outline-cost", {
    config: {
      outline: {
        auto_full_read: {
          thresholds: [
            { adapter: "docnav-markdown", unit: "tokens", value: 1000 }
          ]
        }
      }
    }
  });
  const rawRelPath = "docs/small.md";
  fs.writeFileSync(path.join(project.root, rawRelPath), content, "utf8");

  const readable = await runCli("CORE-OUTPUT-004 outline unstructured cost-threshold readable-view", [
    "outline",
    rawRelPath,
    "--output",
    "readable-view"
  ], { project });
  expectExit(readable, 0);
  expectStderrEmpty(readable);
  const header = parseReadableViewHeader(readable);
  expectNoProtocolEnvelope(readable, header);
  expect(readable, header.kind === "unstructured", "cost-threshold readable-view outline uses unstructured kind");
  expect(readable, header.reason === "cost_threshold", "cost-threshold readable-view preserves cost_threshold reason");
  const readableCost = expectJsonObject(readable, header.cost, "cost-threshold readable-view cost is an object");
  const readableMeasurements = expectObjectArray(
    readable,
    readableCost.measurements,
    "cost-threshold readable-view cost measurements are objects"
  );
  expect(readable, readableMeasurements.length > 0, "cost-threshold readable-view cost facts are non-empty");
  expect(readable, !Object.hasOwn(header, "entries"), "cost-threshold readable-view unstructured outline omits entries");
  expect(readable, !Object.hasOwn(header, "page"), "cost-threshold readable-view unstructured outline omits page");
  expectReadableViewBlockRestoresField(readable, readable.stdout, "/content", content);
}
