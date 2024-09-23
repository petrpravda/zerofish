import typescript from 'rollup-plugin-typescript2';
import { terser } from 'rollup-plugin-terser';
import resolve from 'rollup-plugin-node-resolve';
import commonjs from 'rollup-plugin-commonjs';

export default {
  input: 'src/index.ts',
  output: [
    {
      file: 'dist/bundle.cjs.js', // CommonJS format
      format: 'cjs',
      sourcemap: true,
    },
    {
      file: 'dist/bundle.esm.js', // ES module format
      format: 'esm',
      sourcemap: true,
    },
  ],
  plugins: [
    resolve(),
    commonjs(),
    typescript({ tsconfig: './tsconfig.json' }),
    terser(), // Optional: minify the output
  ],
  external: [], // Add external dependencies here if necessary
};



// import typescript from '@rollup/plugin-typescript';
// import { dts } from "rollup-plugin-dts";
//
// export default [
//   {
//     input: 'src/__tests__/testing.perft.helper.ts',
//     output: {
//       file: 'dist/ts-engine.js',
//       format: 'es',
//     },
//     plugins: [
//       typescript({
//         declaration: true,        // Generate .d.ts files
//         declarationDir: './dist/types',  // Directory to store individual .d.ts files
//       }),
//       dts
//     ]
//   },
//   // {
//   //   // Bundle the individual .d.ts files into a single bundle.d.ts
//   //   input: './dist/types/__tests__/testing.perft.helper.d.ts',
//   //   output: {
//   //     file: 'dist/bundle.d.ts',
//   //     format: 'es',
//   //   },
//   //   plugins: [dts()],
//   // }
// ];
