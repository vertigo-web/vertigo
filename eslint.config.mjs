import js from '@eslint/js';
import comments from '@eslint-community/eslint-plugin-eslint-comments/configs';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

const SRC = 'crates/vertigo/src/driver_module/src_js';

export default defineConfig(
  {
    files: [`${SRC}/**/*.ts`],
    extends: [
      js.configs.recommended,
      tseslint.configs.strictTypeChecked,
      comments.recommended,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    linterOptions: {
      reportUnusedDisableDirectives: 'error',
    },
    rules: {
      // Types come from narrowing, generics or type predicates, never from an assertion.
      // `as const` is not an assertion and stays allowed.
      '@typescript-eslint/consistent-type-assertions': ['error', { assertionStyle: 'never' }],
      // The rare cast that has to stay says why.
      '@eslint-community/eslint-comments/require-description': 'error',
      // Ids go into messages everywhere, `id=${id}`.
      '@typescript-eslint/restrict-template-expressions': ['error', { allowNumber: true }],
      // `(value) => this.wasmCallback(id, value)` and `forEach((name) => el.addEventListener(...))`
      // are how the code is written.
      '@typescript-eslint/no-confusing-void-expression': ['error', { ignoreArrowShorthand: true }],
      // `while (true)` in the readers of the wire formats.
      '@typescript-eslint/no-unnecessary-condition': ['error', { allowConstantLoopConditions: 'only-allowed-literals' }],
      // An underscore marks a parameter or binding that is unused on purpose, as for `tsc`.
      '@typescript-eslint/no-unused-vars': ['error', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_',
      }],
    },
  },
  {
    // Tests build mocks of the DOM and of wasm, which is what casts and `any` are for.
    files: [`${SRC}/**/*.test.ts`, `${SRC}/testing/**/*.ts`],
    rules: {
      '@typescript-eslint/consistent-type-assertions': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
    },
  },
);
