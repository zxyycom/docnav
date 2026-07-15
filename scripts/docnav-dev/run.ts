import { runProcessSync } from "../tools/foundation/src/process.ts";

const binaryPath = process.env.DOCNAV_BIN;
if (!binaryPath) {
  console.error("docnav binary path is unavailable");
  process.exit(1);
}

const result = runProcessSync(binaryPath, process.argv.slice(2), {
  cwd: process.cwd(),
  env: process.env,
  stdio: "inherit"
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
