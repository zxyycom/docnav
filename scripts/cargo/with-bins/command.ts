import type { ScriptArgToken } from "../../tools/args.ts";

export function commandFromTokens(tokens: readonly ScriptArgToken[]): [string, ...string[]] {
  const separator = tokens.find((token) => token.kind === "option-terminator");
  if (!separator) {
    throw new Error("missing -- command separator");
  }

  const unexpectedPositional = tokens.find(
    (token) => token.kind === "positional" && token.index < separator.index
  );
  if (unexpectedPositional) {
    throw new Error(`unexpected positional argument before --: ${unexpectedPositional.value ?? ""}`);
  }

  const command = tokens
    .filter((token) => token.kind === "positional" && token.index > separator.index)
    .map((token) => token.value)
    .filter((value): value is string => value !== undefined);
  if (command.length === 0) {
    throw new Error("missing command after --");
  }

  return command as [string, ...string[]];
}
