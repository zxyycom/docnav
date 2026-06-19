export function toSlashPath(filePath: string): string {
  return filePath.replace(/\\/g, "/");
}
