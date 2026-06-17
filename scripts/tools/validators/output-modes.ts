import fs from "node:fs";
import path from "node:path";

import { assert, assertDeepEqual, readJson, toAbs } from "./fs-utils.ts";
import { DOCUMENT_OUTPUT_MODES, OUTPUT_MODE_CONSISTENCY } from "./config.ts";
import { readText, sortedUnique } from "./document-files.ts";

function assertIncludesDocumentOutputModes(relPath: any) {
  const text = readText(relPath);
  for (const mode of DOCUMENT_OUTPUT_MODES) {
    assert(
      text.includes(mode),
      `${relPath} must mention document output mode ${mode}`,
    );
  }
}

function assertDocumentOutputModeSet(values: any, label: any) {
  assertDeepEqual(
    values,
    DOCUMENT_OUTPUT_MODES,
    `${label} must be exactly ${DOCUMENT_OUTPUT_MODES.join(", ")}`,
  );
}

function extractRustStringArray(source: any, constName: any, label: any) {
  const pattern = new RegExp(
    `const\\s+${constName}\\s*:[^=]+?=\\s*&\\s*\\[([^\\]]*)\\]`,
    "su",
  );
  const match = source.match(pattern);
  assert(match, `${label} must declare ${constName}`);
  return [...match[1].matchAll(/"([^"]+)"/gu)].map((item) => item[1]);
}

function extractOutputValueConstants(source: any, label: any) {
  const constants: Record<string, any> = {};
  for (const match of source.matchAll(
    /const\s+(PROTOCOL_JSON|READABLE_JSON|READABLE_VIEW)\s*:\s*&str\s*=\s*"([^"]+)"/gu,
  )) {
    constants[match[1]] = match[2];
  }
  const values = [
    constants.READABLE_VIEW,
    constants.READABLE_JSON,
    constants.PROTOCOL_JSON,
  ];
  assert(
    values.every((value) => typeof value === "string"),
    `${label} must declare readable-view/readable-json/protocol-json constants`,
  );
  return values;
}

function assertOutputHelpShape(source: any, label: any) {
  const valueName = "readable-view|readable-json|protocol-json";
  assert(
    source.includes(`"${valueName}"`),
    `${label} must advertise ${valueName}`,
  );
  assert(
    !source.includes('.value_name("text|readable-json|protocol-json")') &&
      !source.includes(
        '.value_name("readable-view|readable-json|protocol-json|text")',
      ),
    `${label} must not advertise legacy text as a document output mode`,
  );
}

function validateRustOutputModeEnums() {
  const core = readText(OUTPUT_MODE_CONSISTENCY.coreOutputModeRust);
  assert(
    core.includes("pub enum OutputMode"),
    "core OutputMode enum must remain declared in cli/types.rs",
  );
  assertDocumentOutputModeSet(
    extractRustStringArray(core, "ACCEPTED_VALUES", "core OutputMode"),
    "core OutputMode::ACCEPTED_VALUES",
  );
  assert(
    core.includes("ACCEPTED_VALUES") &&
      DOCUMENT_OUTPUT_MODES.every((mode) => core.includes(`"${mode}"`)),
    "core OutputMode::ACCEPTED_VALUES must list the three document output modes",
  );
  assert(
    !core.includes("OutputMode::Text"),
    "core OutputMode must not retain Text variant use",
  );
  assert(
    !/^\s*Text\s*,/mu.test(core),
    "core OutputMode enum must not declare Text",
  );

  const adapter = readText(OUTPUT_MODE_CONSISTENCY.adapterOutputModeRust);
  assert(
    adapter.includes("pub enum DirectOutputMode"),
    "adapter DirectOutputMode enum must remain declared",
  );
  for (const variant of ["ReadableView", "ReadableJson", "ProtocolJson"]) {
    assert(
      adapter.includes(variant),
      `adapter DirectOutputMode missing ${variant}`,
    );
  }
  assert(
    !adapter.includes("DirectOutputMode::Text") &&
      !/^\s*Text\s*,/mu.test(adapter),
    "adapter DirectOutputMode must not retain Text",
  );

  const coreHelp = readText(OUTPUT_MODE_CONSISTENCY.coreOutputHelpRust);
  assertDocumentOutputModeSet(
    extractOutputValueConstants(coreHelp, "core parser output_values"),
    "core parser output_values",
  );
  assertOutputHelpShape(coreHelp, "core document output help");

  const adapterHelp = readText(OUTPUT_MODE_CONSISTENCY.adapterOutputHelpRust);
  assertDocumentOutputModeSet(
    extractOutputValueConstants(adapterHelp, "adapter direct output_values"),
    "adapter direct output_values",
  );
  assertOutputHelpShape(adapterHelp, "adapter direct document output help");
  assert(
    adapterHelp.includes('.value_name("protocol-json")'),
    "adapter protocol-only help must stay protocol-json only",
  );
}

function validateOutputModeSmokeMatrices() {
  for (const relPath of OUTPUT_MODE_CONSISTENCY.smokeMatrices) {
    assertIncludesDocumentOutputModes(relPath);
  }

  const coreHelp = readText(
    "test/smoke/core/cases/config-management.ts",
  );
  assert(
    coreHelp.includes("outline help does not mention text output mode"),
    "core smoke must assert document help omits text output mode",
  );
  const adapterHelp = readText(
    "test/smoke/markdown/cases/cli-args.ts",
  );
  assert(
    adapterHelp.includes("outline help does not mention text output mode"),
    "adapter smoke must assert document help omits text output mode",
  );

  const configSmoke = readText(
    "test/smoke/core/cases/config-management.ts",
  );
  assert(
    configSmoke.includes(
      "accepted values: readable-view, readable-json, protocol-json",
    ),
    "core config smoke must assert exact defaults.output accepted values",
  );
}

function validateConformanceFixtures() {
  const conformanceDir = OUTPUT_MODE_CONSISTENCY.conformanceDir;
  for (const fixture of OUTPUT_MODE_CONSISTENCY.conformanceFixtures) {
    const relPath = path.posix.join(conformanceDir, fixture);
    assert(
      fs.existsSync(toAbs(relPath)),
      `missing readable-view conformance fixture ${relPath}`,
    );
  }

  const readme = readText(OUTPUT_MODE_CONSISTENCY.conformanceReadme);
  for (const fixture of OUTPUT_MODE_CONSISTENCY.conformanceFixtures) {
    assert(
      readme.includes(fixture),
      `conformance README must index ${fixture}`,
    );
  }

  const warningFixture = readText(
    path.posix.join(conformanceDir, "13_warning.json"),
  );
  assert(
    !warningFixture.includes("--text"),
    "warning conformance fixture must not describe removed text output as a current mode",
  );

  const fixtureDirAbs = toAbs(conformanceDir);
  const actualFixtures = fs
    .readdirSync(fixtureDirAbs)
    .filter((name) => name.endsWith(".json"))
    .sort();
  assertDeepEqual(
    actualFixtures,
    OUTPUT_MODE_CONSISTENCY.conformanceFixtures,
    "conformance fixture directory must match validator index",
  );

  const conformanceTests = readText(
    "crates/docnav-readable/tests/conformance_tests.rs",
  );
  const loadedFixtures = [
    ...conformanceTests.matchAll(
      /load_vector!\(\s*"fixtures\/conformance\/([^"]+\.json)"\s*\)/gu,
    ),
  ].map((match) => match[1]);
  assertDeepEqual(
    sortedUnique(loadedFixtures),
    OUTPUT_MODE_CONSISTENCY.conformanceFixtures,
    "each conformance fixture must be consumed by exactly one test",
  );
  assert(
    loadedFixtures.length === sortedUnique(loadedFixtures).length,
    "conformance tests must not load the same fixture more than once",
  );
  assert(
    conformanceTests.includes("checked_add(byte_length_usize)") &&
      conformanceTests.includes("starts_with(end_marker_bytes)") &&
      !conformanceTests.includes(".find(&end_marker)"),
    "conformance test parser must consume declared byte length before checking end marker",
  );

  const markerFixture = readJson(
    path.posix.join(conformanceDir, "12_block_marker_in_body.json"),
  );
  const markerPayload = markerFixture.input?.content;
  assert(
    typeof markerPayload === "string" &&
      markerPayload.includes("[block /content bytes=1]") &&
      markerPayload.includes("[endblock /content]"),
    "fixture 12 must contain same-pointer marker-looking text in the payload",
  );
  const exactBlockAssertion = markerFixture.assertions.find(
    (assertion: any) =>
      assertion.type === "block" &&
      assertion.pointer === "/content" &&
      assertion.payload === markerPayload,
  );
  assert(
    exactBlockAssertion,
    "fixture 12 must assert exact payload restoration",
  );
  assert(
    exactBlockAssertion.byte_length ===
      Buffer.byteLength(markerPayload, "utf8"),
    "fixture 12 exact payload assertion must include matching byte_length",
  );
}

export function validateOutputModeConsistency() {
  validateRustOutputModeEnums();
  validateOutputModeSmokeMatrices();
  validateConformanceFixtures();
  console.log(
    `document output mode consistency ok: ${DOCUMENT_OUTPUT_MODES.join(", ")}`,
  );
}
