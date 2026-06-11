export function findCargoExecutable(output, binName) {
  let executable = null;

  for (const line of output.split(/\r?\n/)) {
    if (line.trim().length === 0) {
      continue;
    }

    let message;
    try {
      message = JSON.parse(line);
    } catch {
      continue;
    }

    if (
      message.reason === "compiler-artifact" &&
      message.executable &&
      message.target?.name === binName &&
      message.target?.kind?.includes("bin")
    ) {
      executable = message.executable;
    }
  }

  return executable;
}
