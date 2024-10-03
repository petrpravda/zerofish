export class Statistics {
  public leafs: number = 0;
  public qleafs: number = 0;
  public betaCutoffs: number = 0;
  public qbetaCutoffs: number = 0;
  public ttHits: number = 0;
  public nodes: number = 0;
  public qnodes: number = 0;

  public totalNodes(): number {
    return this.nodes + this.qnodes;
  }

  public branchingRatio(): number {
    if (this.nodes !== this.leafs) {
      return this.nodes / (this.nodes - this.leafs);
    }
    return 0;
  }

  public qBranchingRatio(): number {
    if (this.qnodes !== this.qleafs) {
      return this.qnodes / (this.qnodes - this.qleafs);
    }
    return 0;
  }

  public reset(): void {
    this.leafs = 0;
    this.qleafs = 0;
    this.betaCutoffs = 0;
    this.qbetaCutoffs = 0;
    this.ttHits = 0;
    this.nodes = 0;
    this.qnodes = 0;
  }
}
