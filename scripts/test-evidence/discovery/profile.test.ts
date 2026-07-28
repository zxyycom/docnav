import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { discoverTestEntities } from "../discover.ts";
import { parseBunJUnit } from "./bun.ts";
import { resolveBunTestFiles } from "./bun-files.ts";
import { parseLibtestList } from "./rust/cargo.ts";
import {
  loadSupportedRunnerProfile,
  workspaceRoot
} from "../profile.ts";

test("parses stable Cargo and Bun runner reports without inferring missing fields", () => {
  assert.deepEqual(
    parseLibtestList([
      "tests::first: test",
      "tests::ignored_but_selectable: test",
      "benchmark: benchmark",
      ""
    ].join("\n")),
    [
      "tests::first",
      "tests::ignored_but_selectable"
    ]
  );
  assert.deepEqual(
    parseBunJUnit([
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
      "<testsuites tests=\"1\" failures=\"0\">",
      "  <testcase name=\"rejects &quot;bad&quot; input\" classname=\"suite\" file=\"tests/example.test.ts\" line=\"7\" />",
      "</testsuites>"
    ].join("\n")),
    [
      {
        name: "rejects \"bad\" input",
        className: "suite",
        file: "tests/example.test.ts",
        line: 7
      }
    ]
  );
  assert.throws(
    () => parseBunJUnit("<testsuites tests=\"1\" failures=\"0\"></testsuites>"),
    /contains 0 testcase/
  );
});

test("loads one versioned and sorted supported runner profile", async () => {
  const profile = loadSupportedRunnerProfile();
  assert.equal(profile.schemaVersion, 2);
  assert.equal(profile.id, "docnav-native-tests");
  assert.equal(profile.version, 2);
  assert.deepEqual(
    profile.bun.sourceRoots,
    [...profile.bun.sourceRoots].sort((left, right) => left.localeCompare(right))
  );
  assert.deepEqual(
    resolveBunTestFiles({ workspaceRoot, profile: profile.bun }),
    findConventionalBunTests(workspaceRoot, profile.bun.sourceRoots)
  );

  const temporaryRoot = fs.mkdtempSync(path.join(
    os.tmpdir(),
    "docnav-runner-profile-"
  ));
  try {
    const invalidProfiles = [
      {
        ...profile,
        id: 1
      },
      {
        ...profile,
        cargo: {
          ...profile.cargo,
          sourceRoots: ["../outside"]
        }
      },
      {
        ...profile,
        bun: {
          ...profile.bun,
          sourceRoots: []
        }
      },
      {
        ...profile,
        bun: {
          ...profile.bun,
          include: ["../**/*.test.ts"]
        }
      },
      {
        ...profile,
        smoke: {
          ...profile.smoke,
          factory: "test/smoke/other/profile.ts"
        }
      },
      {
        ...profile,
        smoke: {
          ...profile.smoke,
          sourceRoots: ["/tmp"]
        }
      }
    ];
    for (const [index, invalidProfile] of invalidProfiles.entries()) {
      const sourcePath = path.join(temporaryRoot, `${index}.json`);
      writeJson(sourcePath, invalidProfile);
      assert.throws(
        () => loadSupportedRunnerProfile(sourcePath),
        /identity|safe relative POSIX paths|non-empty string array|positive relative POSIX globs|smoke identity/
      );
    }
  } finally {
    fs.rmSync(temporaryRoot, { force: true, recursive: true });
  }

  const rootMismatch = await discoverTestEntities({
    workspaceRoot: os.tmpdir()
  });
  assert.ok(rootMismatch.diagnostics.some(({ code, message }) => (
    code === "runner-profile-invalid" &&
    message.includes("current checkout")
  )));
});

function findConventionalBunTests(
  root: string,
  sourceRoots: readonly string[]
): string[] {
  const files: string[] = [];
  for (const sourceRoot of sourceRoots) {
    visit(path.join(root, sourceRoot), sourceRoot);
  }
  return files.sort();

  function visit(directoryPath: string, relativeDirectory: string): void {
    for (const entry of fs.readdirSync(directoryPath, { withFileTypes: true })) {
      const relativePath = path.posix.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) {
        visit(path.join(directoryPath, entry.name), relativePath);
      } else if (entry.isFile() && entry.name.endsWith(".test.ts")) {
        files.push(relativePath);
      }
    }
  }
}

function writeJson(targetPath: string, value: unknown): void {
  fs.writeFileSync(targetPath, `${JSON.stringify(value, null, 2)}\n`);
}
