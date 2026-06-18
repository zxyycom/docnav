/**
 * Tool availability checks for quality scanners.
 */

import { spawn } from "node:child_process";

import { DEFAULT_CONFIG } from "../config.ts";
import { SCC_VERSION_OUTPUT } from "./scc.ts";
import { buildPmdShellCommand, parsePmdVersionOutput } from "./cpd.ts";
import type { ToolAvailability, ToolConfig } from "../schema.ts";

type VersionCommandResult = {
  error?: Error;
  signal: NodeJS.Signals | null;
  status: number | null;
  stderr: string;
  stdout: string;
};

export async function checkTools(rootDir: string): Promise<ToolAvailability[]> {
  return Promise.all([
    checkLizard(rootDir),
    checkScc(rootDir),
    checkPmdCpd(rootDir)
  ]);
}

async function checkLizard(rootDir: string): Promise<ToolAvailability> {
  try {
    const result = await runVersionCommand({
      args: [...DEFAULT_CONFIG.tools.lizard.args, "--version"],
      command: DEFAULT_CONFIG.tools.lizard.command,
      cwd: rootDir
    });
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
    const result = await runVersionCommand({
      args: [...DEFAULT_CONFIG.tools.scc.args, "--version"],
      command: DEFAULT_CONFIG.tools.scc.command,
      cwd: rootDir
    });
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
    const result = await runPmdVersionCommand(rootDir, DEFAULT_CONFIG.tools.pmdCpd);
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

function runPmdVersionCommand(rootDir: string, toolConfig: ToolConfig): Promise<VersionCommandResult> {
  return runVersionCommand({
    command: buildPmdShellCommand(toolConfig.command, ["--version"]),
    cwd: rootDir,
    shell: true
  });
}

function runVersionCommand({
  args = [],
  command,
  cwd,
  shell = false
}: {
  args?: string[];
  command: string;
  cwd: string;
  shell?: boolean;
}): Promise<VersionCommandResult> {
  return new Promise((resolve) => {
    const child = spawn(command, args, {
      cwd,
      shell,
      windowsHide: true
    });
    let stdout = "";
    let stderr = "";
    let settled = false;

    const finish = (result: VersionCommandResult) => {
      if (settled) return;
      settled = true;
      resolve(result);
    };

    child.stdout?.on("data", (chunk: Buffer | string) => {
      stdout += chunk.toString();
    });
    child.stderr?.on("data", (chunk: Buffer | string) => {
      stderr += chunk.toString();
    });
    child.on("error", (error) => finish({ status: null, signal: null, stdout, stderr, error }));
    child.on("close", (status, signal) => finish({ status, signal, stdout, stderr }));
  });
}
