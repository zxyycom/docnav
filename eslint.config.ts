import eslint from "@eslint/js";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import globals from "globals";
import tseslint from "typescript-eslint";

const tsconfigRootDir = dirname(fileURLToPath(import.meta.url));
const scriptFiles = ["scripts/**/*.ts", "test/**/*.ts"];

export default tseslint.config(
  {
    ignores: [
      ".git/**",
      ".log/**",
      "build/**",
      "dist/**",
      "node_modules/**",
      "target/**"
    ]
  },
  {
    files: ["eslint.config.ts", ...scriptFiles],
    extends: [
      eslint.configs.recommended,
      ...tseslint.configs.recommended
    ],
    languageOptions: {
      ecmaVersion: "latest",
      globals: globals.node,
      sourceType: "module"
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          args: "after-used",
          argsIgnorePattern: "^_",
          caughtErrors: "all",
          caughtErrorsIgnorePattern: "^_",
          varsIgnorePattern: "^_"
        }
      ],
      "no-unused-vars": "off"
    }
  },
  {
    files: scriptFiles,
    languageOptions: {
      parserOptions: {
        project: "./tsconfig.scripts.json",
        tsconfigRootDir
      }
    },
    rules: {
      "@typescript-eslint/no-unsafe-argument": "error",
      "@typescript-eslint/no-unsafe-assignment": "error",
      "@typescript-eslint/no-unsafe-call": "error",
      "@typescript-eslint/no-unsafe-member-access": "error",
      "@typescript-eslint/no-unsafe-return": "error"
    }
  }
);
