import sourcemaps from 'rollup-plugin-sourcemaps';
import typescript from '@rollup/plugin-typescript';

export default [
    {
        input: 'crates/vertigo/src/driver_module/src_js/exec_command/command/dom/hydration.test.ts',
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
    }
];
