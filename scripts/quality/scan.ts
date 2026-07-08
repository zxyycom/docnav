#!/usr/bin/env bun

/**
 * Docnav 代码质量观测命令入口。
 *
 * Docnav 拥有默认扫描范围、artifact 路径和 CLI 参数语义；共享 quality core
 * 只接收 typed config/options 并执行扫描内核。
 */

import { resolve, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { DEFAULT_CONFIG } from "./config.ts";
import { errorMessage } from "../tools/foundation/src/errors.ts";
import { parseArgs } from "./args.ts";
import {
  qualityScanErrorExitCode,
  runQualityScan
} from "../tools/quality-core/src/index.ts";
import type { QualityScanOptions } from "../tools/quality-core/src/index.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

async function main(): Promise<void> {
  const opts = parseArgs();
  const status = await runQualityScan({
    banner: printBanner,
    config: DEFAULT_CONFIG,
    options: opts,
    root,
    timingsEnabled: process.env.DOCNAV_QUALITY_TIMINGS === "1"
  });

  process.exit(status === "failed" ? 2 : 0);
}

function printBanner(scanProfile: QualityScanOptions["scanProfile"]): void {
  console.log("Docnav Code Quality Observability");
  console.log(`Profile: ${scanProfile}`);
  if (scanProfile === "quick") {
    console.log("Quick check — skips baseline comparison and jscpd duplicate detection.");
  } else {
    console.log("Full check — runs all configured scanners; baseline comparison is opt-in.");
  }
  console.log("");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((err: unknown) => {
    console.error("Fatal error in quality scan:", errorMessage(err));
    process.exit(qualityScanErrorExitCode(err));
  });
}
