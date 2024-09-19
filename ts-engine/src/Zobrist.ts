export class Zobrist {
  public static ZOBRIST_TABLE: number[][] =
    Array.from({ length: 14 }, () => Array.from({ length: 64 }, () => Math.floor(Math.random() * Number.MAX_SAFE_INTEGER)));
  public static EN_PASSANT: number[] = Array.from({ length: 8 }, () => Math.floor(Math.random() * Number.MAX_SAFE_INTEGER));
  public static SIDE: number = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
}
