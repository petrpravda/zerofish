package search;

import org.javafish.move.Move;

import java.util.HashMap;

public class TranspositionTable {
    private final HashMap<Long, TTEntry> table = new HashMap<>();

    public void set(long key, int score, int depth, int flag, Move bestMove){
        table.put(key, new TTEntry(key, score, depth, flag, bestMove));
    }

    public TTEntry probe(long key){
        return table.get(key);
    }

    public void reset(){
        table.clear();
    }


//    //  let count = upper_limit.next_power_of_two() / 2;
//    private int count = 1048576; // 2 ** 20
//    private TTEntry[] table = new TTEntry[count];
//    private long bitmask = count - 1;
//
//    public void set(long key, int score, int depth, int flag, Move bestMove){
//        table[this.index(key)] = new TTEntry(key, score, depth, flag, bestMove);
//    }
//
//    public TTEntry probe(long key){
//        TTEntry value = table[this.index(key)];
//        if (value != null && value.key == key) {
//            return value;
//        } else {
//            return null;
//        }
//    }
//
//    public void reset(){
//        //table = new TTEntry[count];
//        for (int i = 0; i < count; i++) {
//            table[i] = null;
//        }
//    }
//
//    private int index(long key) {
//        return (int) (key & bitmask);
//    }



//    public static void main(String[] args) {
//        TranspTable tt = new TranspTable();
//        long start = System.currentTimeMillis();
//        for (int i = 0; i < 1000; i++) {
//            tt.reset();
//        }
//        long end = System.currentTimeMillis();
//        System.out.println(end - start);
//    }
}
