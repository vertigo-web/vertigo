import sourcemaps from 'rollup-plugin-sourcemaps';
import typescript from '@rollup/plugin-typescript';

export default [
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/hydration.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/hydration.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/dom/dom_wire.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/dom_wire.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    },
    {
        input: 'crates/vertigo/src/driver_module/src_js/api/command/fetchExec.test.ts',
        output: [
            {
                sourcemap: true,
                file: 'build/fetchExec.test.js',
                format: 'cjs',
            }
        ],
        plugins: [
            typescript({
                sourceMap: true,
                inlineSources: true,
            }),
            sourcemaps(),
        ],
    }
];
