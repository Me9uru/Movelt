import { computed, reactive, ref, watch } from "vue";
import { StyleProvider, Themes } from "@varlet/ui";
import { defineStore } from "pinia";
import { paperTheme } from "../themes/paper";
import type { ReaderKind, ReaderSettings, ReaderTheme } from "../types/reader";

const themeStorageKey = "reader.theme.v1";
const storageKeys: Record<ReaderKind, string> = {
  novel: "novel.reader.settings.v1",
  comic: "comic.reader.settings.v1",
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

const isReaderTheme = (value: unknown): value is ReaderTheme => {
  return value === "paper" || value === "light" || value === "night";
}

const loadStoredSettings = (kind: ReaderKind): Partial<ReaderSettings> => {
  try {
    return JSON.parse(localStorage.getItem(storageKeys[kind]) ?? "{}") as Partial<ReaderSettings>;
  } catch {
    return {};
  }
}

const loadTheme = (): ReaderTheme => {
  const savedTheme = localStorage.getItem(themeStorageKey);
  if (isReaderTheme(savedTheme)) return savedTheme;

  // Migrate the existing per-reader theme on first launch after this change.
  const novelTheme = loadStoredSettings("novel").theme;
  if (isReaderTheme(novelTheme)) return novelTheme;
  const comicTheme = loadStoredSettings("comic").theme;
  return isReaderTheme(comicTheme) ? comicTheme : defaults.theme;
}

const createSettings = (kind: ReaderKind, theme: { value: ReaderTheme }): ReaderSettings => {
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

const createReaderStyle = (settings: ReaderSettings) => {
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

const resolveVarletTheme = (value: ReaderTheme) => {
  if (value === "paper") return paperTheme;
  return value === "night" ? Themes.md3Dark : Themes.md3Light;
}

export const useReaderSettingsStore = defineStore("reader-settings", () => {
  const theme = ref<ReaderTheme>(loadTheme());
  const novelSettings = createSettings("novel", theme);
  const comicSettings = createSettings("comic", theme);
  const novelStyle = createReaderStyle(novelSettings);
  const comicStyle = createReaderStyle(comicSettings);

  watch(theme, (value) => {
    localStorage.setItem(themeStorageKey, value);
    document.documentElement.dataset.readerTheme = value;
    StyleProvider(resolveVarletTheme(value));
  }, { immediate: true });

  const reset = (kind: ReaderKind) => {
    Object.assign(kind === "novel" ? novelSettings : comicSettings, defaults);
  }

  return {
    theme,
    novelSettings,
    comicSettings,
    novelStyle,
    comicStyle,
    reset,
  };
});
