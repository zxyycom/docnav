/**
 * Tool availability checks for quality scanners.
 */

import { DEFAULT_CONFIG } from "../config.ts";
import { getLizardVersion } from "./lizard.ts";
import { getSccVersion } from "./scc.ts";
import { getCpdVersion } from "./cpd.ts";

export function checkTools(rootDir: ExternalValue) {
  return [
    checkLizard(rootDir),
    checkScc(rootDir),
    checkPmdCpd(rootDir)
  ];
}

function checkLizard(rootDir: ExternalValue) {
  try {
    const ver = getLizardVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.lizard });
    return {
      name: "lizard",
      available: ver.ok,
      version: ver.ok ? ver.version : null,
      error: ver.ok ? null : ver.error,
      source: "uv"
    };
  } catch {
    return { name: "lizard", available: false, version: null, error: "ExternalValue error", source: "uv" };
  }
}

function checkScc(rootDir: ExternalValue) {
  try {
    const ver = getSccVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.scc });
    return {
      name: "scc",
      available: ver.ok,
      version: ver.ok ? ver.version : null,
      error: ver.ok ? null : ver.error,
      source: "system",
      reason: ver.ok ? null : ver.reason
    };
  } catch {
    return {
      name: "scc",
      available: false,
      version: null,
      error: "ExternalValue error",
      source: "system",
      reason: "execution-error"
    };
  }
}

function checkPmdCpd(rootDir: ExternalValue) {
  try {
    const ver = getCpdVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.pmdCpd });
    return {
      name: "pmd-cpd",
      available: ver.ok,
      version: ver.ok ? ver.version : null,
      error: ver.ok ? null : ver.error,
      source: "system"
    };
  } catch {
    return { name: "pmd-cpd", available: false, version: null, error: "ExternalValue error", source: "system" };
  }
}
