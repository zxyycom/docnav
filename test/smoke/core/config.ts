import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const logDir = path.join(root, ".log", "smoke", "core");
export const timestamp = new Date().toISOString().replace(/[:]/g, "-");
const smokeRunId = `${timestamp}-${process.pid}`;
export const tempRoot = resolveCoreSmokeTempRoot(
  root,
  process.env.DOCNAV_CORE_SMOKE_TEMP_ROOT,
  smokeRunId
);
export const logPaths = [
  path.join(logDir, "latest.log"),
  path.join(logDir, `${timestamp}.log`)
];

export const schemaPaths = {
  protocolResponse: "docs/schemas/protocol-response.schema.json"
};

export const exitCodes = Object.freeze({
  success: 0,
  internal: 1,
  input: 2,
  documentRefFormat: 3,
  protocolOrAdapterProcess: 4
});

/** Resolve a unique run directory below a caller-owned or repository-owned base. */
function resolveCoreSmokeTempRoot(
  workspaceRoot: string,
  overrideBase: string | undefined,
  runId: string
): string {
  const base = overrideBase
    ? path.resolve(workspaceRoot, overrideBase)
    : path.join(workspaceRoot, ".tmp", "docnav", "smoke", "core");
  const resolved = path.resolve(base, runId);
  const relative = path.relative(base, resolved);

  if (!relative || relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new Error("core smoke run directory must be a strict child of its temp base");
  }

  return resolved;
}
