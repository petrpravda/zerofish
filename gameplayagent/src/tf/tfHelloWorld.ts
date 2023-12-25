import * as tf from '@tensorflow/tfjs';
import {NAME} from './mylocal';

console.info(NAME);

// Define two 2x2 matrices.
let matrix1 = tf.tensor2d([1, 2, 3, 4], [2, 2]);
let matrix2 = tf.tensor2d([5, 6, 7, 8], [2, 2]);

// Multiply the matrices.
let result = tf.matMul(matrix1, matrix2);

// Print the result.
result.print();
