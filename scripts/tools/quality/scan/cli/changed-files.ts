import { getChangedFileList, type ChangedFilesOptions } from "../../files.ts";
import type { ChangeScope } from "./types.ts";

export function resolveChangedFilesForScan({
  opts,
  root,
  scope,
  collectChangedFiles = getChangedFileList
}: {
  collectChangedFiles?: (opts: ChangedFilesOptions, rootDir: string) => string[];
  opts: ChangedFilesOptions;
  root: string;
  scope: ChangeScope;
}): string[] {
  if (opts.changedFiles) {
    return collectChangedFiles(opts, root);
  }

  if (scope.changedFiles.length > 0 || !scope.changed) {
    return scope.changedFiles;
  }

  return collectChangedFiles(opts, root);
}
