import { errorMessage } from "../../foundation/src/errors.ts";
import { parseReleaseTarget } from "../target.ts";

export type ProducerKind = "github-actions" | "local";

export function parseOptionalTargetValue(value: string | undefined): string | null {
  return value === undefined ? null : parseReleaseTarget(value);
}

export function parseOptionalBoolean(value: string | undefined, label: string): boolean | null {
  if (value === undefined) {
    return null;
  }
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  throw new Error(`${label} must be true or false`);
}

export function parseOptionalProducerKind(value: string | undefined): ProducerKind | null {
  if (value === undefined) {
    return null;
  }
  if (value === "local" || value === "github-actions") {
    return value;
  }
  throw new Error("--expect-producer-kind must be local or github-actions");
}

export function normalizeParseArgsError(error: unknown): string {
  return normalizeParseArgsMessage(errorMessage(error));
}

function normalizeParseArgsMessage(message: string): string {
  const unknownOption = message.match(/^Unknown option '(--[^']+)'/);
  if (unknownOption) {
    return `unknown option ${unknownOption[1]}`;
  }

  const missingOptionValue = message.match(/^Option '(--[^ ]+) <value>' argument missing$/);
  if (missingOptionValue) {
    return `${missingOptionValue[1]} requires a value`;
  }

  const unexpectedArgument = message.match(/^Unexpected argument '([^']+)'/);
  if (unexpectedArgument) {
    return `unexpected positional argument ${unexpectedArgument[1]}`;
  }

  return message;
}
