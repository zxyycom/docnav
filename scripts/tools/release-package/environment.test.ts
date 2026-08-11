import assert from "node:assert/strict";
import test from "node:test";

import { parseWorkspaceVersion } from "./environment.ts";

test("workspace version uses the complete Cargo workspace member mapping", () => {
  const metadata = Object.freeze({
    packages: Object.freeze([
      Object.freeze({ id: "path+file:///workspace/one#0.1.0", version: "0.1.0" }),
      Object.freeze({ id: "path+file:///workspace/two#0.1.0", version: "0.1.0" }),
    ]),
    workspace_members: Object.freeze([
      "path+file:///workspace/one#0.1.0",
      "path+file:///workspace/two#0.1.0",
    ]),
  });

  assert.equal(
    parseWorkspaceVersion(metadata),
    "0.1.0",
  );
});

test("workspace version rejects malformed Cargo metadata records instead of filtering them", () => {
  const cases: Array<{ diagnostic: RegExp; metadata: unknown }> = [
    {
      diagnostic: /workspace_members must be an array/,
      metadata: { packages: [], workspace_members: "member" },
    },
    {
      diagnostic: /workspace_members\[1\] must be a non-empty string/,
      metadata: { packages: [], workspace_members: ["member", 42] },
    },
    {
      diagnostic: /packages\[1\]\.version must be a non-empty string/,
      metadata: {
        packages: [
          { id: "one", version: "1.0.0" },
          { id: "two" },
        ],
        workspace_members: ["one", "two"],
      },
    },
  ];

  for (const { diagnostic, metadata } of cases) {
    assert.throws(() => parseWorkspaceVersion(metadata), diagnostic);
  }
});

test("workspace version rejects incomplete or ambiguous Cargo member mappings", () => {
  const cases: Array<{ diagnostic: RegExp; metadata: unknown }> = [
    {
      diagnostic: /workspace member two has no matching package/,
      metadata: {
        packages: [{ id: "one", version: "1.0.0" }],
        workspace_members: ["one", "two"],
      },
    },
    {
      diagnostic: /expected one workspace version, found 2/,
      metadata: {
        packages: [
          { id: "one", version: "1.0.0" },
          { id: "two", version: "2.0.0" },
        ],
        workspace_members: ["one", "two"],
      },
    },
  ];

  for (const { diagnostic, metadata } of cases) {
    assert.throws(() => parseWorkspaceVersion(metadata), diagnostic);
  }
});
