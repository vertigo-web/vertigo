import { rmSync } from 'node:fs';
import sourcemaps from 'rollup-plugin-sourcemaps';
import terser from '@rollup/plugin-terser';
import typescript from '@rollup/plugin-typescript';

const dir = 'crates/vertigo/src/driver_module';

// Code imported dynamically goes to a chunk of its own in `chunks/`, named by its content, which
// vertigo's build.rs and vertigo-cli copy next to `wasm_run.js` as they find it. The chunks of
// the last build are removed, so that only those of this one are committed.
//
// A chunk may import nothing: what it shares with `wasm_run.js` it would import from there, and
// vertigo-cli renames `wasm_run.js`. Such code has to be the chunk's own.
const chunks = () => ({
  name: 'vertigo-chunks',
  generateBundle(_options, bundle) {
    for (const file of Object.values(bundle)) {
      if (file.type === 'chunk' && !file.isEntry && file.imports.length > 0) {
        this.error(`${file.fileName} imports ${file.imports.join(', ')}, a chunk may import nothing`);
      }
    }

    rmSync(`${dir}/chunks`, { recursive: true, force: true });
  },
});

export default [
  {
    input: 'crates/vertigo/src/driver_module/src_js/index.ts',
    output: [
      {
        sourcemap: true,
        dir,
        entryFileNames: 'wasm_run.js',
        chunkFileNames: 'chunks/[name].[hash:11].js',
        format: 'es',
      }
    ],
    plugins: [
      typescript({
        sourceMap: true,
        inlineSources: true,
      }),
      sourcemaps(),
      terser(),
      chunks(),
    ],
  }
];
