export const PLAIN_TEXT_PROCESS_ENV = Object.freeze({
  CARGO_TERM_COLOR: "never",
  CLICOLOR: "0",
  CLICOLOR_FORCE: "0",
  FORCE_COLOR: "0",
  NO_COLOR: "1",
  PNPM_CONFIG_COLOR: "false",
  PY_COLORS: "0",
  TERM: "dumb",
  UV_NO_COLOR: "1",
  npm_config_color: "false"
} satisfies NodeJS.ProcessEnv);

export function plainTextProcessEnv(env: NodeJS.ProcessEnv = process.env): NodeJS.ProcessEnv {
  return {
    ...env,
    ...PLAIN_TEXT_PROCESS_ENV
  };
}
