pub struct Statistics {
    pub leafs: u32,
    pub qleafs: u32,
    pub beta_cutoffs: u32,
    pub qbeta_cutoffs: u32,
    pub tt_hits: u32,
    pub nodes: u32,
    // pub qnodes: u32,
}

impl Statistics {
    pub fn total_nodes(&self) -> u32 {
        self.leafs + self.qleafs
    }
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            leafs: 0,
            qleafs: 0,
            beta_cutoffs: 0,
            qbeta_cutoffs: 0,
            tt_hits: 0,
            nodes: 0,
//            qnodes: 0,
        }
    }

    pub fn reset(&mut self) {
        self.leafs = 0;
        self.qleafs = 0;
        self.beta_cutoffs = 0;
        self.qbeta_cutoffs = 0;
        self.tt_hits = 0;
        self.nodes = 0;
        // self.qnodes = 0;
    }

    pub fn increment_leafs(&mut self) {
        self.leafs += 1;
    }

    pub fn increment_nodes(&mut self) {
        self.nodes += 1;
    }

    pub fn increment_qnodes(&mut self) {
        self.nodes += 1;
    }

    pub fn increment_qleafs(&mut self) {
        self.qleafs += 1;
    }

    pub fn increment_qbeta_cutoffs(&mut self) {
        self.qbeta_cutoffs += 1;
    }

    pub fn increase_tthits(&mut self) {
        self.tt_hits += 1;
    }

    pub fn increase_beta_cutoffs(&mut self) {
        self.beta_cutoffs += 1;
    }
}
