export type TestEvidenceDiagnosticOrigin =
  | "profile"
  | "static"
  | "runner"
  | "case"
  | "query";

export type TestEvidenceDiagnostic = {
  code: string;
  origin: TestEvidenceDiagnosticOrigin;
  severity: "error" | "warning";
  blocking: boolean;
  message: string;
  path?: string;
  line?: number;
  column?: number;
  runner?: string;
  target?: string;
  selector?: string;
  entityKey?: string;
  caseId?: string;
};

export type SourceRange = {
  startLine: number;
  startColumn: number;
  endLine: number;
  endColumn: number;
};

export type TestEntity = {
  entityKey: string;
  runner: string;
  target: string;
  selector: string;
  sourcePath: string;
  sourceRange: SourceRange;
};

export type StaticTestEntity = {
  identity: string;
  sourcePath: string;
  sourceRange: SourceRange;
};

export type RuntimeTestEntity = {
  identity: string;
  target: string;
  selector: string;
};

export type DiscoveryResult = {
  profile: {
    id: string;
    version: number;
  };
  entities: TestEntity[];
  diagnostics: TestEvidenceDiagnostic[];
};

export function diagnostic(
  code: string,
  origin: TestEvidenceDiagnosticOrigin,
  message: string,
  details: Partial<Omit<TestEvidenceDiagnostic, "code" | "origin" | "message">> = {}
): TestEvidenceDiagnostic {
  return {
    code,
    origin,
    severity: details.severity ?? "error",
    blocking: details.blocking ?? true,
    message,
    ...(details.path === undefined ? {} : { path: details.path }),
    ...(details.line === undefined ? {} : { line: details.line }),
    ...(details.column === undefined ? {} : { column: details.column }),
    ...(details.runner === undefined ? {} : { runner: details.runner }),
    ...(details.target === undefined ? {} : { target: details.target }),
    ...(details.selector === undefined ? {} : { selector: details.selector }),
    ...(details.entityKey === undefined ? {} : { entityKey: details.entityKey }),
    ...(details.caseId === undefined ? {} : { caseId: details.caseId })
  };
}
