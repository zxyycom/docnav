import type { ProcessResult } from "../../process.ts";
import type { CpdScanResult } from "./cpd-types.ts";
import { parseCpdXml } from "./cpd-xml.ts";

const CPD_XML_ROOT_PATTERN = /<pmd-cpd\b/;

type HandleCpdProcessResultOptions = {
  child: ProcessResult;
  cwd: string;
  skipIfUnavailable: boolean;
};

export function handleCpdProcessResult({
  child,
  cwd,
  skipIfUnavailable
}: HandleCpdProcessResultOptions): CpdScanResult {
  if (child.error) {
    return cpdProcessError(child.error);
  }

  if (isFailedExitStatus(child.status)) {
    return handleFailedCpdExit({
      status: child.status,
      stdout: child.stdout,
      stderr: child.stderr,
      cwd,
      skipIfUnavailable
    });
  }

  return parseSuccessfulCpdOutput(child.stdout, cwd);
}

function cpdProcessError(error: Error): CpdScanResult {
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

type FailedCpdExitOptions = {
  cwd: string;
  skipIfUnavailable: boolean;
  status: number;
  stderr: string;
  stdout: string;
};

function handleFailedCpdExit({
  status,
  stdout,
  stderr,
  cwd,
  skipIfUnavailable
}: FailedCpdExitOptions): CpdScanResult {
  if (status === 4) {
    return handleCpdExitFour({ status, stdout, cwd, skipIfUnavailable });
  }

  return cpdExecutionFailure(
    status,
    trimmedProcessOutput(stderr, stdout),
    skipIfUnavailable
  );
}

function handleCpdExitFour({
  status,
  stdout,
  cwd,
  skipIfUnavailable
}: Omit<FailedCpdExitOptions, "stderr">): CpdScanResult {
  if (!stdout.trim()) {
    return cpdExecutionFailure(status, "no output", skipIfUnavailable);
  }
  if (!CPD_XML_ROOT_PATTERN.test(stdout)) {
    return cpdExecutionFailure(status, "missing PMD CPD XML output", skipIfUnavailable);
  }
  return parseCpdXml(stdout, cwd);
}

function parseSuccessfulCpdOutput(stdout: string, cwd: string): CpdScanResult {
  if (!stdout) {
    return { ok: true, fragments: [] };
  }

  return parseCpdXml(stdout, cwd);
}

function trimmedProcessOutput(stderr: string, stdout: string): string {
  return stderr.trim() || stdout.trim() || "no output";
}

function cpdExecutionFailure(status: number, output: string, skipIfUnavailable: boolean): CpdScanResult {
  return {
    ok: false,
    skipped: true,
    error: `PMD CPD exit ${status}: ${output}`,
    reason: skipIfUnavailable ? "cpd-scan-skipped" : "cpd-execution-error"
  };
}
