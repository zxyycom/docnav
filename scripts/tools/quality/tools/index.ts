/**
 * Tool availability checks for quality scanners.
 */

import { DEFAULT_CONFIG } from "../config.ts";
import { SCC_VERSION_OUTPUT } from "./scc.ts";
import { parsePmdVersionOutput } from "./cpd.ts";
import { runProcess } from "../../process.ts";
import type { ToolAvailability, ToolConfig } from "../schema.ts";

export async function checkTools(rootDir: string): Promise<ToolAvailability[]> {
  return Promise.all([
    checkLizard(rootDir),
    checkScc(rootDir),
    checkPmdCpd(rootDir)
  ]);
}

async function checkLizard(rootDir: string): Promise<ToolAvailability> {
  try {
    const result = await runToolCommand(rootDir, DEFAULT_CONFIG.tools.lizard, ["--version"]);
    if (result.error) {
      return {
        name: "lizard",
        available: false,
        version: null,
        error: `lizard version error: ${result.error.message}`,
        source: "uv"
      };
    }

    const version = (result.stdout || "").trim() || (result.stderr || "").trim();
    if (!version && result.status !== 0) {
      return {
        name: "lizard",
        available: false,
        version: null,
        error: `lizard --version failed, exit ${result.status}`,
        source: "uv"
      };
    }

    return { name: "lizard", available: true, version: version || "unknown", error: null, source: "uv" };
  } catch {
    return { name: "lizard", available: false, version: null, error: "unknown error", source: "uv" };
  }
}

async function checkScc(rootDir: string): Promise<ToolAvailability> {
  try {
    const result = await runToolCommand(rootDir, DEFAULT_CONFIG.tools.scc, ["--version"]);
    if (result.error) {
      const code = (result.error as NodeJS.ErrnoException).code;
      return {
        name: "scc",
        available: false,
        version: null,
        error: code === "ENOENT" ? `scc not installed: ${result.error.message}` : `scc version error: ${result.error.message}`,
        source: "system",
        reason: code === "ENOENT" ? "tool-unavailable" : "execution-error"
      };
    }

    const version = (result.stdout || "").trim() || (result.stderr || "").trim();
    if (result.status !== 0) {
      const failure = typeof result.status === "number"
        ? `exit ${result.status}`
        : `signal ${result.signal || "unknown"}`;
      return {
        name: "scc",
        available: false,
        version: null,
        error: `scc --version failed, ${failure}${version ? `: ${version}` : ""}`,
        source: "system",
        reason: "execution-error"
      };
    }

    if (version !== SCC_VERSION_OUTPUT) {
      return {
        name: "scc",
        available: false,
        version: null,
        error: `expected ${SCC_VERSION_OUTPUT}, got "${version || "unknown"}"`,
        source: "system",
        reason: "contract-error"
      };
    }

    return { name: "scc", available: true, version, error: null, source: "system", reason: null };
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

async function checkPmdCpd(rootDir: string): Promise<ToolAvailability> {
  try {
    const result = await runProcess({
      args: ["--version"],
      command: DEFAULT_CONFIG.tools.pmdCpd.command,
      cwd: rootDir
    });
    if (result.error) {
      const code = (result.error as NodeJS.ErrnoException).code;
      return {
        name: "pmd-cpd",
        available: false,
        version: null,
        error: code === "ENOENT" ? "PMD not installed" : `PMD version error: ${result.error.message}`,
        source: "system"
      };
    }

    const output = (result.stdout || "").trim() || (result.stderr || "").trim();
    if (result.status !== 0) {
      return {
        name: "pmd-cpd",
        available: false,
        version: null,
        error: `PMD --version failed, exit ${result.status}${output ? `: ${output}` : ""}`,
        source: "system"
      };
    }

    return {
      name: "pmd-cpd",
      available: true,
      version: parsePmdVersionOutput(output),
      error: null,
      source: "system"
    };
  } catch {
    return { name: "pmd-cpd", available: false, version: null, error: "unknown error", source: "system" };
  }
}

function runToolCommand(rootDir: string, toolConfig: ToolConfig, args: string[]) {
  return runProcess({
    args: [...toolConfig.args, ...args],
    command: toolConfig.command,
    cwd: rootDir
  });
}
