import fs from "node:fs";
import path from "node:path";

import { assert, assertDeepEqual, readJson, toAbs } from "./fs-utils.mjs";
import { DOCUMENT_OUTPUT_MODES, OUTPUT_MODE_CONSISTENCY } from "./config.mjs";
import {
  listMainMarkdownDocs,
  readText,
  sortedUnique,
} from "./document-files.mjs";

function assertIncludesDocumentOutputModes(relPath) {
  const text = readText(relPath);
  for (const mode of DOCUMENT_OUTPUT_MODES) {
    assert(
      text.includes(mode),
      `${relPath} must mention document output mode ${mode}`,
    );
  }
}

function assertDocumentOutputModeSet(values, label) {
  assertDeepEqual(
    values,
    DOCUMENT_OUTPUT_MODES,
    `${label} must be exactly ${DOCUMENT_OUTPUT_MODES.join(", ")}`,
  );
}

function extractRustStringArray(source, constName, label) {
  const pattern = new RegExp(
    `const\\s+${constName}\\s*:[^=]+?=\\s*&\\s*\\[([^\\]]*)\\]`,
    "su",
  );
  const match = source.match(pattern);
  assert(match, `${label} must declare ${constName}`);
  return [...match[1].matchAll(/"([^"]+)"/gu)].map((item) => item[1]);
}

function extractOutputValueConstants(source, label) {
  const constants = {};
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

function assertOutputHelpShape(source, label) {
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

function validateDocumentOutputModeDocs() {
  const bannedLegacyDeclarations = [
    { pattern: /默认文本/u, label: "默认文本" },
    { pattern: /`text`\s*输出模式/u, label: "`text` 输出模式" },
    { pattern: /text\s*输出模式/u, label: "text 输出模式" },
    { pattern: /text output mode/iu, label: "text output mode" },
    { pattern: /human text formatting/iu, label: "human text formatting" },
    { pattern: /`text`\s+vs\s+JSON/iu, label: "`text` vs JSON" },
    {
      pattern: /readable JSON 与 text output/iu,
      label: "readable JSON 与 text output",
    },
  ];

  for (const relPath of [
    ...OUTPUT_MODE_CONSISTENCY.currentDocs,
    ...OUTPUT_MODE_CONSISTENCY.projectSkillDocs,
  ]) {
    const text = readText(relPath);
    for (const mode of DOCUMENT_OUTPUT_MODES) {
      assert(text.includes(mode), `${relPath} must stay aligned with ${mode}`);
    }
    for (const { pattern, label } of bannedLegacyDeclarations) {
      assert(
        !pattern.test(text),
        `${relPath} contains legacy document output wording: ${label}`,
      );
    }
    assert(
      !/readable-view\|readable-json\|protocol-json\|[a-z0-9_-]+/iu.test(text),
      `${relPath} advertises an extra document output mode next to current modes`,
    );
  }
}

function validateConfigTemplateDocs() {
  // Catch claims that config can change readable output text/copy/guidance/templates.
  // Verbs describe config exercising power over readable output; objects are the
  // text/copy/guidance/template artifacts config must not control.
  // Negative lookbehind excludes negated/prohibitive forms (不得/不能/不可/不会/不)
  // so that "配置不得改变 protocol-json 字段" and "不改变阅读输出文案" are not
  // flagged — only positive claims of config power are violations.
  const claimPattern =
    /(配置|配置域)[^\n]{0,120}(?<!(?:不得|不能|不可|不会|不))(拥有|可以控制|可以通过|调整|读取|可以改变|可以修改|可以定制|可以自定义|改变|修改)[^\n]{0,80}(阅读输出文案|阅读输出文本|阅读文案|输出文案|guidance|阅读文本模板|输出文本模板|TextContent\s*模板|TextContent\s*包装文本|TextContent\s*包装模板|阅读输出\s*模板|header\s*文案|header\s*模板|tool\s*暴露策略)/u;
  for (const relPath of listMainMarkdownDocs()) {
    const text = readText(relPath);
    assert(
      !claimPattern.test(text),
      `${relPath} must not claim config can change readable output text/copy/guidance/templates`,
    );
    assert(
      !/用户修改配置即可生效/u.test(text),
      `${relPath} must not claim unimplemented readable text template config is live`,
    );
  }
}

function validateSchemaReadmeReadableViewBoundary() {
  const readme = readText("docs/schemas/README.md");
  const schemaRows = readme
    .split(/\r?\n/u)
    .filter((line) => /\[readable-[^\]]+\.schema\.json\]/u.test(line));

  for (const line of schemaRows) {
    assert(
      !/readable-view|header/u.test(line),
      "readable schema README rows must not bind readable-view headers to readable JSON schemas",
    );
  }

  assert(
    /CLI `readable-json` 和 MCP structuredContent/u.test(readme),
    "schema README must state readable schemas validate readable-json and MCP structuredContent",
  );
  assert(
    /readable-view header block refs、framing 和 payload 还原由 committed conformance vectors 验收/u.test(
      readme,
    ),
    "schema README must assign readable-view framing/header block refs to conformance vectors",
  );
}

function validateOutputModeSmokeMatrices() {
  for (const relPath of OUTPUT_MODE_CONSISTENCY.smokeMatrices) {
    assertIncludesDocumentOutputModes(relPath);
  }

  const coreHelp = readText(
    "scripts/docnav-core-cli-smoke/cases/config-management.mjs",
  );
  assert(
    coreHelp.includes("outline help does not mention text output mode"),
    "core smoke must assert document help omits text output mode",
  );
  const adapterHelp = readText(
    "scripts/docnav-markdown-cli-smoke/cases/cli-args.mjs",
  );
  assert(
    adapterHelp.includes("outline help does not mention text output mode"),
    "adapter smoke must assert document help omits text output mode",
  );

  const configSmoke = readText(
    "scripts/docnav-core-cli-smoke/cases/config-management.mjs",
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
    (assertion) =>
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
  validateDocumentOutputModeDocs();
  validateConfigTemplateDocs();
  validateSchemaReadmeReadableViewBoundary();
  validateOutputModeSmokeMatrices();
  validateConformanceFixtures();
  console.log(
    `document output mode consistency ok: ${DOCUMENT_OUTPUT_MODES.join(", ")}`,
  );
}
