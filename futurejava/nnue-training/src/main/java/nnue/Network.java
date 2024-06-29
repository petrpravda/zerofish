package nnue;

import jdk.incubator.vector.*;

import static nnue.Constants.INPUT_LAYER_BIAS;
import static nnue.Constants.INPUT_LAYER_WEIGHT;
import static nnue.Constants.L1_BIAS;
import static nnue.Constants.L1_WEIGHT;

public class Network {
    private final Layer inputLayer;
    private final Layer hiddenLayer;
    private static final short SCALE = 64;
    private static final int NNUE2SCORE = 600;

    public Network() {
        this.inputLayer = new Layer(INPUT_LAYER_WEIGHT, INPUT_LAYER_BIAS);
        this.hiddenLayer = new Layer(L1_WEIGHT, L1_BIAS);
    }

    public void movePiece(Piece piece, int fromSq, int toSq) {
        deactivate(piece, fromSq);
        activate(piece, toSq);
    }

    public void activate(Piece piece, int sq) {
        updateActivation(piece, sq, (activation, weight) -> (short) (activation + weight));
    }

    public void deactivate(Piece piece, int sq) {
        updateActivation(piece, sq, (activation, weight) -> (short) (activation - weight));
    }

    private void updateActivation(Piece piece, int sq, ActivationUpdater updater) {
        int featureIdx = (piece.index() * SQ.N_SQUARES + sq) * inputLayer.getActivations().length;
        for (int i = 0; i < inputLayer.getActivations().length; i++) {
            inputLayer.getActivations()[i] = updater.update(inputLayer.getActivations()[i], inputLayer.getWeights()[featureIdx + i]);
        }
    }

    public int eval() {
        VectorSpecies<Short> vSPECIES = ShortVector.SPECIES_PREFERRED;
        int length = hiddenLayer.getBiases().length;
        ShortVector outputVector = ShortVector.fromArray(vSPECIES, hiddenLayer.getBiases(), 0);

        for (int i = 0; i < inputLayer.getActivations().length; i += vSPECIES.length()) {
            ShortVector activationVector = ShortVector.fromArray(vSPECIES, inputLayer.getActivations(), i);
            ShortVector weightVector = ShortVector.fromArray(vSPECIES, hiddenLayer.getWeights(), i);
            activationVector = activationVector.mul(weightVector);
            outputVector = outputVector.add(activationVector);
        }

        int output = outputVector.reduceLanes(VectorOperators.ADD) + hiddenLayer.getBiases()[0];
        return NNUE2SCORE * output / (SCALE * SCALE);
    }

    private short clippedRelu(short x) {
        return (short) Math.max(0, Math.min(SCALE, x));
    }

    @FunctionalInterface
    private interface ActivationUpdater {
        short update(short activation, short weight);
    }
}
