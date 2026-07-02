import path from "node:path";
import { fileURLToPath } from "node:url";

export const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const logDir = path.join(root, ".log", "smoke", "core");
export const timestamp = new Date().toISOString().replace(/[:]/g, "-");
export const tempRoot = process.env.DOCNAV_CORE_SMOKE_TEMP_ROOT
  ? path.resolve(root, process.env.DOCNAV_CORE_SMOKE_TEMP_ROOT)
  : path.join(root, ".tmp", "docnav", "smoke", "core", timestamp);
export const logPaths = [
  path.join(logDir, "latest.log"),
  path.join(logDir, `${timestamp}.log`)
];

export const schemaPaths = {
  protocolResponse: "docs/schemas/protocol-response.schema.json",
  readableOutline: "docs/schemas/readable-outline.schema.json",
  readableRead: "docs/schemas/readable-read.schema.json",
  readableFind: "docs/schemas/readable-find.schema.json",
  readableInfo: "docs/schemas/readable-info.schema.json",
  readableError: "docs/schemas/readable-error.schema.json"
};

export const exitCodes = Object.freeze({
  success: 0,
  internal: 1,
  input: 2,
  documentRefFormat: 3,
  protocolOrAdapterProcess: 4
});
