import type { ProcessResult } from "../../../../process.ts";
import type { PmdCpdScanResult } from "./types.ts";
import { parsePmdCpdXml } from "./xml.ts";

const CPD_XML_ROOT_PATTERN = /<pmd-cpd\b/;

type HandlePmdCpdProcessResultOptions = {
  child: ProcessResult;
  cwd: string;
  skipIfUnavailable: boolean;
};

export function handlePmdCpdProcessResult({
  child,
  cwd,
  skipIfUnavailable
}: HandlePmdCpdProcessResultOptions): PmdCpdScanResult {
  if (child.error) {
    return pmdCpdProcessError(child.error);
  }

  if (isFailedExitStatus(child.status)) {
    return handleFailedPmdCpdExit({
      status: child.status,
      stdout: child.stdout,
      stderr: child.stderr,
      cwd,
      skipIfUnavailable
    });
  }

  return parseSuccessfulPmdCpdOutput(child.stdout, cwd);
}

function pmdCpdProcessError(error: Error): PmdCpdScanResult {
  if ((error as NodeJS.ErrnoException).code === "ENOENT") {
    return {
      ok: false,
      skipped: true,
      error: `PMD CPD not found: ${error.message}`,
      reason: "tool-unavailable"
    };
  }
  return {
    ok: false,
    skipped: false,
    error: `PMD CPD process error: ${error.message}`
  };
}

function isFailedExitStatus(status: number | null): status is number {
  return status !== 0 && status !== null;
}

type FailedPmdCpdExitOptions = {
  cwd: string;
  skipIfUnavailable: boolean;
  status: number;
  stderr: string;
  stdout: string;
};

function handleFailedPmdCpdExit({
  status,
  stdout,
  stderr,
  cwd,
  skipIfUnavailable
}: FailedPmdCpdExitOptions): PmdCpdScanResult {
  if (status === 4) {
    return handlePmdCpdExitFour({ status, stdout, cwd, skipIfUnavailable });
  }

  return pmdCpdExecutionFailure(
    status,
    trimmedProcessOutput(stderr, stdout),
    skipIfUnavailable
  );
}

function handlePmdCpdExitFour({
  status,
  stdout,
  cwd,
  skipIfUnavailable
}: Omit<FailedPmdCpdExitOptions, "stderr">): PmdCpdScanResult {
  if (!stdout.trim()) {
    return pmdCpdExecutionFailure(status, "no output", skipIfUnavailable);
  }
  if (!CPD_XML_ROOT_PATTERN.test(stdout)) {
    return pmdCpdExecutionFailure(status, "missing PMD CPD XML output", skipIfUnavailable);
  }
  return parsePmdCpdXml(stdout, cwd);
}

function parseSuccessfulPmdCpdOutput(stdout: string, cwd: string): PmdCpdScanResult {
  if (!stdout) {
    return { ok: true, fragments: [] };
  }

  return parsePmdCpdXml(stdout, cwd);
}

function trimmedProcessOutput(stderr: string, stdout: string): string {
  return stderr.trim() || stdout.trim() || "no output";
}

function pmdCpdExecutionFailure(status: number, output: string, skipIfUnavailable: boolean): PmdCpdScanResult {
  return {
    ok: false,
    skipped: true,
    error: `PMD CPD exit ${status}: ${output}`,
    reason: skipIfUnavailable ? "cpd-scan-skipped" : "cpd-execution-error"
  };
}
