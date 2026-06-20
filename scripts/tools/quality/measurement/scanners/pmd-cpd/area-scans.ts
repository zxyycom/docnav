/**
 * PMD CPD scan task planning and cache orchestration.
 */

import {
  loadScanCacheEntry,
  writeScanCacheEntry,
  type CpdCacheIdentity,
  type ScanKind
} from "../../cache.ts";
import { isExcluded } from "../../../model/code-areas.ts";
import { getPmdCpdLanguageForCodeArea, scanWithPmdCpdAsync, type PmdCpdScanResult } from "./scanner.ts";
import { runBoundedTasks } from "./parallel.ts";
import type {
  CodeAreaFileMap,
  CodeAreaFingerprint,
  DuplicateCodeFragment,
  QualityConfig,
  ToolAvailability
} from "../../../model/schema.ts";

export type PmdCpdAreaScanInput = {
  area: string;
  files: string[];
  minimumTokens: number;
};

export type PmdCpdAreaScanTask = {
  area: string;
  codeArea: string;
  files: string[];
  id: string;
  minimumTokens: number;
};

type PmdCpdAreaScanOptions = {
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

type PmdCpdCacheMiss = PmdCpdAreaScanInput & {
  identity: CpdCacheIdentity;
};

type PmdCpdAreaScanResult = {
  result: PmdCpdScanResult;
  task: PmdCpdAreaScanTask;
};

type PmdCpdAreaScanWork = {
  allFragments: DuplicateCodeFragment[];
  misses: PmdCpdCacheMiss[];
};

export async function scanPmdCpdAreasWithCache(options: PmdCpdAreaScanOptions): Promise<DuplicateCodeFragment[]> {
  const work = collectPmdCpdAreaScanWork(options);
  const tasks = planPmdCpdAreaScanTasks(work.misses);
  const taskResults = await runBoundedTasks(
    tasks,
    options.config.pmdCpd.maxParallelTasks,
    async (task) => runPmdCpdAreaScanTask(options, task)
  );

  appendPmdCpdAreaScanResults(options, work, taskResults);
  return work.allFragments;
}

function collectPmdCpdAreaScanWork(options: PmdCpdAreaScanOptions): PmdCpdAreaScanWork {
  const work: PmdCpdAreaScanWork = { allFragments: [], misses: [] };
  for (const [area, areaFiles] of options.fileMap.entries()) {
    const targetFiles = areaFiles.filter(
      (file) => !isExcluded(file, options.config.excludeDirs, options.config.generatedFiles)
    );
    const minTokens = options.config.pmdCpd.minimumTokens[area] ??
      options.config.pmdCpd.defaultMinimumTokens;
    const identity = createPmdCpdCacheIdentity(options, area, minTokens);
    const cached = loadScanCacheEntry({
      rootDir: options.cacheRootDir,
      identity
    });

    if (cached.hit) {
      const fragments = annotatePmdCpdFragments(cached.metrics, area, options.changedFiles);
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

function appendPmdCpdAreaScanResults(
  options: PmdCpdAreaScanOptions,
  work: PmdCpdAreaScanWork,
  taskResults: PmdCpdAreaScanResult[]
): void {
  const missByArea = new Map(work.misses.map((miss) => [miss.area, miss]));
  for (const { task, result } of taskResults) {
    if (result.ok) {
      const miss = missByArea.get(task.area);
      if (!miss) continue;

      const fragments = annotatePmdCpdFragments(result.fragments ?? [], task.area, options.changedFiles);
      work.allFragments.push(...fragments);
      writeScanCacheEntry({
        rootDir: options.cacheRootDir,
        identity: miss.identity,
        metrics: fragments
      });
      console.log(`${options.logPrefix}  CPD ${task.area}: found ${fragments.length} duplicate fragments`);
    } else {
      handlePmdCpdAreaScanFailure(options, task, result);
    }
  }
}

export function planPmdCpdAreaScanTasks(areas: PmdCpdAreaScanInput[]): PmdCpdAreaScanTask[] {
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

function createPmdCpdCacheIdentity(
  options: PmdCpdAreaScanOptions,
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
    normalizedToolArgs: pmdCpdCacheArgs(options.config, codeArea, minTokens),
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

function pmdCpdCacheArgs(config: QualityConfig, codeArea: string, minTokens: number): string[] {
  const language = getPmdCpdLanguageForCodeArea(codeArea);
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

async function runPmdCpdAreaScanTask(options: PmdCpdAreaScanOptions, task: PmdCpdAreaScanTask): Promise<PmdCpdAreaScanResult> {
  console.log(
    `${options.logPrefix}CPD task ${task.id}: ${task.files.length} files, ` +
    `minimum tokens=${task.minimumTokens}`
  );

  const result = await scanWithPmdCpdAsync({
    files: task.files,
    cwd: options.cwd,
    toolConfig: options.config.tools.pmdCpd,
    minimumTokens: task.minimumTokens,
    codeArea: task.codeArea,
    skipIfUnavailable: true
  });

  return { task, result };
}

function handlePmdCpdAreaScanFailure(
  options: PmdCpdAreaScanOptions,
  task: PmdCpdAreaScanTask,
  result: Extract<PmdCpdScanResult, { ok: false }>
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

function annotatePmdCpdFragments(
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
