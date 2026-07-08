import { describe, expect, test } from "bun:test";

import {
  parseJsonValue,
  parsePositiveInteger,
  processFailed,
  toSlashPath
} from "../src/index.ts";

describe("script foundation", () => {
  test("parses strict positive integers", () => {
    expect(parsePositiveInteger("4", "concurrency")).toBe(4);
    expect(() => parsePositiveInteger("0", "concurrency")).toThrow("concurrency must be a positive integer");
  });

  test("parses JSON values and normalizes slash paths", () => {
    expect(parseJsonValue("{\"ok\":true}")).toEqual({ ok: true });
    expect(toSlashPath("a\\b\\c.ts")).toBe("a/b/c.ts");
  });

  test("detects failed process results", () => {
    expect(processFailed({ status: 1 })).toBe(true);
    expect(processFailed({ status: 0 })).toBe(false);
  });
});
