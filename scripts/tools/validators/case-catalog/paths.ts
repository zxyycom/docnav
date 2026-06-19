export function normalizeCasePath(value: string): string {
  return value.replaceAll("\\", "/");
}
