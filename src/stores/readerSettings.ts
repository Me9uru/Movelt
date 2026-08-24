import { computed, reactive, ref, watch } from "vue";
import { defineStore } from "pinia";

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

const themeStorageKey = "reader.theme.v1";
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

function isReaderTheme(value: unknown): value is ReaderTheme {
  return value === "paper" || value === "light" || value === "night";
}

function loadStoredSettings(kind: ReaderKind): Partial<ReaderSettings> {
  try {
    return JSON.parse(localStorage.getItem(storageKeys[kind]) ?? "{}") as Partial<ReaderSettings>;
  } catch {
    return {};
  }
}

function loadTheme(): ReaderTheme {
  const savedTheme = localStorage.getItem(themeStorageKey);
  if (isReaderTheme(savedTheme)) return savedTheme;

  // Migrate the existing per-reader theme on first launch after this change.
  const novelTheme = loadStoredSettings("novel").theme;
  if (isReaderTheme(novelTheme)) return novelTheme;
  const mangaTheme = loadStoredSettings("manga").theme;
  return isReaderTheme(mangaTheme) ? mangaTheme : defaults.theme;
}

function createSettings(kind: ReaderKind, theme: { value: ReaderTheme }): ReaderSettings {
  const saved = loadStoredSettings(kind);
  const settings = reactive<ReaderSettings>({
    ...defaults,
    ...saved,
    theme: defaults.theme,
    pageTurnDirection: saved.pageTurnDirection === "left-next"
      ? "left-next"
      : "left-previous",
    convert: saved.convert === "t2s" || saved.convert === "s2t"
      ? saved.convert
      : "original",
  });

  Object.defineProperty(settings, "theme", {
    enumerable: true,
    get: () => theme.value,
    set: (value: ReaderTheme) => {
      theme.value = value;
    },
  });

  watch(settings, ({ theme: _theme, ...value }) => {
    localStorage.setItem(storageKeys[kind], JSON.stringify(value));
  }, { deep: true });

  return settings;
}

function createReaderStyle(settings: ReaderSettings) {
  return computed(() => ({
    "--reader-font-size": `${settings.fontSize}px`,
    "--reader-line-height": String(settings.lineHeight),
    "--reader-letter-spacing": `${settings.letterSpacing}px`,
    "--reader-paragraph-spacing": `${settings.paragraphSpacing}em`,
    "--reader-width": `${settings.contentWidth}px`,
    "--reader-font-family": settings.font === "serif"
      ? '"Noto Serif CJK SC", "Noto Serif CJK TC", "Source Han Serif SC", "Source Han Serif TC", "Songti SC", "STSong", "SimSun", serif'
      : '"Noto Sans CJK SC", "Noto Sans CJK TC", "Source Han Sans CN", "Source Han Sans TC", "Microsoft YaHei", "PingFang SC", sans-serif',
  }));
}

export const useReaderSettingsStore = defineStore("reader-settings", () => {
  const theme = ref<ReaderTheme>(loadTheme());
  const novelSettings = createSettings("novel", theme);
  const mangaSettings = createSettings("manga", theme);
  const novelStyle = createReaderStyle(novelSettings);
  const mangaStyle = createReaderStyle(mangaSettings);

  watch(theme, (value) => {
    localStorage.setItem(themeStorageKey, value);
    document.documentElement.dataset.readerTheme = value;
  }, { immediate: true });

  function reset(kind: ReaderKind) {
    Object.assign(kind === "novel" ? novelSettings : mangaSettings, defaults);
  }

  return {
    theme,
    novelSettings,
    mangaSettings,
    novelStyle,
    mangaStyle,
    reset,
  };
});
