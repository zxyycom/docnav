import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SCHEMA_VERSION = 1;
const DEFAULT_LIMIT = 20;
const MAX_LIMIT = 200;
const CLAIM_ID_PATTERN = /^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+){2,}-\d{3}$/;
const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
const FINGERPRINT_PATTERN = /^sha256:[0-9a-f]{64}$/;
const CLAIM_FILE_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*\.md$/;
const TEMPLATE_PATTERNS = [
  /^stable contract\.?$/i,
  /^the result is observable\.?$/i,
  /^this test verifies (?:the )?behavior described by the test name\.?$/i,
  /^the test passes when the implementation is correct\.?$/i
];

export function buildTestEvidenceProjection(options) {
  const paths = evidencePaths(options.workspaceRoot);
  const diagnostics = [];
  validateEvidenceRootLayout(paths, diagnostics);
  const inventory = loadInventory(paths.inventoryPath, diagnostics);
  const topicCatalog = loadTopicCatalog(paths.topicCatalogPath, diagnostics);
  const claims = inventory && topicCatalog
    ? loadClaims(paths, inventory, topicCatalog, diagnostics)
    : [];

  if (!inventory || !topicCatalog || diagnostics.some(({ blocking }) => blocking)) {
    return {
      diagnostics,
      projection: null
    };
  }

  return {
    diagnostics,
    projection: createProjection(inventory, topicCatalog, claims)
  };
}

export function validateTestEvidence(options) {
  const built = buildTestEvidenceProjection(options);
  const diagnostics = [...built.diagnostics];
  if (built.projection) {
    diagnostics.push(...validatePersistedIndex(
      evidencePaths(options.workspaceRoot).indexPath,
      built.projection
    ));
  }
  return reportFor(built.projection, diagnostics);
}

export function syncTestEvidenceIndex(options) {
  const built = buildTestEvidenceProjection(options);
  if (!built.projection || built.diagnostics.some(({ blocking }) => blocking)) {
    return syncResult(options.mode, null, built.diagnostics);
  }

  const indexPath = evidencePaths(options.workspaceRoot).indexPath;
  if (options.mode === "write") {
    assertDistinctIndexIdentity(indexPath, options.workspaceRoot);
    writeJsonAtomic(indexPath, built.projection);
    return syncResult("write", built.projection, []);
  }
  if (options.mode !== "check") {
    throw new Error(`unsupported sync mode: ${String(options.mode)}`);
  }
  return syncResult(
    "check",
    built.projection,
    validatePersistedIndex(indexPath, built.projection)
  );
}

export function queryTestEvidence(options) {
  const loaded = loadQueryProjection(options.workspaceRoot);
  if (!loaded.projection) {
    return {
      schemaVersion: SCHEMA_VERSION,
      status: "error",
      source: "memory",
      diagnostics: loaded.diagnostics,
      offset: normalizeOffset(options.offset),
      limit: normalizeLimit(options.limit),
      total: 0,
      items: []
    };
  }

  const offset = normalizeOffset(options.offset);
  const limit = normalizeLimit(options.limit);
  const items = createQueryItems(loaded.projection)
    .filter((item) => matchesKind(item, options.kind))
    .filter((item) => matchesExactFilters(item, options, loaded.projection))
    .filter((item) => matchesText(item, options.query))
    .sort(compareQueryItems);

  return {
    schemaVersion: SCHEMA_VERSION,
    status: "ok",
    source: loaded.source,
    diagnostics: loaded.diagnostics,
    offset,
    limit,
    total: items.length,
    items: items.slice(offset, offset + limit)
  };
}

export function showTestEvidence(options) {
  const loaded = loadQueryProjection(options.workspaceRoot);
  if (!loaded.projection) {
    return {
      schemaVersion: SCHEMA_VERSION,
      status: "error",
      source: "memory",
      diagnostics: loaded.diagnostics,
      item: null
    };
  }

  const matches = createQueryItems(loaded.projection)
    .filter((item) => item.id === options.id);
  const diagnostics = [...loaded.diagnostics];
  if (matches.length === 0) {
    diagnostics.push(diagnostic(
      "query.target-missing",
      "query",
      `no Entry or Claim has id ${options.id}`,
      { blocking: true }
    ));
  } else if (matches.length > 1) {
    diagnostics.push(diagnostic(
      "query.target-ambiguous",
      "query",
      `both an Entry and Claim have id ${options.id}`,
      { blocking: true }
    ));
  }

  return {
    schemaVersion: SCHEMA_VERSION,
    status: diagnostics.some(({ blocking }) => blocking) ? "error" : "ok",
    source: loaded.source,
    diagnostics,
    item: matches.length === 1 ? matches[0] : null
  };
}

export function listTestEvidenceTopics(options) {
  const diagnostics = [];
  const topicCatalog = loadTopicCatalog(
    evidencePaths(options.workspaceRoot).topicCatalogPath,
    diagnostics
  );
  return {
    schemaVersion: SCHEMA_VERSION,
    status: diagnostics.some(({ blocking }) => blocking) ? "error" : "ok",
    diagnostics,
    topics: topicCatalog?.topics ?? []
  };
}

export async function runTestEvidenceCatalogCli(argv = process.argv.slice(2)) {
  let parsed;
  try {
    parsed = parseCli(argv);
  } catch (error) {
    process.stderr.write(`${errorMessage(error)}\n`);
    return 2;
  }

  try {
    const result = runCliCommand(parsed);
    if (parsed.json) {
      process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    } else {
      process.stdout.write(formatHumanResult(parsed.command, result));
    }
    return result.status === "ok" ? 0 : 1;
  } catch (error) {
    if (parsed.json) {
      process.stdout.write(`${JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
        status: "error",
        diagnostics: [
          diagnostic("execution.failed", "query", errorMessage(error), {
            blocking: true
          })
        ]
      }, null, 2)}\n`);
    } else {
      process.stderr.write(`${errorMessage(error)}\n`);
    }
    return 1;
  }
}

function evidencePaths(workspaceRoot) {
  const root = path.resolve(workspaceRoot);
  const evidenceRoot = path.join(root, "docs", "test-evidence");
  return {
    workspaceRoot: root,
    evidenceRoot,
    claimsRoot: path.join(evidenceRoot, "claims"),
    inventoryPath: path.join(evidenceRoot, "native-test-inventory.json"),
    topicCatalogPath: path.join(evidenceRoot, "claim-topics.json"),
    indexPath: path.join(evidenceRoot, "test-evidence-index.json")
  };
}

function validateEvidenceRootLayout(paths, diagnostics) {
  if (!fs.existsSync(paths.evidenceRoot)) {
    diagnostics.push(diagnostic(
      "catalog.root-missing",
      "claim",
      "docs/test-evidence does not exist",
      { blocking: true, path: "docs/test-evidence" }
    ));
    return;
  }
  const allowed = new Set([
    "claim-topics.json",
    "claims",
    "native-test-inventory.json",
    "test-evidence-index.json"
  ]);
  for (const entry of fs.readdirSync(paths.evidenceRoot, { withFileTypes: true })) {
    if (!allowed.has(entry.name)) {
      diagnostics.push(diagnostic(
        "catalog.member-unknown",
        "claim",
        `unknown test evidence root member ${entry.name}`,
        {
          blocking: true,
          path: toSlash(path.join("docs", "test-evidence", entry.name))
        }
      ));
    }
  }
}

function loadInventory(inventoryPath, diagnostics) {
  const raw = readJson(inventoryPath, "inventory", diagnostics);
  if (!raw) {
    return null;
  }
  if (!isRecord(raw) || !hasExactKeys(
    raw,
    ["schemaVersion", "profile", "sourceRevision", "entries"]
  )) {
    diagnostics.push(invalidDiagnostic(
      "inventory.invalid",
      "inventory",
      inventoryPath,
      "inventory must have exactly schemaVersion, profile, sourceRevision and entries"
    ));
    return null;
  }
  if (raw.schemaVersion !== SCHEMA_VERSION) {
    diagnostics.push(invalidDiagnostic(
      "inventory.version-unsupported",
      "inventory",
      inventoryPath,
      `inventory schemaVersion must be ${SCHEMA_VERSION}`
    ));
  }
  const profile = normalizeProfile(raw.profile, inventoryPath, diagnostics);
  if (!FINGERPRINT_PATTERN.test(String(raw.sourceRevision))) {
    diagnostics.push(invalidDiagnostic(
      "inventory.source-revision-invalid",
      "inventory",
      inventoryPath,
      "inventory sourceRevision must be a sha256 fingerprint"
    ));
  }
  if (!Array.isArray(raw.entries)) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entries-invalid",
      "inventory",
      inventoryPath,
      "inventory entries must be an array"
    ));
    return null;
  }

  const entries = [];
  const seen = new Set();
  let previousKey = null;
  for (const [index, value] of raw.entries.entries()) {
    const entry = normalizeEntry(value, inventoryPath, index, diagnostics);
    if (!entry) {
      continue;
    }
    if (seen.has(entry.entryKey)) {
      diagnostics.push(diagnostic(
        "inventory.entry-duplicate",
        "inventory",
        `duplicate inventory entryKey ${entry.entryKey}`,
        {
          blocking: true,
          entryKey: entry.entryKey,
          path: relativeDisplayPath(inventoryPath)
        }
      ));
    }
    if (previousKey !== null && compareStrings(previousKey, entry.entryKey) >= 0) {
      diagnostics.push(diagnostic(
        "inventory.entries-unsorted",
        "inventory",
        "inventory entries must be uniquely sorted by entryKey",
        {
          blocking: true,
          entryKey: entry.entryKey,
          path: relativeDisplayPath(inventoryPath)
        }
      ));
    }
    seen.add(entry.entryKey);
    previousKey = entry.entryKey;
    entries.push(entry);
  }

  if (!profile || !FINGERPRINT_PATTERN.test(String(raw.sourceRevision))) {
    return null;
  }
  return {
    schemaVersion: SCHEMA_VERSION,
    profile,
    sourceRevision: raw.sourceRevision,
    entries
  };
}

function normalizeProfile(value, sourcePath, diagnostics) {
  if (
    !isRecord(value) ||
    !hasExactKeys(value, ["id", "version"]) ||
    !SLUG_PATTERN.test(String(value.id)) ||
    !Number.isInteger(value.version) ||
    value.version < 1
  ) {
    diagnostics.push(invalidDiagnostic(
      "inventory.profile-invalid",
      "inventory",
      sourcePath,
      "inventory profile must contain a slug id and positive integer version"
    ));
    return null;
  }
  return {
    id: value.id,
    version: value.version
  };
}

function normalizeEntry(value, sourcePath, index, diagnostics) {
  const keys = [
    "entryKey",
    "runner",
    "target",
    "selector",
    "sourcePath",
    "sourceRange",
    "sourceFingerprint"
  ];
  if (!isRecord(value) || !hasExactKeys(value, keys)) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entry-invalid",
      "inventory",
      sourcePath,
      `inventory entry ${index} has an invalid object shape`
    ));
    return null;
  }
  const textFields = ["entryKey", "target", "selector"];
  if (textFields.some((key) => !isNonEmptyTrimmedString(value[key]))) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entry-invalid",
      "inventory",
      sourcePath,
      `inventory entry ${index} has an empty or untrimmed identity field`
    ));
    return null;
  }
  if (!SLUG_PATTERN.test(String(value.runner))) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entry-invalid",
      "inventory",
      sourcePath,
      `inventory entry ${index} runner must be a slug`
    ));
    return null;
  }
  if (!isRelativePosixPath(value.sourcePath)) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entry-invalid",
      "inventory",
      sourcePath,
      `inventory entry ${index} sourcePath must be a safe relative POSIX path`
    ));
    return null;
  }
  const sourceRange = normalizeSourceRange(value.sourceRange);
  if (!sourceRange) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entry-invalid",
      "inventory",
      sourcePath,
      `inventory entry ${index} sourceRange is invalid`
    ));
    return null;
  }
  if (!FINGERPRINT_PATTERN.test(String(value.sourceFingerprint))) {
    diagnostics.push(invalidDiagnostic(
      "inventory.entry-invalid",
      "inventory",
      sourcePath,
      `inventory entry ${index} sourceFingerprint is invalid`
    ));
    return null;
  }
  return {
    entryKey: value.entryKey,
    runner: value.runner,
    target: value.target,
    selector: value.selector,
    sourcePath: value.sourcePath,
    sourceRange,
    sourceFingerprint: value.sourceFingerprint
  };
}

function normalizeSourceRange(value) {
  const keys = ["startLine", "startColumn", "endLine", "endColumn"];
  if (!isRecord(value) || !hasExactKeys(value, keys)) {
    return null;
  }
  if (keys.some((key) => !Number.isInteger(value[key]) || value[key] < 1)) {
    return null;
  }
  if (
    value.endLine < value.startLine ||
    (value.endLine === value.startLine && value.endColumn <= value.startColumn)
  ) {
    return null;
  }
  return {
    startLine: value.startLine,
    startColumn: value.startColumn,
    endLine: value.endLine,
    endColumn: value.endColumn
  };
}

function loadTopicCatalog(topicCatalogPath, diagnostics) {
  const raw = readJson(topicCatalogPath, "claim", diagnostics);
  if (!raw) {
    return null;
  }
  if (
    !isRecord(raw) ||
    !hasExactKeys(raw, ["schemaVersion", "topics"]) ||
    raw.schemaVersion !== SCHEMA_VERSION ||
    !Array.isArray(raw.topics)
  ) {
    diagnostics.push(invalidDiagnostic(
      "claim.topic-catalog-invalid",
      "claim",
      topicCatalogPath,
      "claim topic catalog must have schemaVersion 1 and a topics array"
    ));
    return null;
  }

  const topics = [];
  const seen = new Set();
  let previousId = null;
  for (const [index, value] of raw.topics.entries()) {
    if (
      !isRecord(value) ||
      !hasExactKeys(value, ["id", "description"]) ||
      !SLUG_PATTERN.test(String(value.id)) ||
      !isNonEmptyTrimmedString(value.description) ||
      value.description.includes("\n") ||
      [...value.description].length < 4 ||
      [...value.description].length > 200
    ) {
      diagnostics.push(invalidDiagnostic(
        "claim.topic-invalid",
        "claim",
        topicCatalogPath,
        `claim topic ${index} is invalid`
      ));
      continue;
    }
    if (seen.has(value.id)) {
      diagnostics.push(diagnostic(
        "claim.topic-duplicate",
        "claim",
        `duplicate claim topic ${value.id}`,
        {
          blocking: true,
          path: relativeDisplayPath(topicCatalogPath)
        }
      ));
    }
    if (previousId !== null && compareStrings(previousId, value.id) >= 0) {
      diagnostics.push(diagnostic(
        "claim.topics-unsorted",
        "claim",
        "claim topics must be uniquely sorted by id",
        {
          blocking: true,
          path: relativeDisplayPath(topicCatalogPath)
        }
      ));
    }
    seen.add(value.id);
    previousId = value.id;
    topics.push({
      id: value.id,
      description: value.description
    });
  }
  return {
    schemaVersion: SCHEMA_VERSION,
    topics
  };
}

function loadClaims(paths, inventory, topicCatalog, diagnostics) {
  if (!fs.existsSync(paths.claimsRoot)) {
    return [];
  }
  const rootStats = fs.lstatSync(paths.claimsRoot);
  if (!rootStats.isDirectory() || rootStats.isSymbolicLink()) {
    diagnostics.push(invalidDiagnostic(
      "claim.layout-invalid",
      "claim",
      paths.claimsRoot,
      "claims must be a real directory"
    ));
    return [];
  }

  const topicIds = new Set(topicCatalog.topics.map(({ id }) => id));
  const entryKeys = new Set(inventory.entries.map(({ entryKey }) => entryKey));
  const claims = [];
  const claimIds = new Set();
  for (const topicEntry of fs.readdirSync(paths.claimsRoot, { withFileTypes: true })) {
    const topicPath = path.join(paths.claimsRoot, topicEntry.name);
    if (
      !topicEntry.isDirectory() ||
      topicEntry.isSymbolicLink() ||
      !topicIds.has(topicEntry.name)
    ) {
      diagnostics.push(invalidDiagnostic(
        "claim.layout-invalid",
        "claim",
        topicPath,
        `claims member ${topicEntry.name} is not a controlled topic directory`
      ));
      continue;
    }
    for (const claimEntry of fs.readdirSync(topicPath, { withFileTypes: true })) {
      const claimPath = path.join(topicPath, claimEntry.name);
      if (
        !claimEntry.isFile() ||
        claimEntry.isSymbolicLink() ||
        !CLAIM_FILE_PATTERN.test(claimEntry.name)
      ) {
        diagnostics.push(invalidDiagnostic(
          "claim.layout-invalid",
          "claim",
          claimPath,
          `claim member ${claimEntry.name} must be a direct slug-named Markdown file`
        ));
        continue;
      }
      const claim = parseClaim(
        paths,
        topicEntry.name,
        claimPath,
        entryKeys,
        diagnostics
      );
      if (!claim) {
        continue;
      }
      if (claimIds.has(claim.id)) {
        diagnostics.push(diagnostic(
          "claim.id-duplicate",
          "claim",
          `duplicate Claim id ${claim.id}`,
          {
            blocking: true,
            claimId: claim.id,
            path: claim.sourcePath
          }
        ));
      }
      claimIds.add(claim.id);
      claims.push(claim);
    }
  }
  return claims.sort((left, right) => compareStrings(left.id, right.id));
}

function parseClaim(paths, directoryTopic, claimPath, entryKeys, diagnostics) {
  const source = normalizeMarkdown(fs.readFileSync(claimPath, "utf8"));
  const pattern = /^# Claim ([^:\n]+): ([^\n]+)\n\nTopic: `([^`\n]+)`\nOwner ref: `([^`\n]+)`\n\nStatement:\n((?:- [^\n]+\n)+)\nObservations:\n((?:- [^\n]+\n)+)\nSupported by:\n((?:- `[^`\n]+`\n)+)$/;
  const match = pattern.exec(source);
  const sourcePath = toSlash(path.relative(paths.evidenceRoot, claimPath));
  if (!match) {
    diagnostics.push(diagnostic(
      "claim.layout-invalid",
      "claim",
      "Claim must use the exact v8 Markdown field order and list layout",
      { blocking: true, path: sourcePath }
    ));
    return null;
  }

  const [, id, title, topic, ownerRef, statementBlock, observationBlock, supportBlock] = match;
  const statement = parseTextList(statementBlock);
  const observations = parseTextList(observationBlock);
  const supportedBy = supportBlock
    .trimEnd()
    .split("\n")
    .map((line) => line.slice(3, -1));
  let valid = true;

  if (!CLAIM_ID_PATTERN.test(id) || !isNonEmptyTrimmedString(title)) {
    diagnostics.push(diagnostic(
      "claim.identity-invalid",
      "claim",
      `invalid Claim identity ${id}`,
      { blocking: true, claimId: id, path: sourcePath }
    ));
    valid = false;
  }
  if (topic !== directoryTopic) {
    diagnostics.push(diagnostic(
      "claim.topic-mismatch",
      "claim",
      `Claim topic ${topic} does not match directory ${directoryTopic}`,
      { blocking: true, claimId: id, path: sourcePath }
    ));
    valid = false;
  }
  if (!uniqueNonEmptyList(statement) || !uniqueNonEmptyList(observations)) {
    diagnostics.push(diagnostic(
      "claim.content-invalid",
      "claim",
      "Statement and Observations must contain unique non-empty list items",
      { blocking: true, claimId: id, path: sourcePath }
    ));
    valid = false;
  }
  const templateValue = [...statement, ...observations]
    .find((value) => TEMPLATE_PATTERNS.some((patternValue) => patternValue.test(value)));
  if (templateValue) {
    diagnostics.push(diagnostic(
      "claim.template-repetition",
      "claim",
      `Claim repeats a no-information template: ${templateValue}`,
      { blocking: true, claimId: id, path: sourcePath }
    ));
    valid = false;
  }
  if (!uniqueNonEmptyList(supportedBy)) {
    diagnostics.push(diagnostic(
      "claim.support-empty",
      "claim",
      "Supported by must contain at least one unique current entryKey",
      { blocking: true, claimId: id, path: sourcePath }
    ));
    valid = false;
  }
  for (const entryKey of supportedBy) {
    if (!entryKeys.has(entryKey)) {
      diagnostics.push(diagnostic(
        "claim.entry-unknown",
        "claim",
        `Claim references unknown entryKey ${entryKey}`,
        {
          blocking: true,
          claimId: id,
          entryKey,
          path: sourcePath
        }
      ));
      valid = false;
    }
  }

  const owner = resolveOwner(paths.workspaceRoot, ownerRef);
  if (!owner) {
    diagnostics.push(diagnostic(
      "claim.owner-unknown",
      "claim",
      `Claim ownerRef does not resolve to a Markdown heading: ${ownerRef}`,
      { blocking: true, claimId: id, path: sourcePath }
    ));
    valid = false;
  }
  if (!valid || !owner) {
    return null;
  }
  return {
    id,
    title,
    topic,
    ownerRef,
    statement,
    observations,
    supportedBy,
    sourcePath,
    sourceFingerprint: sha256(source),
    ownerFingerprint: sha256(owner.section)
  };
}

function resolveOwner(workspaceRoot, ownerRef) {
  if (typeof ownerRef !== "string") {
    return null;
  }
  const separatorIndex = ownerRef.lastIndexOf("#");
  if (separatorIndex <= 0 || separatorIndex === ownerRef.length - 1) {
    return null;
  }
  const relativePath = ownerRef.slice(0, separatorIndex);
  const fragment = ownerRef.slice(separatorIndex + 1);
  if (!relativePath.endsWith(".md") || !isRelativePosixPath(relativePath)) {
    return null;
  }
  const absolutePath = path.resolve(workspaceRoot, ...relativePath.split("/"));
  if (!isWithin(workspaceRoot, absolutePath) || !fs.existsSync(absolutePath)) {
    return null;
  }
  const stats = fs.lstatSync(absolutePath);
  if (!stats.isFile() || stats.isSymbolicLink()) {
    return null;
  }
  const lines = normalizeMarkdown(fs.readFileSync(absolutePath, "utf8"))
    .trimEnd()
    .split("\n");
  let inFence = false;
  const slugCounts = new Map();
  const headings = [];
  for (const [lineIndex, line] of lines.entries()) {
    if (/^\s*(?:```|~~~)/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) {
      continue;
    }
    const match = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line);
    if (!match) {
      continue;
    }
    const baseSlug = headingSlug(match[2]);
    const duplicateIndex = slugCounts.get(baseSlug) ?? 0;
    slugCounts.set(baseSlug, duplicateIndex + 1);
    headings.push({
      fragment: duplicateIndex === 0 ? baseSlug : `${baseSlug}-${duplicateIndex}`,
      level: match[1].length,
      lineIndex
    });
  }
  const headingIndex = headings.findIndex(({ fragment: value }) => value === fragment);
  if (headingIndex < 0) {
    return null;
  }
  const heading = headings[headingIndex];
  const next = headings
    .slice(headingIndex + 1)
    .find(({ level }) => level <= heading.level);
  const endLine = next?.lineIndex ?? lines.length;
  return {
    section: `${lines.slice(heading.lineIndex, endLine).join("\n").trimEnd()}\n`
  };
}

function createProjection(inventory, topicCatalog, claims) {
  const claimIdsByEntry = new Map(
    inventory.entries.map(({ entryKey }) => [entryKey, []])
  );
  for (const claim of claims) {
    for (const entryKey of claim.supportedBy) {
      claimIdsByEntry.get(entryKey)?.push(claim.id);
    }
  }
  const entries = inventory.entries.map((entry) => ({
    ...entry,
    claimIds: [...(claimIdsByEntry.get(entry.entryKey) ?? [])].sort(compareStrings)
  }));
  const projectionSource = {
    inventory: {
      profile: inventory.profile,
      sourceRevision: inventory.sourceRevision,
      entries: inventory.entries
    },
    topics: topicCatalog.topics,
    claims
  };
  return {
    schemaVersion: SCHEMA_VERSION,
    sourceRevision: sha256(canonicalJson(projectionSource)),
    inventoryRevision: inventory.sourceRevision,
    topics: topicCatalog.topics,
    entries,
    claims
  };
}

function validatePersistedIndex(indexPath, projection) {
  if (!fs.existsSync(indexPath)) {
    return [
      diagnostic(
        "index.missing",
        "index",
        "test evidence index is missing; run sync-index --write",
        { blocking: true, path: relativeDisplayPath(indexPath) }
      )
    ];
  }
  let persisted;
  try {
    persisted = JSON.parse(fs.readFileSync(indexPath, "utf8"));
  } catch (error) {
    return [
      diagnostic(
        "index.invalid",
        "index",
        `test evidence index is invalid JSON: ${errorMessage(error)}`,
        { blocking: true, path: relativeDisplayPath(indexPath) }
      )
    ];
  }
  if (
    !isRecord(persisted) ||
    persisted.schemaVersion !== SCHEMA_VERSION ||
    !Array.isArray(persisted.entries) ||
    !Array.isArray(persisted.claims)
  ) {
    return [
      diagnostic(
        "index.invalid",
        "index",
        "test evidence index has an invalid v8 shape",
        { blocking: true, path: relativeDisplayPath(indexPath) }
      )
    ];
  }
  if (canonicalJson(persisted) === canonicalJson(projection)) {
    return [];
  }

  const diagnostics = [];
  const currentClaims = new Map(projection.claims.map((claim) => [claim.id, claim]));
  for (const persistedClaim of persisted.claims) {
    if (!isRecord(persistedClaim) || typeof persistedClaim.id !== "string") {
      continue;
    }
    const current = currentClaims.get(persistedClaim.id);
    if (
      current &&
      (
        persistedClaim.ownerFingerprint !== current.ownerFingerprint ||
        canonicalJson(persistedClaim.supportedBy) !== canonicalJson(current.supportedBy)
      )
    ) {
      diagnostics.push(diagnostic(
        "claim.stale",
        "claim",
        `Claim ${current.id} owner content or supported Entry set changed`,
        {
          blocking: true,
          claimId: current.id,
          path: current.sourcePath
        }
      ));
    }
  }
  diagnostics.push(diagnostic(
    "index.stale",
    "index",
    "test evidence index is stale; run sync-index --write after reviewing changes",
    { blocking: true, path: relativeDisplayPath(indexPath) }
  ));
  return diagnostics;
}

function loadQueryProjection(workspaceRoot) {
  const built = buildTestEvidenceProjection({ workspaceRoot });
  if (!built.projection) {
    return {
      projection: null,
      source: "memory",
      diagnostics: built.diagnostics
    };
  }
  const indexPath = evidencePaths(workspaceRoot).indexPath;
  const indexDiagnostics = validatePersistedIndex(indexPath, built.projection);
  if (indexDiagnostics.length === 0) {
    return {
      projection: JSON.parse(fs.readFileSync(indexPath, "utf8")),
      source: "index",
      diagnostics: built.diagnostics
    };
  }
  return {
    projection: built.projection,
    source: "memory",
    diagnostics: [
      ...built.diagnostics,
      ...indexDiagnostics.map((value) => ({
        ...value,
        severity: "warning",
        blocking: false
      }))
    ]
  };
}

function createQueryItems(projection) {
  return [
    ...projection.entries.map((entry) => ({
      kind: "entry",
      id: entry.entryKey,
      entry,
      claimIds: entry.claimIds
    })),
    ...projection.claims.map((claim) => ({
      kind: "claim",
      id: claim.id,
      claim,
      entryKeys: claim.supportedBy
    }))
  ];
}

function matchesKind(item, kind = "all") {
  return kind === "all" || kind === undefined || item.kind === kind;
}

function matchesExactFilters(item, options, projection) {
  const linkedClaims = item.kind === "claim"
    ? [item.claim]
    : projection.claims.filter(({ id }) => item.claimIds.includes(id));
  const linkedEntries = item.kind === "entry"
    ? [item.entry]
    : projection.entries.filter(({ entryKey }) => item.entryKeys.includes(entryKey));

  return (
    exactOrUnset(options.entryKey, linkedEntries.some(({ entryKey }) => entryKey === options.entryKey)) &&
    exactOrUnset(options.runner, linkedEntries.some(({ runner }) => runner === options.runner)) &&
    exactOrUnset(options.target, linkedEntries.some(({ target }) => target === options.target)) &&
    exactOrUnset(options.sourcePath, linkedEntries.some(({ sourcePath }) => sourcePath === options.sourcePath)) &&
    exactOrUnset(options.claimId, linkedClaims.some(({ id }) => id === options.claimId)) &&
    exactOrUnset(options.topic, linkedClaims.some(({ topic }) => topic === options.topic)) &&
    exactOrUnset(options.ownerRef, linkedClaims.some(({ ownerRef }) => ownerRef === options.ownerRef))
  );
}

function exactOrUnset(optionValue, matches) {
  return optionValue === undefined || matches;
}

function matchesText(item, query) {
  if (query === undefined || query.trim() === "") {
    return true;
  }
  const words = query.toLocaleLowerCase().trim().split(/\s+/u);
  const haystack = canonicalJson(item).toLocaleLowerCase();
  return words.every((word) => haystack.includes(word));
}

function compareQueryItems(left, right) {
  const kindOrder = compareStrings(left.kind, right.kind);
  return kindOrder === 0 ? compareStrings(left.id, right.id) : kindOrder;
}

function reportFor(projection, diagnostics) {
  return {
    schemaVersion: SCHEMA_VERSION,
    status: diagnostics.some(({ blocking }) => blocking) ? "error" : "ok",
    diagnostics,
    summary: {
      topics: projection?.topics.length ?? 0,
      entries: projection?.entries.length ?? 0,
      claims: projection?.claims.length ?? 0
    }
  };
}

function syncResult(mode, projection, diagnostics) {
  return {
    schemaVersion: SCHEMA_VERSION,
    status: diagnostics.some(({ blocking }) => blocking) ? "error" : "ok",
    mode,
    diagnostics,
    sourceRevision: projection?.sourceRevision ?? null,
    summary: {
      topics: projection?.topics.length ?? 0,
      entries: projection?.entries.length ?? 0,
      claims: projection?.claims.length ?? 0
    }
  };
}

function readJson(sourcePath, origin, diagnostics) {
  if (!fs.existsSync(sourcePath)) {
    diagnostics.push(diagnostic(
      `${origin}.missing`,
      origin,
      `${path.basename(sourcePath)} is missing`,
      { blocking: true, path: relativeDisplayPath(sourcePath) }
    ));
    return null;
  }
  const stats = fs.lstatSync(sourcePath);
  if (!stats.isFile() || stats.isSymbolicLink()) {
    diagnostics.push(invalidDiagnostic(
      `${origin}.invalid`,
      origin,
      sourcePath,
      `${path.basename(sourcePath)} must be a regular file`
    ));
    return null;
  }
  try {
    return JSON.parse(fs.readFileSync(sourcePath, "utf8"));
  } catch (error) {
    diagnostics.push(invalidDiagnostic(
      `${origin}.json-invalid`,
      origin,
      sourcePath,
      `${path.basename(sourcePath)} is invalid JSON: ${errorMessage(error)}`
    ));
    return null;
  }
}

function assertDistinctIndexIdentity(indexPath, workspaceRoot) {
  if (!fs.existsSync(indexPath)) {
    return;
  }
  const indexStats = fs.lstatSync(indexPath);
  if (!indexStats.isFile() || indexStats.isSymbolicLink()) {
    throw new Error("test evidence index must be a regular non-symbolic file");
  }
  const paths = evidencePaths(workspaceRoot);
  const sourcePaths = [paths.inventoryPath, paths.topicCatalogPath];
  if (fs.existsSync(paths.claimsRoot)) {
    sourcePaths.push(...walkFiles(paths.claimsRoot));
  }
  for (const sourcePath of sourcePaths) {
    const sourceStats = fs.statSync(sourcePath);
    if (sourceStats.dev === indexStats.dev && sourceStats.ino === indexStats.ino) {
      throw new Error(`test evidence index aliases source file ${sourcePath}`);
    }
  }
}

function writeJsonAtomic(targetPath, value) {
  fs.mkdirSync(path.dirname(targetPath), { recursive: true });
  const temporaryPath = path.join(
    path.dirname(targetPath),
    `.${path.basename(targetPath)}.${process.pid}.tmp`
  );
  try {
    fs.writeFileSync(temporaryPath, `${JSON.stringify(value, null, 2)}\n`, {
      flag: "wx"
    });
    fs.renameSync(temporaryPath, targetPath);
  } finally {
    fs.rmSync(temporaryPath, { force: true });
  }
}

function walkFiles(root) {
  const files = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...walkFiles(entryPath));
    } else if (entry.isFile()) {
      files.push(entryPath);
    }
  }
  return files;
}

function parseCli(argv) {
  const [command, ...tokens] = argv;
  if (!["topics", "check", "sync-index", "list", "show"].includes(command)) {
    throw new Error("usage: test-evidence-catalog <topics|check|sync-index|list|show>");
  }
  const values = {
    command,
    json: false,
    write: false,
    root: null,
    positionals: []
  };
  const valueFlags = new Set([
    "--root",
    "--kind",
    "--entry-key",
    "--runner",
    "--target",
    "--source-path",
    "--claim-id",
    "--topic",
    "--owner-ref",
    "--query",
    "--limit",
    "--offset"
  ]);
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token === "--json") {
      values.json = true;
    } else if (token === "--write") {
      values.write = true;
    } else if (valueFlags.has(token)) {
      const value = tokens[index + 1];
      if (value === undefined || value.startsWith("--")) {
        throw new Error(`${token} requires a value`);
      }
      const key = token.slice(2).replace(/-([a-z])/g, (_, letter) => letter.toUpperCase());
      if (values[key] !== undefined && values[key] !== null) {
        throw new Error(`${token} may be provided only once`);
      }
      values[key] = value;
      index += 1;
    } else if (token.startsWith("--")) {
      throw new Error(`unknown option ${token}`);
    } else {
      values.positionals.push(token);
    }
  }
  if (!values.root) {
    throw new Error("--root is required");
  }
  if (command === "show" && values.positionals.length !== 1) {
    throw new Error("show requires exactly one Entry or Claim id");
  }
  if (command !== "show" && values.positionals.length !== 0) {
    throw new Error(`${command} does not accept positional arguments`);
  }
  if (command !== "sync-index" && values.write) {
    throw new Error("--write is only valid with sync-index");
  }
  if (values.kind !== undefined && !["entry", "claim", "all"].includes(values.kind)) {
    throw new Error("--kind must be entry, claim or all");
  }
  values.limit = parseBoundedInteger(values.limit, "--limit", 1, MAX_LIMIT);
  values.offset = parseBoundedInteger(values.offset, "--offset", 0, Number.MAX_SAFE_INTEGER);
  return values;
}

function runCliCommand(options) {
  switch (options.command) {
    case "topics":
      return listTestEvidenceTopics({ workspaceRoot: options.root });
    case "check":
      return validateTestEvidence({ workspaceRoot: options.root });
    case "sync-index":
      return syncTestEvidenceIndex({
        workspaceRoot: options.root,
        mode: options.write ? "write" : "check"
      });
    case "list":
      return queryTestEvidence({
        workspaceRoot: options.root,
        kind: options.kind,
        entryKey: options.entryKey,
        runner: options.runner,
        target: options.target,
        sourcePath: options.sourcePath,
        claimId: options.claimId,
        topic: options.topic,
        ownerRef: options.ownerRef,
        query: options.query,
        limit: options.limit,
        offset: options.offset
      });
    case "show":
      return showTestEvidence({
        workspaceRoot: options.root,
        id: options.positionals[0]
      });
    default:
      throw new Error(`unsupported command ${options.command}`);
  }
}

function formatHumanResult(command, result) {
  if (command === "check" && result.status === "ok") {
    return `Test evidence check passed: ${result.summary.topics} topic(s), ${result.summary.entries} native entry/entries, ${result.summary.claims} claim(s).\n`;
  }
  if (command === "sync-index" && result.status === "ok") {
    return `Test evidence index ${result.mode === "write" ? "written" : "is current"}: ${result.summary.entries} entry/entries, ${result.summary.claims} claim(s).\n`;
  }
  if (command === "topics" && result.status === "ok") {
    return `${result.topics.map(({ id, description }) => `${id}\t${description}`).join("\n")}${result.topics.length > 0 ? "\n" : ""}`;
  }
  if (command === "list" && result.status === "ok") {
    const warningText = result.diagnostics
      .map(({ code, message }) => `warning ${code}: ${message}`)
      .join("\n");
    const itemText = result.items
      .map(({ kind, id }) => `${kind}\t${id}`)
      .join("\n");
    return `${warningText}${warningText && itemText ? "\n" : ""}${itemText}${warningText || itemText ? "\n" : ""}`;
  }
  if (command === "show" && result.status === "ok") {
    const warningText = result.diagnostics
      .map(({ code, message }) => `warning ${code}: ${message}`)
      .join("\n");
    return `${warningText}${warningText ? "\n" : ""}${JSON.stringify(result.item, null, 2)}\n`;
  }
  return `${result.diagnostics
    .map(({ code, message }) => `${code}: ${message}`)
    .join("\n")}\n`;
}

function parseBoundedInteger(value, flag, minimum, maximum) {
  if (value === undefined) {
    return undefined;
  }
  if (!/^\d+$/.test(value)) {
    throw new Error(`${flag} must be an integer`);
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < minimum || parsed > maximum) {
    throw new Error(`${flag} must be between ${minimum} and ${maximum}`);
  }
  return parsed;
}

function normalizeLimit(value) {
  return value === undefined ? DEFAULT_LIMIT : value;
}

function normalizeOffset(value) {
  return value === undefined ? 0 : value;
}

function parseTextList(block) {
  return block
    .trimEnd()
    .split("\n")
    .map((line) => line.slice(2));
}

function uniqueNonEmptyList(values) {
  return (
    values.length > 0 &&
    values.every(isNonEmptyTrimmedString) &&
    new Set(values).size === values.length
  );
}

function hasExactKeys(value, expectedKeys) {
  const actual = Object.keys(value).sort(compareStrings);
  const expected = [...expectedKeys].sort(compareStrings);
  return canonicalJson(actual) === canonicalJson(expected);
}

function isRecord(value) {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isNonEmptyTrimmedString(value) {
  return typeof value === "string" && value.length > 0 && value === value.trim();
}

function isRelativePosixPath(value) {
  return (
    typeof value === "string" &&
    value.length > 0 &&
    value === toSlash(value) &&
    !value.startsWith("/") &&
    !value.includes("\\") &&
    !value.split("/").includes("..") &&
    path.posix.normalize(value) === value
  );
}

function isWithin(root, candidate) {
  const relative = path.relative(path.resolve(root), path.resolve(candidate));
  return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}

function headingSlug(value) {
  return value
    .normalize("NFKD")
    .toLocaleLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .trim()
    .replace(/\s+/gu, "-");
}

function normalizeMarkdown(value) {
  return `${value.replace(/\r\n?/g, "\n").trimEnd()}\n`;
}

function canonicalJson(value) {
  return JSON.stringify(canonicalize(value));
}

function canonicalize(value) {
  if (Array.isArray(value)) {
    return value.map(canonicalize);
  }
  if (isRecord(value)) {
    return Object.fromEntries(
      Object.keys(value)
        .sort(compareStrings)
        .map((key) => [key, canonicalize(value[key])])
    );
  }
  return value;
}

function sha256(value) {
  return `sha256:${crypto.createHash("sha256").update(value).digest("hex")}`;
}

function compareStrings(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

function toSlash(value) {
  return value.split(path.sep).join("/");
}

function relativeDisplayPath(value) {
  return toSlash(path.relative(process.cwd(), value));
}

function diagnostic(code, origin, message, details = {}) {
  return {
    code,
    origin,
    severity: details.severity ?? "error",
    blocking: details.blocking ?? true,
    message,
    ...(details.path === undefined ? {} : { path: details.path }),
    ...(details.entryKey === undefined ? {} : { entryKey: details.entryKey }),
    ...(details.claimId === undefined ? {} : { claimId: details.claimId })
  };
}

function invalidDiagnostic(code, origin, sourcePath, message) {
  return diagnostic(code, origin, message, {
    blocking: true,
    path: relativeDisplayPath(sourcePath)
  });
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

const isDirectExecution = (
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
);
if (isDirectExecution) {
  process.exitCode = await runTestEvidenceCatalogCli();
}
