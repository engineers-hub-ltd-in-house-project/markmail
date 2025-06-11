import js from "@eslint/js";
import ts from "@typescript-eslint/eslint-plugin";
import tsParser from "@typescript-eslint/parser";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import svelteParser from "svelte-eslint-parser";

export default [
  js.configs.recommended,
  {
    ignores: [
      ".svelte-kit/**",
      "node_modules/**",
      "build/**",
      "dist/**",
      "src/tests/**",
      "src/setupTests.ts",
    ],
  },
  {
    files: ["**/*.{js,ts,svelte}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 2020,
        sourceType: "module",
        extraFileExtensions: [".svelte"],
      },
      globals: {
        console: "readonly",
        fetch: "readonly",
        Headers: "readonly",
        Request: "readonly",
        Response: "readonly",
        RequestInit: "readonly",
        localStorage: "readonly",
        window: "readonly",
        document: "readonly",
        confirm: "readonly",
        alert: "readonly",
      },
    },
    plugins: {
      "@typescript-eslint": ts,
    },
    rules: {
      ...ts.configs.recommended.rules,
      // TypeScript ルール - 開発中は警告に緩和
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-expressions": "warn",
      "@typescript-eslint/explicit-function-return-type": "off",
      "@typescript-eslint/explicit-module-boundary-types": "off",

      // 一般的なルール
      "no-console": "warn",
      "no-debugger": "error",
      "prefer-const": "error",
      "no-var": "error",
    },
  },
  {
    files: ["**/*.config.js", "**/*.config.ts"],
    languageOptions: {
      globals: {
        require: "readonly",
        module: "readonly",
        __dirname: "readonly",
        __filename: "readonly",
        exports: "readonly",
        process: "readonly",
      },
    },
  },
  {
    files: ["**/*.test.{js,ts}", "**/*.spec.{js,ts}", "**/setupTests.ts"],
    languageOptions: {
      globals: {
        describe: "readonly",
        it: "readonly",
        expect: "readonly",
        test: "readonly",
        beforeEach: "readonly",
        afterEach: "readonly",
        beforeAll: "readonly",
        afterAll: "readonly",
        vi: "readonly",
        global: "writable",
      },
    },
    rules: {
      "no-undef": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": "off",
    },
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tsParser,
      },
    },
    plugins: {
      svelte,
    },
    rules: {
      ...svelte.configs.recommended.rules,
      // 開発中は一時的に警告に緩和
      "svelte/no-at-html-tags": "warn",
      "svelte/no-target-blank": "error",
      "svelte/no-reactive-functions": "warn",
      "svelte/no-reactive-literals": "warn",
      "no-console": "warn",
      "@typescript-eslint/no-unused-vars": "warn",
      "@typescript-eslint/no-unused-expressions": "warn",
    },
  },
  prettier,
];
