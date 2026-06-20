/**
 * Tool availability checks for quality scanners.
 */

import { DEFAULT_CONFIG } from "../../model/config.ts";
import { SCC_VERSION_OUTPUT } from "./scc.ts";
import { parsePmdVersionOutput } from "./pmd-cpd/scanner.ts";
import { runProcess } from "../../../process.ts";
import type { ToolAvailability, ToolConfig } from "../../model/schema.ts";

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
    return sccAvailabilityFromVersionResult(result);
  } catch {
    return unavailableScc("unknown error", "execution-error");
  }
}

function sccAvailabilityFromVersionResult(
  result: Awaited<ReturnType<typeof runToolCommand>>
): ToolAvailability {
  if (result.error) {
    return sccProcessErrorAvailability(result.error);
  }

  const version = versionOutput(result);
  if (result.status !== 0) {
    return unavailableScc(
      `scc --version failed, ${processFailure(result)}${version ? `: ${version}` : ""}`,
      "execution-error"
    );
  }

  if (version !== SCC_VERSION_OUTPUT) {
    return unavailableScc(
      `expected ${SCC_VERSION_OUTPUT}, got "${version || "unknown"}"`,
      "contract-error"
    );
  }

  return { name: "scc", available: true, version, error: null, source: "system", reason: null };
}

function sccProcessErrorAvailability(error: Error): ToolAvailability {
  const code = (error as NodeJS.ErrnoException).code;
  const isMissingTool = code === "ENOENT";
  return unavailableScc(
    isMissingTool ? `scc not installed: ${error.message}` : `scc version error: ${error.message}`,
    isMissingTool ? "tool-unavailable" : "execution-error"
  );
}

function unavailableScc(error: string, reason: NonNullable<ToolAvailability["reason"]>): ToolAvailability {
  return {
    name: "scc",
    available: false,
    version: null,
    error,
    source: "system",
    reason
  };
}

function versionOutput(result: Awaited<ReturnType<typeof runToolCommand>>): string {
  return (result.stdout || "").trim() || (result.stderr || "").trim();
}

function processFailure(result: Awaited<ReturnType<typeof runToolCommand>>): string {
  return typeof result.status === "number"
    ? `exit ${result.status}`
    : `signal ${result.signal || "unknown"}`;
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
