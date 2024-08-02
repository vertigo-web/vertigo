import sourcemaps from 'rollup-plugin-sourcemaps';
import terser from '@rollup/plugin-terser';
import typescript from '@rollup/plugin-typescript';

export default [
  {
    input: 'crates/vertigo/src/driver_module/src_js/index.ts',
    output: [
      {
        sourcemap: true,
        file: 'crates/vertigo/src/driver_module/wasm_run.js',
        format: 'cjs',
      }
    ],
    plugins: [
      typescript({
        sourceMap: true,
        inlineSources: true,
      }),
      sourcemaps(),
      terser(),
    ],
  }
];
