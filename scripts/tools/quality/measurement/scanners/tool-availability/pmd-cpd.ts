import { runProcess } from "../../../../process.ts";
import { DEFAULT_CONFIG } from "../../../model/config.ts";
import type { ToolAvailability } from "../../../model/schema.ts";
import { parsePmdVersionOutput } from "../pmd-cpd/scanner.ts";

export async function checkPmdCpd(rootDir: string): Promise<ToolAvailability> {
  try {
    const result = await runProcess({
      args: ["--version"],
      command: DEFAULT_CONFIG.tools.pmdCpd.command,
      cwd: rootDir
    });
    if (result.error) {
      return pmdProcessErrorAvailability(result.error);
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

function pmdProcessErrorAvailability(error: Error): ToolAvailability {
  const code = (error as NodeJS.ErrnoException).code;
  return {
    name: "pmd-cpd",
    available: false,
    version: null,
    error: code === "ENOENT" ? "PMD not installed" : `PMD version error: ${error.message}`,
    source: "system"
  };
}
