export type ReaderTheme = "paper" | "light" | "night";
export type ReaderFont = "serif" | "sans";
export type ReaderMode = "scroll" | "paged";
export type PageTurnDirection = "left-previous" | "left-next";
export type ReaderConvert = "original" | "t2s" | "s2t";
export type ReaderKind = "novel" | "comic";

export interface ReaderSettings {
  fontSize: number;
  lineHeight: number;
  letterSpacing: number;
  paragraphSpacing: number;
  contentWidth: number;
  font: ReaderFont;
  theme: ReaderTheme;
  mode: ReaderMode;
  pageTurnDirection: PageTurnDirection;
  convert: ReaderConvert;
}
