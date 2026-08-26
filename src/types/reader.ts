export type ReaderTheme = "paper" | "light" | "night";
export type ReaderFont = "serif" | "sans";
export type ReaderMode = "scroll" | "paged";
export type PageTurnDirection = "left-previous" | "left-next";
export type ReaderConvert = "original" | "t2s" | "s2t";
export type ReaderKind = "novel" | "manga";

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

/** Props for the shared novel and manga reader settings/navigation drawer. */
export interface ReaderSettingsDrawerProps {
  kind: ReaderKind;
  title: string;
  titleClick?: () => void;
  previousDisabled?: boolean;
  nextDisabled?: boolean;
}
