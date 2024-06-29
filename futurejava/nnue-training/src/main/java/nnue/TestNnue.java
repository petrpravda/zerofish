package nnue;

public class TestNnue {
    public static void main(String[] args) {
        Network network = new Network();
        int result = network.eval();
        System.out.println(result);
    }
}
