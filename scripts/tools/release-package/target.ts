const rustTargetTriplePattern = /^[A-Za-z0-9_]+(?:\.[A-Za-z0-9_]+)*(?:-[A-Za-z0-9_]+(?:\.[A-Za-z0-9_]+)*)+$/;

export function parseReleaseTarget(value: string): string {
  if (!rustTargetTriplePattern.test(value)) {
    throw new Error(
      "--target must be a Rust target triple using letters, digits, underscores, dots, and hyphens",
    );
  }
  return value;
}
