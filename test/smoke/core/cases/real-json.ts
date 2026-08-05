import fs from "node:fs";
import path from "node:path";

import type { CommandRecord } from "../../../tools/smoke-harness.ts";

import { copyDocumentFixture, createProject, type SmokeProject } from "../fixtures.ts";
import {
  runCli,
  runSuccessfulJsonCase,
  validateSchema
} from "../harness.ts";
import {
  expect,
  expectExit,
  expectJsonObject,
  expectNoProtocolEnvelope,
  expectObjectArray,
  expectProtocolFailure,
  expectProtocolSuccess,
  expectReadableViewBlockRestoresField,
  expectStderrEmpty,
  expectString,
  expectStringArray,
  parseJson,
  parseReadableViewHeader
} from "../assertions.ts";
import type { JsonRecord } from "../assertions.ts";
import { exitCodes } from "../config.ts";

const jsonAdapterId = "docnav-json";
const jsonDocumentPath = "docs/navigation.json";
const jsoncAutomaticJsonPath = "docs/comment-navigation.json";
const jsoncAutomaticPath = "docs/comment-navigation.jsonc";
const jsoncExplicitPath = "docs/comment-navigation.md";
const jsonldAutomaticPath = "docs/navigation.jsonld";
const pipfileLockAutomaticPath = "docs/Pipfile.lock";
const invalidSarifAutomaticPath = "docs/invalid.sarif";
const jsonFormatExtensions = [
  ".json",
  ".code-workspace",
  ".jsonc",
  ".code-snippets",
  ".jsonld",
  ".geojson",
  ".har",
  ".webmanifest",
  ".ipynb",
  ".sarif"
] as const;
const jsonFormatFilenames = [
  ".prettierrc",
  ".watchmanconfig",
  "Pipfile.lock",
  "deno.lock"
] as const;
const jsonFormatContentTypes = ["application/json", "application/jsonc"] as const;
const directCommentRef = "json:comments:#/value";
const tailCommentRef = "json:tail-comments:#";
const directCommentContent = "// direct-needle\n1";
const tailCommentContent = `// tail-needle
{
  "value": 1
}`;
const specialRef = "json:#/a~1b~0caf%C3%A9%01";
const specialContent = `{
  "zeta": 1.2300e+04,
  "alpha": "needle source"
}`;

export function createRealJsonTasks() {
  return [
    {
      id: "CORE-JSON-NAV-001",
      label: "CORE-JSON-NAV-001 JSON selection and navigation roundtrip",
      run: testJsonSelectionAndNavigationRoundtrip
    },
    {
      id: "CORE-JSON-FAIL-001",
      label: "CORE-JSON-FAIL-001 JSON ref and document failure classification",
      run: testJsonFailureClassification
    }
  ];
}

async function testJsonSelectionAndNavigationRoundtrip() {
  const project = jsonFixtureProject("real-json-navigation");

  await assertAdapterRegistry(project);
  await assertJsoncSelectionAndNavigation(project);
  await assertExpandedPathnameRoundtrips(project);
  const outlineRef = await assertAutomaticOutline(project);
  const content = await assertExplicitRead(project, outlineRef);
  const findRef = await assertFindSourceLocations(project);
  await assertFindRefRead(project, findRef);
  await assertGenericReadableView(project, outlineRef, content);
}

async function testJsonFailureClassification() {
  const project = jsonFixtureProject("real-json-failures");

  await assertInvalidArrayRef(project);
  await assertMissingArrayRef(project);
  await assertSelectedInvalidJsonReasons(project);
  await assertExpandedPathnameInvalidNoFallback(project);
}

function jsonFixtureProject(name: string) {
  const project = createProject(name, { normalDocument: false });
  copyDocumentFixture(
    project,
    "json-navigation.json",
    jsonDocumentPath
  );
  for (const relativePath of [
    jsoncAutomaticJsonPath,
    jsoncAutomaticPath,
    jsoncExplicitPath
  ]) {
    copyDocumentFixture(project, "jsonc-navigation.jsonc", relativePath);
  }
  for (const relativePath of [
    jsonldAutomaticPath,
    pipfileLockAutomaticPath
  ]) {
    copyDocumentFixture(project, "json-navigation.json", relativePath);
  }
  return project;
}

async function assertAdapterRegistry(project: SmokeProject) {
  const record = await runCli(
    "CORE-JSON-NAV-001 adapter list includes ordered Markdown and JSON adapters",
    ["adapter", "list"],
    { project }
  );
  expectExit(record, 0);
  expectStderrEmpty(record);
  const json = parseJson(record);
  const adapters = expectObjectArray(
    record,
    json.adapters,
    "adapter list returns adapter objects"
  );
  const requiredIds = ["docnav-markdown", jsonAdapterId];
  const listedRequiredIds = adapters
    .map((adapter) => adapter.id)
    .filter((id) => requiredIds.includes(String(id)));
  expect(
    record,
    JSON.stringify(listedRequiredIds) === JSON.stringify(requiredIds),
    "adapter list contains Markdown then JSON"
  );
  for (const id of requiredIds) {
    const adapter = expectJsonObject(
      record,
      adapters.find((candidate) => candidate.id === id),
      `adapter list contains ${id}`
    );
    expect(
      record,
      adapter.implementation_source === "core_static",
      `${id} is linked from the core static registry`
    );
  }
  const jsonAdapter = expectJsonObject(
    record,
    adapters.find((candidate) => candidate.id === jsonAdapterId),
    "adapter list contains the JSON descriptor"
  );
  const formats = expectObjectArray(
    record,
    jsonAdapter.formats,
    "JSON descriptor formats are objects"
  );
  assertJsonFormatDescriptor(record, formats);
}

function assertJsonFormatDescriptor(
  record: CommandRecord,
  formats: readonly JsonRecord[]
) {
  expect(record, formats.length === 1, "JSON descriptor has one format");
  const format = expectJsonObject(
    record,
    formats[0],
    "JSON descriptor format is an object"
  );
  expect(record, format.id === "json", "JSON descriptor format id is json");
  expectExactStringArray(
    record,
    format.extensions,
    jsonFormatExtensions,
    "JSON descriptor extensions"
  );
  expectExactStringArray(
    record,
    format.filenames,
    jsonFormatFilenames,
    "JSON descriptor exact filenames"
  );
  expectExactStringArray(
    record,
    format.content_types,
    jsonFormatContentTypes,
    "JSON descriptor content types"
  );
}

function expectExactStringArray(
  record: CommandRecord,
  value: unknown,
  expected: readonly string[],
  label: string
) {
  const actual = expectStringArray(record, value, `${label} are strings`);
  expect(
    record,
    actual.length === expected.length &&
      expected.every((item, index) => actual[index] === item),
    `${label} preserve the exact manifest order`
  );
}

async function assertJsoncSelectionAndNavigation(project: SmokeProject) {
  const { record, json } = await runProtocolSuccess(
    "CORE-JSON-NAV-001 automatic .json accepts JSONC outline grammar",
    ["outline", jsoncAutomaticJsonPath, "--auto-read", "disabled"],
    project,
    "outline"
  );
  const result = expectJsonObject(
    record,
    json.result,
    "JSONC outline result is an object"
  );
  const entries = expectObjectArray(
    record,
    result.entries,
    "JSONC outline entries are objects"
  );
  expect(
    record,
    JSON.stringify(entries.map((entry) => entry.ref)) ===
      JSON.stringify([directCommentRef, tailCommentRef]),
    "automatic .json outline preserves opaque direct and tail comment refs"
  );

  for (const [ref, expectedContent] of [
    [directCommentRef, directCommentContent],
    [tailCommentRef, tailCommentContent]
  ] as const) {
    const read = await runProtocolSuccess(
      `CORE-JSON-NAV-001 automatic JSONC read preserves ${ref}`,
      ["read", jsoncAutomaticJsonPath, "--ref", ref],
      project,
      "read"
    );
    const readResult = expectJsonObject(
      read.record,
      read.json.result,
      "comment read result is an object"
    );
    expect(read.record, readResult.ref === ref, "comment read preserves the opaque input ref");
    expect(
      read.record,
      readResult.content === expectedContent,
      "comment read returns the selected projection"
    );
    expect(
      read.record,
      readResult.content_type === "application/jsonc",
      "comment read exposes application/jsonc"
    );
  }

  const explicit = await runProtocolSuccess(
    "CORE-JSON-NAV-001 explicit JSON adapter accepts the same JSONC grammar",
    ["read", jsoncExplicitPath, "--adapter", jsonAdapterId, "--ref", directCommentRef],
    project,
    "read"
  );
  const explicitResult = expectJsonObject(
    explicit.record,
    explicit.json.result,
    "explicit JSONC read result is an object"
  );
  expect(
    explicit.record,
    explicitResult.content === directCommentContent,
    "explicit selection uses the automatic JSONC grammar"
  );

  const direct = await runProtocolSuccess(
    "CORE-JSON-NAV-001 unique direct-comment find auto-read protocol-json",
    ["find", jsoncAutomaticJsonPath, "--query", "direct-needle"],
    project,
    "find"
  );
  assertCommentAutoRead(direct.record, direct.json, directCommentRef, directCommentContent);

  const readable = await runCli(
    "CORE-JSON-NAV-001 automatic .jsonc unique tail-comment auto-read readable-view",
    ["find", jsoncAutomaticPath, "--query", "tail-needle", "--output", "readable-view"],
    { project }
  );
  expectExit(readable, 0);
  expectStderrEmpty(readable);
  const header = parseReadableViewHeader(readable);
  expectNoProtocolEnvelope(readable, header);
  const autoRead = expectJsonObject(
    readable,
    header.auto_read,
    "readable JSONC find includes auto_read"
  );
  const nestedRead = expectJsonObject(
    readable,
    autoRead.read,
    "readable JSONC auto_read includes nested read"
  );
  expect(readable, nestedRead.ref === tailCommentRef, "readable auto-read preserves the tail ref");
  expect(readable, nestedRead.content_type === "application/jsonc", "readable auto-read preserves JSONC content type");
  expectReadableViewBlockRestoresField(
    readable,
    readable.stdout,
    "/auto_read/read/content",
    tailCommentContent
  );
}

async function assertExpandedPathnameRoundtrips(project: SmokeProject) {
  for (const [label, relativePath] of [
    [".jsonld suffix", jsonldAutomaticPath],
    ["Pipfile.lock exact filename", pipfileLockAutomaticPath]
  ] as const) {
    const outlined = await runProtocolSuccess(
      `CORE-JSON-NAV-001 automatic ${label} outline`,
      ["outline", relativePath, "--auto-read", "disabled"],
      project,
      "outline"
    );
    const outlineResult = expectJsonObject(
      outlined.record,
      outlined.json.result,
      `${label} outline result is an object`
    );
    const entries = expectObjectArray(
      outlined.record,
      outlineResult.entries,
      `${label} outline entries are objects`
    );
    const selectedEntry = expectJsonObject(
      outlined.record,
      entries.find((entry) => entry.ref === specialRef),
      `${label} outline returns the generic JSON ref`
    );
    const ref = expectString(
      outlined.record,
      selectedEntry.ref,
      `${label} outline ref is readable`
    );

    const read = await runProtocolSuccess(
      `CORE-JSON-NAV-001 automatic ${label} outline-ref read`,
      ["read", relativePath, "--ref", ref],
      project,
      "read"
    );
    const readResult = expectJsonObject(
      read.record,
      read.json.result,
      `${label} read result is an object`
    );
    expect(read.record, readResult.ref === ref, `${label} read preserves the outline ref`);
    expect(
      read.record,
      readResult.content === specialContent,
      `${label} read uses generic structural JSON content`
    );
    expect(
      read.record,
      readResult.content_type === "application/json",
      `${label} read keeps source-derived JSON content type`
    );
  }
}

function assertCommentAutoRead(
  record: CommandRecord,
  json: JsonRecord,
  ref: string,
  content: string
) {
  const result = expectJsonObject(
    record,
    json.result,
    "comment find result is an object"
  );
  const matches = expectObjectArray(
    record,
    result.matches,
    "comment find matches are objects"
  );
  expect(
    record,
    matches.length === 1 && matches[0]?.ref === ref,
    "comment find returns one opaque ref"
  );
  const autoRead = expectJsonObject(
    record,
    result.auto_read,
    "unique comment ref triggers auto_read"
  );
  expect(record, autoRead.reason === "unique_ref", "comment auto_read reports unique_ref");
  const read = expectJsonObject(record, autoRead.read, "comment auto_read includes nested read");
  expect(record, read.ref === ref, "nested read preserves the comment ref");
  expect(record, read.content === content, "nested read returns the comment projection");
  expect(record, read.content_type === "application/jsonc", "nested read preserves JSONC content type");
  expectJsonObject(record, read.cost, "nested read preserves cost facts");
  expect(record, read.page === null, "nested read preserves terminal page facts");
}

async function assertAutomaticOutline(project: SmokeProject): Promise<string> {
  const { record, json } = await runProtocolSuccess(
    "CORE-JSON-NAV-001 automatic JSON outline protocol-json",
    ["outline", jsonDocumentPath],
    project,
    "outline"
  );
  const result = expectJsonObject(
    record,
    json.result,
    "JSON outline result is an object"
  );
  expect(record, result.kind === "structured", "JSON outline is structured");
  const entries = expectObjectArray(
    record,
    result.entries,
    "JSON outline entries are objects"
  );
  const emptyKeyEntry = expectJsonObject(
    record,
    entries.find((entry) => entry.ref === "json:#/"),
    "JSON outline includes the empty-key entry"
  );
  expect(
    record,
    emptyKeyEntry.label === "\"\"",
    "empty-key entry uses the visible empty label"
  );

  const specialEntry = expectJsonObject(
    record,
    entries.find((entry) => entry.ref === specialRef),
    "JSON outline includes the special-key entry"
  );
  const ref = expectString(
    record,
    specialEntry.ref,
    "special-key entry exposes a ref"
  );
  expect(record, ref === specialRef, "special-key ref is canonical");
  expect(
    record,
    /^[\x20-\x7e]+$/.test(ref),
    "special-key ref is ASCII-safe"
  );
  expect(record, specialEntry.kind === "object", "special-key entry is an object");
  expect(
    record,
    entries.every((entry) => !Object.hasOwn(entry, "display")),
    "protocol outline entries omit readable display"
  );
  return ref;
}

async function assertExplicitRead(
  project: SmokeProject,
  ref: string
): Promise<string> {
  const { record, json } = await runProtocolSuccess(
    "CORE-JSON-NAV-001 explicit JSON read protocol-json",
    ["read", jsonDocumentPath, "--adapter", jsonAdapterId, "--ref", ref],
    project,
    "read"
  );
  const result = expectJsonObject(
    record,
    json.result,
    "JSON read result is an object"
  );
  expect(record, result.ref === ref, "JSON read preserves the outline ref");
  expect(
    record,
    result.content_type === "application/json",
    "JSON read exposes application/json"
  );
  const content = expectString(
    record,
    result.content,
    "JSON read content is a string"
  );
  expect(
    record,
    content === specialContent,
    "JSON read preserves source order and raw number spelling"
  );
  expectJsonObject(record, result.cost, "JSON read exposes raw cost facts");
  expect(
    record,
    !Object.hasOwn(result, "display"),
    "protocol read omits readable display"
  );
  return content;
}

async function assertFindSourceLocations(
  project: SmokeProject
): Promise<string> {
  const { record, json } = await runProtocolSuccess(
    "CORE-JSON-NAV-001 JSON find source occurrences",
    ["find", jsonDocumentPath, "--query", "needle"],
    project,
    "find"
  );
  const result = expectJsonObject(
    record,
    json.result,
    "JSON find result is an object"
  );
  const matches = expectObjectArray(
    record,
    result.matches,
    "JSON find matches are objects"
  );
  expect(record, matches.length === 2, "JSON find returns both source occurrences");
  expect(
    record,
    JSON.stringify(matches.map((match) => match.ref)) === JSON.stringify([
      `${specialRef}/alpha`,
      "json:#/array/0/message"
    ]),
    "JSON find maps source occurrences to readable refs in source order"
  );
  const lineStarts = matches.map((match, index) => {
    const location = expectJsonObject(
      record,
      match.location,
      `JSON find match ${index + 1} has a source location`
    );
    return location.line_start;
  });
  expect(
    record,
    JSON.stringify(lineStarts) === JSON.stringify([5, 8]),
    "JSON find preserves fixture source lines"
  );
  expect(
    record,
    matches.every(
      (match) => match.kind === "match" && !Object.hasOwn(match, "display")
    ),
    "protocol find keeps raw match facts without readable display"
  );
  return expectString(
    record,
    matches[0]?.ref,
    "first JSON find match exposes a ref"
  );
}

async function assertFindRefRead(project: SmokeProject, ref: string) {
  const { record, json } = await runProtocolSuccess(
    "CORE-JSON-NAV-001 read JSON find ref",
    ["read", jsonDocumentPath, "--ref", ref],
    project,
    "read"
  );
  const result = expectJsonObject(
    record,
    json.result,
    "JSON find-ref read result is an object"
  );
  expect(record, result.ref === ref, "JSON read preserves the find ref");
  expect(
    record,
    result.content === "\"needle source\"",
    "JSON find ref reads the matched value"
  );
}

async function assertGenericReadableView(
  project: SmokeProject,
  ref: string,
  content: string
) {
  const record = await runCli(
    "CORE-JSON-NAV-001 JSON generic readable-view",
    ["read", jsonDocumentPath, "--ref", ref, "--output", "readable-view"],
    { project }
  );
  expectExit(record, 0);
  expectStderrEmpty(record);
  const header = parseReadableViewHeader(record);
  expectNoProtocolEnvelope(record, header);
  expect(record, header.ref === ref, "generic readable-view preserves the JSON ref");
  expect(
    record,
    header.content_type === "application/json",
    "generic readable-view preserves JSON content type"
  );
  expect(
    record,
    typeof header.cost === "string",
    "generic readable-view derives common cost text"
  );
  expect(
    record,
    !Object.hasOwn(header, "display"),
    "generic readable-view does not claim a JSON-specific display"
  );
  expectReadableViewBlockRestoresField(
    record,
    record.stdout,
    "/content",
    content
  );
}

async function assertInvalidArrayRef(project: SmokeProject) {
  const { record, error } = await runProtocolFailure(
    "CORE-JSON-FAIL-001 noncanonical array ref",
    ["read", jsonDocumentPath, "--ref", "json:#/array/01"],
    project,
    "read",
    "REF_INVALID",
    exitCodes.documentRefFormat
  );
  const details = expectJsonObject(
    record,
    error.details,
    "REF_INVALID details are an object"
  );
  expect(
    record,
    details.ref === "json:#/array/01",
    "REF_INVALID preserves the rejected ref"
  );
  expect(
    record,
    typeof details.reason === "string" && details.reason.length > 0,
    "REF_INVALID explains the grammar failure"
  );
}

async function assertMissingArrayRef(project: SmokeProject) {
  const { record, error } = await runProtocolFailure(
    "CORE-JSON-FAIL-001 missing canonical array ref",
    ["read", jsonDocumentPath, "--ref", "json:#/array/9"],
    project,
    "read",
    "REF_NOT_FOUND",
    exitCodes.documentRefFormat
  );
  const details = expectJsonObject(
    record,
    error.details,
    "REF_NOT_FOUND details are an object"
  );
  expect(
    record,
    details.ref === "json:#/array/9",
    "REF_NOT_FOUND preserves the missing ref"
  );
}

async function assertSelectedInvalidJsonReasons(project: SmokeProject) {
  const cases = [
    ["syntax", "{\"value\":}", "JSON_SYNTAX_INVALID"],
    ["trailing", "{} trailing", "JSON_TRAILING_INPUT"],
    ["duplicate", "{\"a\":1,\"\\u0061\":2}", "JSON_DUPLICATE_MEMBER"],
    [
      "depth",
      `${"[".repeat(128)}[]${"]".repeat(128)}`,
      "JSON_MAXIMUM_DEPTH_EXCEEDED"
    ]
  ] as const;

  for (const [name, content, reason] of cases) {
    const relativePath = `docs/invalid-${name}.md`;
    const absolutePath = path.join(project.root, relativePath);
    fs.writeFileSync(absolutePath, content, "utf8");
    const normalizedPath = absolutePath.replaceAll(path.sep, "/");
    const { record, error } = await runProtocolFailure(
      `CORE-JSON-FAIL-001 explicit JSON ${name} failure`,
      ["outline", relativePath, "--adapter", jsonAdapterId],
      project,
      "outline",
      "DOCUMENT_CONTENT_INVALID",
      exitCodes.documentRefFormat
    );
    const details = expectJsonObject(
      record,
      error.details,
      `selected JSON ${name} details are an object`
    );
    expect(
      record,
      Object.keys(details).sort().join(",") === "path,reason",
      `selected JSON ${name} details contain only path and reason`
    );
    expect(record, details.path === normalizedPath, `selected JSON ${name} preserves the normalized path`);
    expect(record, details.reason === reason, `selected JSON ${name} uses ${reason}`);
  }
}

async function assertExpandedPathnameInvalidNoFallback(project: SmokeProject) {
  const absolutePath = path.join(project.root, invalidSarifAutomaticPath);
  fs.writeFileSync(absolutePath, "# not JSON\n", "utf8");
  const normalizedPath = absolutePath.replaceAll(path.sep, "/");
  const { record, error } = await runProtocolFailure(
    "CORE-JSON-FAIL-001 automatic .sarif selected JSON failure",
    ["outline", invalidSarifAutomaticPath],
    project,
    "outline",
    "DOCUMENT_CONTENT_INVALID",
    exitCodes.documentRefFormat
  );
  const details = expectJsonObject(
    record,
    error.details,
    "automatic .sarif failure details are an object"
  );
  expect(record, error.owner === "adapter", "automatic .sarif failure remains JSON-owned");
  expect(
    record,
    Object.keys(details).sort().join(",") === "path,reason",
    "automatic .sarif failure details contain only path and reason"
  );
  expect(record, details.path === normalizedPath, "automatic .sarif failure preserves path");
  expect(
    record,
    details.reason === "JSON_SYNTAX_INVALID",
    "automatic .sarif failure does not reroute or fallback"
  );
}

function runProtocolSuccess(
  name: string,
  args: string[],
  project: SmokeProject,
  operation: string
) {
  return runSuccessfulJsonCase(name, [...args, "--output", "protocol-json"], {
    schema: "protocolResponse",
    commandOptions: { project },
    check: (record, json) => expectProtocolSuccess(record, json, operation)
  });
}

async function runProtocolFailure(
  name: string,
  args: string[],
  project: SmokeProject,
  operation: string,
  code: string,
  exitCode: number
): Promise<{
  error: JsonRecord;
  record: CommandRecord;
}> {
  const record = await runCli(
    name,
    [...args, "--output", "protocol-json"],
    { project }
  );
  expectExit(record, exitCode);
  expectStderrEmpty(record);
  const json = parseJson(record);
  validateSchema(record, "protocolResponse", json);
  const error = expectProtocolFailure(record, json, operation, code);
  return { error, record };
}
