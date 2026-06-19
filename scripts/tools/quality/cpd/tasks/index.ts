/**
 * PMD CPD scan task planning and cache orchestration.
 */

import {
  loadScanCacheEntry,
  writeScanCacheEntry,
  type CpdCacheIdentity,
  type ScanKind
} from "../../cache.ts";
import { isExcluded } from "../../classify.ts";
import { getCpdLanguageForCodeArea, scanWithCpdAsync, type CpdScanResult } from "../../tools/cpd/index.ts";
import { runBoundedTasks } from "./parallel.ts";
import type {
  CodeAreaFileMap,
  CodeAreaFingerprint,
  DuplicateCodeFragment,
  QualityConfig,
  ToolAvailability
} from "../../schema.ts";

export type CpdAreaScanInput = {
  area: string;
  files: string[];
  minimumTokens: number;
};

export type CpdScanTask = {
  area: string;
  codeArea: string;
  files: string[];
  id: string;
  minimumTokens: number;
};

type CachedCpdScanOptions = {
  cacheRootDir: string;
  changedFiles?: string[];
  commitSha: string;
  config: QualityConfig;
  cwd: string;
  failOnSkipped: boolean;
  fileMap: CodeAreaFileMap;
  fingerprints: Record<string, CodeAreaFingerprint>;
  logPrefix: string;
  scanKind: ScanKind;
  toolResults: ToolAvailability[];
};

type CpdMiss = CpdAreaScanInput & {
  identity: CpdCacheIdentity;
};

type CpdTaskResult = {
  result: CpdScanResult;
  task: CpdScanTask;
};

type CpdScanWork = {
  allFragments: DuplicateCodeFragment[];
  misses: CpdMiss[];
};

export async function scanCpdPartitionsWithCache(options: CachedCpdScanOptions): Promise<DuplicateCodeFragment[]> {
  const work = collectCpdScanWork(options);
  const tasks = planCpdScanTasks(work.misses);
  const taskResults = await runBoundedTasks(
    tasks,
    options.config.pmdCpd.maxParallelTasks,
    async (task) => runCpdTask(options, task)
  );

  appendCpdTaskResults(options, work, taskResults);
  return work.allFragments;
}

function collectCpdScanWork(options: CachedCpdScanOptions): CpdScanWork {
  const work: CpdScanWork = { allFragments: [], misses: [] };
  for (const [area, areaFiles] of options.fileMap.entries()) {
    const targetFiles = areaFiles.filter(
      (file) => !isExcluded(file, options.config.excludeDirs, options.config.generatedFiles)
    );
    const minTokens = options.config.pmdCpd.minimumTokens[area] ??
      options.config.pmdCpd.defaultMinimumTokens;
    const identity = createCpdCacheIdentity(options, area, minTokens);
    const cached = loadScanCacheEntry({
      rootDir: options.cacheRootDir,
      identity
    });

    if (cached.hit) {
      const fragments = annotateCpdFragments(cached.metrics, area, options.changedFiles);
      work.allFragments.push(...fragments);
      console.log(`${options.logPrefix}CPD ${area}: ${fragments.length} duplicate fragments from cache`);
      continue;
    }

    if (targetFiles.length < 2) {
      console.log(`${options.logPrefix}CPD ${area}: too few files (${targetFiles.length}), skipping`);
      continue;
    }

    work.misses.push({
      area,
      files: targetFiles,
      minimumTokens: minTokens,
      identity
    });
  }

  return work;
}

function appendCpdTaskResults(
  options: CachedCpdScanOptions,
  work: CpdScanWork,
  taskResults: CpdTaskResult[]
): void {
  const missByArea = new Map(work.misses.map((miss) => [miss.area, miss]));
  for (const { task, result } of taskResults) {
    if (result.ok) {
      const miss = missByArea.get(task.area);
      if (!miss) continue;

      const fragments = annotateCpdFragments(result.fragments ?? [], task.area, options.changedFiles);
      work.allFragments.push(...fragments);
      writeScanCacheEntry({
        rootDir: options.cacheRootDir,
        identity: miss.identity,
        metrics: fragments
      });
      console.log(`${options.logPrefix}  CPD ${task.area}: found ${fragments.length} duplicate fragments`);
    } else {
      handleCpdTaskFailure(options, task, result);
    }
  }
}

export function planCpdScanTasks(areas: CpdAreaScanInput[]): CpdScanTask[] {
  return areas
    .filter((area) => area.files.length >= 2)
    .map((area) => ({
      area: area.area,
      codeArea: area.area,
      files: uniqueSorted(area.files),
      id: `pmd-cpd:${area.area}`,
      minimumTokens: area.minimumTokens
    }));
}

function createCpdCacheIdentity(
  options: CachedCpdScanOptions,
  codeArea: string,
  minTokens: number
): CpdCacheIdentity {
  const toolVersion = toolVersionFor(options.toolResults, "pmd-cpd");
  if (!toolVersion) {
    throw new Error("missing available tool version for pmd-cpd");
  }

  return {
    scanKind: options.scanKind,
    toolName: "pmd-cpd",
    toolVersion,
    normalizedToolArgs: cpdCacheArgs(options.config, codeArea, minTokens),
    configVersion: options.config.version,
    codeArea,
    commitSha: options.commitSha,
    inputFingerprint: options.fingerprints[codeArea] ?? {
      fileCount: 0,
      fileList: [],
      fingerprint: "empty"
    }
  };
}

function toolVersionFor(toolResults: ToolAvailability[], name: string): string | null {
  return toolResults.find((tool) => tool.name === name && tool.available)?.version ?? null;
}

function cpdCacheArgs(config: QualityConfig, codeArea: string, minTokens: number): string[] {
  const language = getCpdLanguageForCodeArea(codeArea);
  return [
    ...config.tools.pmdCpd.args,
    "--minimum-tokens",
    String(minTokens),
    "--format",
    "xml",
    "--file-list",
    "<input-fingerprint>",
    "--no-fail-on-error",
    ...(language ? ["--language", language] : [])
  ];
}

async function runCpdTask(options: CachedCpdScanOptions, task: CpdScanTask): Promise<CpdTaskResult> {
  console.log(
    `${options.logPrefix}CPD task ${task.id}: ${task.files.length} files, ` +
    `minimum tokens=${task.minimumTokens}`
  );

  const result = await scanWithCpdAsync({
    files: task.files,
    cwd: options.cwd,
    toolConfig: options.config.tools.pmdCpd,
    minimumTokens: task.minimumTokens,
    codeArea: task.codeArea,
    skipIfUnavailable: true
  });

  return { task, result };
}

function handleCpdTaskFailure(
  options: CachedCpdScanOptions,
  task: CpdScanTask,
  result: Extract<CpdScanResult, { ok: false }>
): void {
  if (result.skipped) {
    if (options.failOnSkipped) {
      throw new Error(`baseline CPD scan skipped for task ${task.id}: ${result.error}`);
    }
    console.log(`${options.logPrefix}⚠️  CPD task ${task.id}: ${result.error} (skipped)`);
    return;
  }

  if (options.failOnSkipped) {
    throw new Error(`baseline CPD scan failed for task ${task.id}: ${result.error}`);
  }
  console.log(`${options.logPrefix}⚠️  CPD task ${task.id} error: ${result.error}`);
}

function annotateCpdFragments(
  fragments: DuplicateCodeFragment[],
  area: string,
  changedFiles: string[] | undefined
): DuplicateCodeFragment[] {
  return fragments.map((fragment) => ({
    ...fragment,
    codeAreas: [area],
    hitsChangedScope: changedFiles
      ? fragment.locations.some((location) => isInChangedScope(location.path, changedFiles))
      : false,
    locations: fragment.locations.map((location) => ({
      ...location,
      codeArea: area
    }))
  }));
}

function isInChangedScope(filePath: string, changedFiles: string[]): boolean {
  return changedFiles.some((changedFile) => filePath.includes(changedFile) || changedFile.includes(filePath));
}

function uniqueSorted(files: string[]): string[] {
  return [...new Set(files)].sort((a, b) => a.localeCompare(b));
}
