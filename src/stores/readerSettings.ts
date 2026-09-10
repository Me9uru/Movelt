import { computed, reactive, ref, watch } from "vue";
import { StyleProvider } from "@varlet/ui";
import { defineStore } from "pinia";
import { paperTheme } from "../themes/paper";
import { lightTheme, nightTheme } from "../themes/app";
import type { ComicReaderSettings, ReaderKind, NovelReaderSettings, ReaderTheme } from "../types/reader";

const themeStorageKey = "reader.theme.v1";
const storageKeys: Record<ReaderKind, string> = {
  novel: "novel.reader.settings.v1",
  comic: "comic.reader.settings.v1",
};
const comicDefaults: ComicReaderSettings = { mode: "scroll", pageTurnDirection: "left-previous" };
const novelDefaults: NovelReaderSettings = {
  ...comicDefaults, fontSize: 18, lineHeight: 2.05, letterSpacing: 0,
  paragraphSpacing: 1.35, contentWidth: 780, font: "serif", convert: "original",
};

const isReaderTheme = (value: unknown): value is ReaderTheme =>
  value === "paper" || value === "light" || value === "night";

const loadStoredSettings = (kind: ReaderKind): Record<string, unknown> => {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(storageKeys[kind]) ?? "{}");
    return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, unknown> : {};
  } catch {
    return {};
  }
};

const loadTheme = (): ReaderTheme => {
  try {
    const theme = localStorage.getItem(themeStorageKey);
    if (isReaderTheme(theme)) return theme;
  } catch { /* Use the migrated or default theme when storage is unavailable. */ }
  for (const kind of ["novel", "comic"] as const) {
    const theme = loadStoredSettings(kind).theme;
    if (isReaderTheme(theme)) return theme;
  }
  return "light";
};

const loadCommonSettings = (value: Record<string, unknown>): ComicReaderSettings => ({
  mode: value.mode === "paged" ? "paged" : "scroll",
  pageTurnDirection: value.pageTurnDirection === "left-next" ? "left-next" : "left-previous",
});

const numberSetting = (value: unknown, fallback: number, min: number, max: number): number =>
  typeof value === "number" && Number.isFinite(value) ? Math.min(max, Math.max(min, value)) : fallback;

const persist = (key: string, value: string): void => {
  try { localStorage.setItem(key, value); } catch { /* Settings remain usable in memory. */ }
};

const createReaderStyle = (settings: NovelReaderSettings) => {
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
};


export const useReaderSettingsStore = defineStore("reader-settings", () => {
  const theme = ref<ReaderTheme>(loadTheme());
  const saved = loadStoredSettings("novel");
  const novelSettings = reactive<NovelReaderSettings>({
    ...loadCommonSettings(saved),
    fontSize: numberSetting(saved.fontSize, novelDefaults.fontSize, 14, 30),
    lineHeight: numberSetting(saved.lineHeight, novelDefaults.lineHeight, 1.4, 2.6),
    letterSpacing: numberSetting(saved.letterSpacing, novelDefaults.letterSpacing, 0, 4),
    paragraphSpacing: numberSetting(saved.paragraphSpacing, novelDefaults.paragraphSpacing, 0.6, 2.4),
    contentWidth: numberSetting(saved.contentWidth, novelDefaults.contentWidth, 560, 1100),
    font: saved.font === "sans" ? "sans" : "serif",
    convert: saved.convert === "t2s" || saved.convert === "s2t" ? saved.convert : "original",
  });
  const comicSettings = reactive(loadCommonSettings(loadStoredSettings("comic")));
  const novelStyle = createReaderStyle(novelSettings);
  watch(novelSettings, (value) => persist(storageKeys.novel, JSON.stringify(value)), { deep: true });
  watch(comicSettings, (value) => persist(storageKeys.comic, JSON.stringify(value)), { deep: true });

  watch(theme, (value) => {
    persist(themeStorageKey, value);
    document.documentElement.dataset.readerTheme = value;
    StyleProvider(value === "paper" ? paperTheme : value === "night" ? nightTheme : lightTheme);
  }, { immediate: true });

  const reset = (kind: ReaderKind): void => {
    if (kind === "novel") Object.assign(novelSettings, novelDefaults);
    else Object.assign(comicSettings, comicDefaults);
  };

  return { theme, novelSettings, comicSettings, novelStyle, reset };
});
