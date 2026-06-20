export type QualityScanOptions = {
  artifactDir: string;
  baseline: string | null;
  changedFiles: string | null;
  skipBaseline: boolean;
  topN: number;
};

export type ChangeScope = {
  changed: boolean;
  changedFiles: string[];
};
