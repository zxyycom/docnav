import type {
  CodeAreaFingerprint,
  FatalIssue,
  QualityConfig,
  QualityMetrics,
  ToolAvailability
} from "../../schema.ts";

export type ScanContext = {
  changedFiles: string[];
  config: QualityConfig;
  fatalIssues: FatalIssue[];
  fingerprints: Record<string, CodeAreaFingerprint>;
  metrics: QualityMetrics;
  rawDir: string;
  root: string;
  toolResults: ToolAvailability[];
};
