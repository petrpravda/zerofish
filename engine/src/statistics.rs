pub struct Statistics {
    pub leafs: u32,
    pub qleafs: u32,
    pub betaCutoffs: u32,
    pub qbetaCutoffs: u32,
    pub ttHits: u32,
    pub nodes: u32,
    pub qnodes: u32,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            leafs: 0,
            qleafs: 0,
            betaCutoffs: 0,
            qbetaCutoffs: 0,
            ttHits: 0,
            nodes: 0,
            qnodes: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.leafs = 0;
        self.qleafs = 0;
        self.betaCutoffs = 0;
        self.qbetaCutoffs = 0;
        self.ttHits = 0;
        self.nodes = 0;
        self.qnodes = 0;
    }
}

// public class Statistics {
//     public int leafs = 0;
//     public int qleafs = 0;
//     public int betaCutoffs = 0;
//     public int qbetaCutoffs = 0;
//     public int ttHits = 0;
//     public int nodes = 0;
//     public int qnodes = 0;
//
//     public int totalNodes(){
//         return nodes + qnodes;
//     }
//
//     public float branchingRatio(){
//         if (nodes != leafs)
//             return (float)nodes / (nodes - leafs);
//         return 0;
//     }
//
//     public float qBranchingRatio(){
//         if (qnodes != qleafs)
//             return (float)qnodes / (qnodes - qleafs);
//         return 0;
//     }
//
//     public void reset(){
//         leafs = 0;
//         qleafs = 0;
//         betaCutoffs = 0;
//         qbetaCutoffs = 0;
//         ttHits = 0;
//         nodes = 0;
//         qnodes = 0;
//     }
// }
