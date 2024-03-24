import resolve from '@rollup/plugin-node-resolve';
import typescript from '@rollup/plugin-typescript';
import terser from '@rollup/plugin-terser';
import wasm from '@rollup/plugin-wasm';

export default {
    input: ['./src/index.ts'],
    output: [
        {
            dir: 'dist',
            format: 'esm', // ES module format
            sourcemap: true
        }
    ],
    plugins: [
        typescript({ tsconfig: './tsconfig.json' }), // Ensure you reference your TS config
        resolve({ browser: true, extensions: ['.js', '.ts', '.wasm'], resolveOnly: [/^@marecchia\/.*/]}),
        wasm({
            // browser or auto-inline?
            targetEnv: "auto-inline",
            sync: [
                '../crates/marecchia-p2p/pkg/marecchia_bg.wasm'
            ]
        }), // Enable WebAssembly support
        terser(), // Minify the output (optional)
    ]
};
