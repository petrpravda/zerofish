package search;

import org.javafish.move.Move;

import java.util.Arrays;

public class TranspositionTable {
    public record MyTtEntry(short move, short score, long hash, short searchDepth) {
        public Move getMove() {
            return new Move(move);
        }
    }

    public void put(long hash, Move move, short score, short currentSearchDepth) {
        this.table[index(hash)] = new MyTtEntry(move.bits(), score, hash, currentSearchDepth);
    }

    //    //  let count = upper_limit.next_power_of_two() / 2;
    private int count = 1048576; // 2 ** 20
    private MyTtEntry[] table = new MyTtEntry[count];
    private long bitmask = count - 1;

    private int index(long key) {
        return (int) (key & bitmask);
    }

    public MyTtEntry probe(long hash, short searchDepth) {
        MyTtEntry value = table[this.index(hash)];
        if (value != null && value.hash == hash && searchDepth <= value.searchDepth) {
            return value;
        } else {
            return null;
        }
    }

    public void reset() {
        Arrays.fill(this.table,null);
    }
}
