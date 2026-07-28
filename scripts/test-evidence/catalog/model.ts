import type { TestEvidenceDiagnostic } from "../model.ts";

export type TestCaseTopic = {
  id: string;
  description: string;
};

export type SemanticTestCase = {
  id: string;
  title: string;
  topic: string;
  ownerRef: string;
  entityKeys: string[];
  proves: string[];
  sourcePath: string;
  sourceLine: number;
};

export type TestCaseCatalog = {
  schemaVersion: 1;
  topics: TestCaseTopic[];
  cases: SemanticTestCase[];
  diagnostics: TestEvidenceDiagnostic[];
};
