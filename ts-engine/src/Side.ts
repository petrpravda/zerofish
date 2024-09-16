export const Side = {
  WHITE: 0,
  BLACK: 1,
} as const;

export namespace SideUtils {
  export function flip(side: number): number {
    return Side.BLACK ^ side;
  }

  export function multiplicator(side: number): number {
    return side === Side.WHITE ? 1 : -1;
  }
}
