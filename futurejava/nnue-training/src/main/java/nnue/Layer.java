package nnue;

import java.util.Arrays;

public class Layer {
    private final short[] weights;
    private final short[] biases;
    private final short[] activations;

    public Layer(short[] weights, short[] biases) {
        this.weights = weights;
        this.biases = biases;
        this.activations = Arrays.copyOf(biases, biases.length);
    }

    public short[] getWeights() {
        return weights;
    }

    public short[] getBiases() {
        return biases;
    }

    public short[] getActivations() {
        return activations;
    }
}
