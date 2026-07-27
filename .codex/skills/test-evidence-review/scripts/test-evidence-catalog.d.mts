export type TestEvidenceOrigin =
  | "inventory"
  | "claim"
  | "index"
  | "query";

export type TestEvidenceDiagnostic = {
  code: string;
  origin: TestEvidenceOrigin;
  severity: "error" | "warning";
  blocking: boolean;
  message: string;
  path?: string;
  entryKey?: string;
  claimId?: string;
};

export type SourceRange = {
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
};

export type NativeTestEntry = {
  entryKey: string;
  runner: string;
  target: string;
  selector: string;
  sourcePath: string;
  sourceRange: SourceRange;
  sourceFingerprint: string;
};

export type EvidenceClaim = {
  id: string;
  title: string;
  topic: string;
  ownerRef: string;
  statement: string[];
  observations: string[];
  supportedBy: string[];
  sourcePath: string;
  sourceFingerprint: string;
  ownerFingerprint: string;
};

export type TestEvidenceProjection = {
  schemaVersion: 1;
  sourceRevision: string;
  inventoryRevision: string;
  topics: Array<{
    id: string;
    description: string;
  }>;
  entries: Array<NativeTestEntry & {
    claimIds: string[];
  }>;
  claims: EvidenceClaim[];
};

export type TestEvidenceReport = {
  schemaVersion: 1;
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  summary: {
    topics: number;
    entries: number;
    claims: number;
  };
};

export type BuildTestEvidenceProjectionResult = {
  diagnostics: TestEvidenceDiagnostic[];
  projection: TestEvidenceProjection | null;
};

export type SyncTestEvidenceIndexResult = TestEvidenceReport & {
  mode: "check" | "write";
  sourceRevision: string | null;
};

export type TestEvidenceEntryItem = {
  kind: "entry";
  id: string;
  entry: TestEvidenceProjection["entries"][number];
  claimIds: string[];
};

export type TestEvidenceClaimItem = {
  kind: "claim";
  id: string;
  claim: EvidenceClaim;
  entryKeys: string[];
};

export type TestEvidenceQueryItem =
  | TestEvidenceEntryItem
  | TestEvidenceClaimItem;

export type TestEvidenceQueryResult = {
  schemaVersion: 1;
  status: "ok" | "error";
  source: "index" | "memory";
  diagnostics: TestEvidenceDiagnostic[];
  offset: number;
  limit: number;
  total: number;
  items: TestEvidenceQueryItem[];
};

export type TestEvidenceShowResult = {
  schemaVersion: 1;
  status: "ok" | "error";
  source: "index" | "memory";
  diagnostics: TestEvidenceDiagnostic[];
  item: TestEvidenceQueryItem | null;
};

export declare function buildTestEvidenceProjection(options: {
  workspaceRoot: string;
}): BuildTestEvidenceProjectionResult;

export declare function validateTestEvidence(options: {
  workspaceRoot: string;
}): TestEvidenceReport;

export declare function syncTestEvidenceIndex(options: {
  workspaceRoot: string;
  mode: "check" | "write";
}): SyncTestEvidenceIndexResult;

export declare function queryTestEvidence(options: {
  workspaceRoot: string;
  kind?: "entry" | "claim" | "all";
  entryKey?: string;
  runner?: string;
  target?: string;
  sourcePath?: string;
  claimId?: string;
  topic?: string;
  ownerRef?: string;
  query?: string;
  offset?: number;
  limit?: number;
}): TestEvidenceQueryResult;

export declare function showTestEvidence(options: {
  workspaceRoot: string;
  id: string;
}): TestEvidenceShowResult;

export declare function listTestEvidenceTopics(options: {
  workspaceRoot: string;
}): {
  schemaVersion: 1;
  status: "ok" | "error";
  diagnostics: TestEvidenceDiagnostic[];
  topics: Array<{
    id: string;
    description: string;
  }>;
};

export declare function runTestEvidenceCatalogCli(
  argv?: readonly string[]
): Promise<number>;
