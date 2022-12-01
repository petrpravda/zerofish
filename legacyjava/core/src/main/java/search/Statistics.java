package search;

public class Statistics {
    public int leafs = 0;
    public int qleafs = 0;
    public int betaCutoffs = 0;
    public int qbetaCutoffs = 0;
    public int ttHits = 0;
    public int nodes = 0;
    public int qnodes = 0;

    public int totalNodes(){
        return nodes + qnodes;
    }

    public float branchingRatio(){
        if (nodes != leafs)
            return (float)nodes / (nodes - leafs);
        return 0;
    }

    public float qBranchingRatio(){
        if (qnodes != qleafs)
            return (float)qnodes / (qnodes - qleafs);
        return 0;
    }

    public void reset(){
        leafs = 0;
        qleafs = 0;
        betaCutoffs = 0;
        qbetaCutoffs = 0;
        ttHits = 0;
        nodes = 0;
        qnodes = 0;
    }
}
