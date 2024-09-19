export const Side = {
  WHITE: 0,
  BLACK: 1,
} as const;

export type SideType = (typeof Side)[keyof typeof Side];

export namespace SideUtils {
  export function flip(side: SideType): SideType {
    return side === Side.BLACK ? Side.WHITE : Side.BLACK;
  }

  export function multiplicator(side: number): number {
    return side === Side.WHITE ? 1 : -1;
  }
}
