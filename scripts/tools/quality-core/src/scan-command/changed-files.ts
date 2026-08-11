import { getChangedFileList, type ChangedFilesOptions } from "../input/files.ts";
import type { ChangeScope } from "./command-model.ts";
import type { QualityConfig } from "../model/schema.ts";

export type ResolveChangedFilesForScanOptions = {
  collectChangedFiles?: (opts: ChangedFilesOptions, rootDir: string) => string[];
  config?: Pick<QualityConfig, "include">;
  opts: Pick<ChangedFilesOptions, "changedFiles">;
  root: string;
  scope: ChangeScope;
};

export type ResolvedChangedInput = {
  changedFiles: string[];
  inputScope: ChangeScope;
};

export function resolveChangedInputForScan(
  options: ResolveChangedFilesForScanOptions
): ResolvedChangedInput {
  const {
    opts,
    config,
    root,
    scope,
    collectChangedFiles = getChangedFileList
  } = options;
  const changedFileOptions = { ...opts, scanInputPaths: config?.include ?? [] };
  if (opts.changedFiles) {
    const changedFiles = collectChangedFiles(changedFileOptions, root);
    return {
      changedFiles,
      inputScope: scope.status === "unavailable"
        ? { status: "available", changed: changedFiles.length > 0, changedFiles }
        : scope
    };
  }

  if (scope.status === "unavailable") {
    return { changedFiles: [], inputScope: scope };
  }

  const changedFiles = scope.changedFiles.length > 0 || !scope.changed
    ? scope.changedFiles
    : collectChangedFiles(changedFileOptions, root);
  return {
    changedFiles,
    inputScope: scope
  };
}
