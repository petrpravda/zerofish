package nnue;

import jdk.incubator.vector.ShortVector;
import jdk.incubator.vector.VectorSpecies;

public class TestNnue2 {
    public static void main(String[] args) {
        short[] biases = new short[16];
        //short[] biases = { 1, 2, 3, 4, 5, 6, 7, 8 };
        int biasesLength = biases.length;
        VectorSpecies<Short> vSPECIES = ShortVector.SPECIES_PREFERRED;
        ShortVector outputVector = ShortVector.fromArray(vSPECIES, biases, 0);
    }
}
