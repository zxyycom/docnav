import { assert, assertDeepEqual } from "../assertions.ts";
import { DOCUMENT_OUTPUT_MODES, OUTPUT_MODE_CONSISTENCY } from "../config.ts";
import { readText } from "../document/markdown-docs.ts";

export function validateRustOutputModes(): void {
  assertCoreOutputModeEnum(readText(OUTPUT_MODE_CONSISTENCY.coreOutputModeRust));
  assertAdapterDirectOutputModeEnum(
    readText(OUTPUT_MODE_CONSISTENCY.adapterOutputModeRust)
  );
  assertCoreDocumentOutputHelp(
    readText(OUTPUT_MODE_CONSISTENCY.coreOutputHelpRust)
  );
  assertAdapterDirectDocumentOutputHelp(
    readText(OUTPUT_MODE_CONSISTENCY.adapterOutputHelpRust)
  );
}

function assertDocumentOutputModeSet(values: string[], label: string): void {
  assertDeepEqual(
    values,
    DOCUMENT_OUTPUT_MODES,
    `${label} must be exactly ${DOCUMENT_OUTPUT_MODES.join(", ")}`
  );
}

function extractRustStringArray(source: string, constName: string, label: string): string[] {
  const pattern = new RegExp(
    `const\\s+${constName}\\s*:[^=]+?=\\s*&\\s*\\[([^\\]]*)\\]`,
    "su"
  );
  const match = source.match(pattern);
  assert(match, `${label} must declare ${constName}`);
  return [...match[1].matchAll(/"([^"]+)"/gu)].map((item) => item[1]);
}

function extractOutputValueConstants(source: string, label: string): string[] {
  const constants: Record<string, string> = {};
  for (const match of source.matchAll(
    /const\s+(PROTOCOL_JSON|READABLE_JSON|READABLE_VIEW)\s*:\s*&str\s*=\s*"([^"]+)"/gu
  )) {
    constants[match[1]] = match[2];
  }
  const values = [
    constants.READABLE_VIEW,
    constants.READABLE_JSON,
    constants.PROTOCOL_JSON
  ];
  assert(
    values.every((value) => typeof value === "string"),
    `${label} must declare readable-view/readable-json/protocol-json constants`
  );
  return values;
}

function assertOutputHelpShape(source: string, label: string): void {
  const valueName = "readable-view|readable-json|protocol-json";
  assert(
    source.includes(`"${valueName}"`),
    `${label} must advertise ${valueName}`
  );
  assert(
    !source.includes('.value_name("text|readable-json|protocol-json")') &&
      !source.includes(
        '.value_name("readable-view|readable-json|protocol-json|text")'
      ),
    `${label} must not advertise legacy text as a document output mode`
  );
}

function assertCoreOutputModeEnum(source: string): void {
  assert(
    source.includes("pub enum OutputMode"),
    "core OutputMode enum must remain declared in cli/command_model.rs"
  );
  assertDocumentOutputModeSet(
    extractRustStringArray(source, "ACCEPTED_VALUES", "core OutputMode"),
    "core OutputMode::ACCEPTED_VALUES"
  );
  assert(
    source.includes("ACCEPTED_VALUES") &&
      DOCUMENT_OUTPUT_MODES.every((mode) => source.includes(`"${mode}"`)),
    "core OutputMode::ACCEPTED_VALUES must list the three document output modes"
  );
  assert(
    !source.includes("OutputMode::Text"),
    "core OutputMode must not retain Text variant use"
  );
  assert(
    !/^\s*Text\s*,/mu.test(source),
    "core OutputMode enum must not declare Text"
  );
}

function assertAdapterDirectOutputModeEnum(source: string): void {
  assert(
    source.includes("pub enum DirectOutputMode"),
    "adapter DirectOutputMode enum must remain declared"
  );
  for (const variant of ["ReadableView", "ReadableJson", "ProtocolJson"]) {
    assert(
      source.includes(variant),
      `adapter DirectOutputMode missing ${variant}`
    );
  }
  assert(
    !source.includes("DirectOutputMode::Text") &&
      !/^\s*Text\s*,/mu.test(source),
    "adapter DirectOutputMode must not retain Text"
  );
}

function assertCoreDocumentOutputHelp(source: string): void {
  assertDocumentOutputModeSet(
    extractOutputValueConstants(source, "core parser output_values"),
    "core parser output_values"
  );
  assertOutputHelpShape(source, "core document output help");
}

function assertAdapterDirectDocumentOutputHelp(source: string): void {
  assertDocumentOutputModeSet(
    extractOutputValueConstants(source, "adapter direct output_values"),
    "adapter direct output_values"
  );
  assertOutputHelpShape(source, "adapter direct document output help");
  assert(
    source.includes('.value_name("protocol-json")'),
    "adapter protocol-only help must stay protocol-json only"
  );
}
