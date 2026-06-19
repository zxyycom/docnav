import { writeJsonFile } from "../fs.ts";

export function writeQualityJsonArtifact(filePath: string, value: unknown): void {
  writeJsonFile(filePath, value, { trailingNewline: false });
}
