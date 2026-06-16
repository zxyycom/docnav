/**
 * Tool availability checks for quality scanners.
 */

import { DEFAULT_CONFIG } from "../config.mjs";
import { getLizardVersion } from "./lizard.mjs";
import { getSccVersion } from "./scc.mjs";
import { getCpdVersion } from "./cpd.mjs";

export function checkTools(rootDir) {
  return [
    checkLizard(rootDir),
    checkScc(rootDir),
    checkPmdCpd(rootDir)
  ];
}

function checkLizard(rootDir) {
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
    return { name: "lizard", available: false, version: null, error: "unknown error", source: "uv" };
  }
}

function checkScc(rootDir) {
  try {
    const ver = getSccVersion({ cwd: rootDir, toolConfig: DEFAULT_CONFIG.tools.scc });
    return {
      name: "scc",
      available: ver.ok,
      version: ver.ok ? ver.version : null,
      error: ver.ok ? null : ver.error,
      source: "system",
      reason: ver.ok ? null : ver.reason,
      fatal: !ver.ok && ver.reason !== "tool-unavailable"
    };
  } catch {
    return {
      name: "scc",
      available: false,
      version: null,
      error: "unknown error",
      source: "system",
      reason: "execution-error",
      fatal: true
    };
  }
}

function checkPmdCpd(rootDir) {
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
    return { name: "pmd-cpd", available: false, version: null, error: "unknown error", source: "system" };
  }
}
