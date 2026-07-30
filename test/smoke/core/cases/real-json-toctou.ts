import fs from "node:fs";
import path from "node:path";

import { createProject } from "../fixtures.ts";
import {
  runTestHelper,
  smokeState,
  validateSchema
} from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectProtocolFailure,
  expectStderrEmpty,
  parseJson
} from "../assertions.ts";
import { root } from "../config.ts";

const documentPath = "docs/toctou.json";
const invalidDocument = "{\n";
const supervisorPath = path.join(
  root,
  "test",
  "tools",
  "json-toctou-supervisor.py"
);

export function createRealJsonToctouTasks() {
  return [
    {
      id: "CORE-JSON-TOCTOU-001",
      label: "CORE-JSON-TOCTOU-001 JSON operation reload detects a changed document",
      run: testJsonOperationReloadDetectsChangedDocument
    }
  ];
}

async function testJsonOperationReloadDetectsChangedDocument() {
  const project = createProject("real-json-toctou", { normalDocument: false });
  const targetPath = path.join(project.root, documentPath);
  const replacementPath = `${targetPath}.invalid.next`;
  fs.writeFileSync(targetPath, "{\"stable\":{\"value\":1}}\n", "utf8");
  fs.writeFileSync(replacementPath, invalidDocument, "utf8");

  const docnavBinaryPath = smokeState.docnavBinaryPath;
  if (!docnavBinaryPath) {
    throw new Error("docnav binary path is required for JSON TOCTOU smoke");
  }

  const record = await runTestHelper(
    "CORE-JSON-TOCTOU-001 explicit JSON outline after atomic replacement",
    "uv",
    [
      "run",
      "--no-project",
      "python",
      supervisorPath,
      "--docnav-bin",
      docnavBinaryPath,
      "--target",
      targetPath,
      "--replacement",
      replacementPath,
      "--",
      "outline",
      documentPath,
      "--adapter",
      "docnav-json",
      "--output",
      "protocol-json"
    ],
    { project }
  );

  expectExit(record, 1);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(
    record,
    json,
    "outline",
    "INTERNAL_ERROR"
  );
  const details = expectJsonObject(
    record,
    error.details,
    "INTERNAL_ERROR details are an object"
  );
  expect(
    record,
    details.error_id === "json-document-changed-after-probe",
    "operation reload reports the JSON document-changed error id"
  );
  expect(
    record,
    fs.readFileSync(targetPath, "utf8") === invalidDocument,
    "atomic replacement leaves the target path invalid"
  );
}
