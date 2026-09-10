import type { PageTurnDirection } from "../types/reader";

export const pageOffsetForSide = (direction: PageTurnDirection, side: "left" | "right"): number => {
  const left = direction === "left-next" ? 1 : -1;
  return side === "left" ? left : -left;
};
