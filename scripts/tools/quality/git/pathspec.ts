export function gitGlobPathspecs(patterns: readonly string[]): string[] {
  return patterns.map((pattern) => `:(glob)${pattern}`);
}
