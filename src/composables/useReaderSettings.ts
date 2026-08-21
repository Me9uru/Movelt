import { computed, reactive, watch } from "vue";

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

const storageKeys: Record<ReaderKind, string> = {
  novel: "novel.reader.settings.v1",
  manga: "manga.reader.settings.v1",
};
const defaults: ReaderSettings = {
  fontSize: 18,
  lineHeight: 2.05,
  letterSpacing: 0,
  paragraphSpacing: 1.35,
  contentWidth: 780,
  font: "serif",
  theme: "light",
  mode: "scroll",
  pageTurnDirection: "left-previous",
  convert: "original",
};

function loadSettings(kind: ReaderKind): ReaderSettings {
  try {
    const saved = JSON.parse(localStorage.getItem(storageKeys[kind]) ?? "{}") as Partial<ReaderSettings>;
    return {
      ...defaults,
      ...saved,
      pageTurnDirection: saved.pageTurnDirection === "left-next"
        ? "left-next"
        : "left-previous",
      convert: saved.convert === "t2s" || saved.convert === "s2t"
        ? saved.convert
        : "original",
    };
  } catch {
    return { ...defaults };
  }
}

function createReaderSettings(kind: ReaderKind) {
  const settings = reactive<ReaderSettings>(loadSettings(kind));

  watch(settings, (value) => localStorage.setItem(storageKeys[kind], JSON.stringify(value)), {
    deep: true,
  });

  const style = computed(() => ({
    "--reader-font-size": `${settings.fontSize}px`,
    "--reader-line-height": String(settings.lineHeight),
    "--reader-letter-spacing": `${settings.letterSpacing}px`,
    "--reader-paragraph-spacing": `${settings.paragraphSpacing}em`,
    "--reader-width": `${settings.contentWidth}px`,
    "--reader-font-family": settings.font === "serif"
      ? '"Noto Serif CJK SC", "Noto Serif CJK TC", "Source Han Serif SC", "Source Han Serif TC", "Songti SC", "STSong", "SimSun", serif'
      : '"Noto Sans CJK SC", "Noto Sans CJK TC", "Source Han Sans CN", "Source Han Sans TC", "Microsoft YaHei", "PingFang SC", sans-serif',
  }));

  function reset() {
    Object.assign(settings, defaults);
  }

  return { settings, style, reset };
}

const settingsByKind = {
  novel: createReaderSettings("novel"),
  manga: createReaderSettings("manga"),
};

export function useReaderSettings(kind: ReaderKind) {
  return settingsByKind[kind];
}
