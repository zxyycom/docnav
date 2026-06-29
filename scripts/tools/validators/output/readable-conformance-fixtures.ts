import fs from "node:fs";
import path from "node:path";

import { assert, assertDeepEqual } from "../assertions.ts";
import { OUTPUT_MODE_CONSISTENCY } from "../config.ts";
import { readText } from "../document/markdown-docs.ts";
import { readJson } from "../json/files.ts";
import { toAbs } from "../repo/paths.ts";
import { isRecord } from "../../type-guards.ts";

export function validateReadableConformanceFixtures(): void {
  const conformanceDir = OUTPUT_MODE_CONSISTENCY.conformanceDir;

  assertConformanceFixturesExist(conformanceDir);
  assertConformanceReadmeIndexesFixtures();
  assertWarningFixtureOmitsLegacyTextMode(conformanceDir);
  assertConformanceDirectoryMatchesIndex(conformanceDir);
  assertMarkerFixtureRestoresPayload(conformanceDir);
}

function assertConformanceFixturesExist(conformanceDir: string): void {
  for (const fixture of OUTPUT_MODE_CONSISTENCY.conformanceFixtures) {
    const relPath = path.posix.join(conformanceDir, fixture);
    assert(
      fs.existsSync(toAbs(relPath)),
      `missing readable-view conformance fixture ${relPath}`
    );
  }
}

function assertConformanceReadmeIndexesFixtures(): void {
  const readme = readText(OUTPUT_MODE_CONSISTENCY.conformanceReadme);
  for (const fixture of OUTPUT_MODE_CONSISTENCY.conformanceFixtures) {
    assert(
      readme.includes(fixture),
      `conformance README must index ${fixture}`
    );
  }
}

function assertWarningFixtureOmitsLegacyTextMode(conformanceDir: string): void {
  const warningFixture = readText(
    path.posix.join(conformanceDir, "13_warning.json")
  );
  assert(
    !warningFixture.includes("--text"),
    "warning conformance fixture must not describe removed text output as a current mode"
  );
}

function assertConformanceDirectoryMatchesIndex(conformanceDir: string): void {
  const fixtureDirAbs = toAbs(conformanceDir);
  const actualFixtures = fs
    .readdirSync(fixtureDirAbs)
    .filter((name) => name.endsWith(".json"))
    .sort();
  assertDeepEqual(
    actualFixtures,
    OUTPUT_MODE_CONSISTENCY.conformanceFixtures,
    "conformance fixture directory must match validator index"
  );
}

function assertMarkerFixtureRestoresPayload(conformanceDir: string): void {
  const markerFixture = readJson(
    path.posix.join(conformanceDir, "12_block_marker_in_body.json")
  );
  assert(isRecord(markerFixture), "fixture 12 must be a JSON object");
  const markerInput = markerFixture.input;
  assert(isRecord(markerInput), "fixture 12 input must be a JSON object");
  const markerPayload = markerInput.content;
  assert(
    typeof markerPayload === "string" &&
      markerPayload.includes("[block /content bytes=1]") &&
      markerPayload.includes("[endblock /content]"),
    "fixture 12 must contain same-pointer marker-looking text in the payload"
  );
  const assertions = markerFixture.assertions;
  assert(Array.isArray(assertions), "fixture 12 assertions must be an array");
  const exactBlockAssertion = assertions.find(
    (assertion) =>
      isRecord(assertion) &&
      assertion.type === "block" &&
      assertion.pointer === "/content" &&
      assertion.payload === markerPayload
  );
  assert(
    isRecord(exactBlockAssertion),
    "fixture 12 must assert exact payload restoration"
  );
  assert(
    exactBlockAssertion.byte_length ===
      Buffer.byteLength(markerPayload, "utf8"),
    "fixture 12 exact payload assertion must include matching byte_length"
  );
}
