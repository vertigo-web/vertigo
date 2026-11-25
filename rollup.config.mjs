import { readdirSync, rmSync } from 'node:fs';
import sourcemaps from 'rollup-plugin-sourcemaps';
import terser from '@rollup/plugin-terser';
import typescript from '@rollup/plugin-typescript';

const dir = 'crates/vertigo/src/driver_module';

// Code imported dynamically goes to a chunk of its own next to `wasm_run.js`, named by its
// content (`name.HASH.js`), which vertigo's build.rs and vertigo-cli copy along with it. With no
// directory of their own: a build is often copied without its directories (`COPY build/*` in a
// Dockerfile), and a page that can't import the runtime doesn't start. The chunks of the last
// build are removed, so that only those of this one are committed.
//
// `wasm_run.js` is the bootstrap alone (`boot.ts`). The runtime, all the rest a page needs to
// run its wasm, is the chunk `runtime`, and what the runtime imports dynamically are chunks
// next to it.
//
// A chunk may import nothing: what it shares with `wasm_run.js` it would import from there, and
// vertigo-cli renames `wasm_run.js`. Such code has to be the chunk's own.
// A chunk or its source map: any JavaScript in `dir` but `wasm_run.js`. The same in vertigo's
// build.rs and in vertigo-cli.
const isChunk = (name) => /\.js(\.map)?$/.test(name) && !name.startsWith('wasm_run.js');

const chunks = () => ({
  name: 'vertigo-chunks',
  generateBundle(_options, bundle) {
    for (const file of Object.values(bundle)) {
      if (file.type === 'chunk' && !file.isEntry && file.imports.length > 0) {
        this.error(`${file.fileName} imports ${file.imports.join(', ')}, a chunk may import nothing`);
      }
    }

    for (const name of readdirSync(dir)) {
      if (isChunk(name)) {
        rmSync(`${dir}/${name}`);
      }
    }
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
        chunkFileNames: '[name].[hash:11].js',
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
