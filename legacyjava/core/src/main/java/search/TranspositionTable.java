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
}
