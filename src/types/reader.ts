export type ReaderTheme = "paper" | "light" | "night";
export type ReaderFont = "serif" | "sans";
export type ReaderMode = "scroll" | "paged";
export type PageTurnDirection = "left-previous" | "left-next";
export type ReaderConvert = "original" | "t2s" | "s2t";
export type ReaderKind = "novel" | "comic";

export interface ReaderSettings {
  mode: ReaderMode;
  pageTurnDirection: PageTurnDirection;
}

export type ComicReaderSettings = ReaderSettings;

export interface NovelReaderSettings extends ReaderSettings {
  fontSize: number;
  lineHeight: number;
  letterSpacing: number;
  paragraphSpacing: number;
  contentWidth: number;
  font: ReaderFont;
  convert: ReaderConvert;
}
