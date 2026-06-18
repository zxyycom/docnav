/**
 * Tool availability checks for quality scanners.
 */

import { DEFAULT_CONFIG } from "../config.ts";
import { getLizardVersion } from "./lizard.ts";
import { getSccVersion } from "./scc.ts";
import { getCpdVersion } from "./cpd.ts";
import type { ToolAvailability } from "../schema.ts";

export function checkTools(rootDir: string): ToolAvailability[] {
  return [
    checkLizard(rootDir),
    checkScc(rootDir),
    checkPmdCpd(rootDir)
  ];
}

function checkLizard(rootDir: string): ToolAvailability {
  try {
    const ver = getLizardVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.lizard });
    return {
      name: "lizard",
      available: ver.ok,
      version: ver.ok && typeof ver.version === "string" ? ver.version : null,
      error: ver.ok ? null : (ver.error ?? null),
      source: "uv"
    };
  } catch {
    return { name: "lizard", available: false, version: null, error: "unknown error", source: "uv" };
  }
}

function checkScc(rootDir: string): ToolAvailability {
  try {
    const ver = getSccVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.scc });
    return {
      name: "scc",
      available: ver.ok,
      version: ver.ok && typeof ver.version === "string" ? ver.version : null,
      error: ver.ok ? null : (ver.error ?? null),
      source: "system",
      reason: ver.ok ? null : ver.reason
    };
  } catch {
    return {
      name: "scc",
      available: false,
      version: null,
      error: "unknown error",
      source: "system",
      reason: "execution-error"
    };
  }
}

function checkPmdCpd(rootDir: string): ToolAvailability {
  try {
    const ver = getCpdVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.pmdCpd });
    return {
      name: "pmd-cpd",
      available: ver.ok,
      version: ver.ok && typeof ver.version === "string" ? ver.version : null,
      error: ver.ok ? null : (ver.error ?? null),
      source: "system"
    };
  } catch {
    return { name: "pmd-cpd", available: false, version: null, error: "unknown error", source: "system" };
  }
}
