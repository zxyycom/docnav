import { runProcessSync } from "../tools/process.ts";

const binaries = new Map([
  ["docnav", process.env.DOCNAV_BIN],
  ["docnav-markdown", process.env.DOCNAV_MARKDOWN_BIN],
]);
const args = process.argv.slice(2);

if (args.length === 0) {
  for (const [name, binaryPath] of binaries) {
    assertBinaryPath(name, binaryPath);
    console.log(`${name}: ${binaryPath}`);
  }
  process.exit(0);
}

const firstArg = args[0];
const requestedBinary: string = firstArg && binaries.has(firstArg) ? args.shift()! : "docnav";
const binaryPath = binaries.get(requestedBinary);
assertBinaryPath(requestedBinary, binaryPath);

const result = runProcessSync(binaryPath, args, {
  cwd: process.cwd(),
  env: process.env,
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);

function assertBinaryPath(name: string, binaryPath: string | undefined): asserts binaryPath is string {
  if (!binaryPath) {
    console.error(`${name} binary path is unavailable`);
    process.exit(1);
  }
}
