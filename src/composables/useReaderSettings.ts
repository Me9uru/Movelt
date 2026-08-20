import { computed, reactive, watch } from "vue";

export type ReaderTheme = "paper" | "light" | "night";
export type ReaderFont = "serif" | "sans";
export type ReaderMode = "scroll" | "paged";

export interface ReaderSettings {
  fontSize: number;
  lineHeight: number;
  letterSpacing: number;
  paragraphSpacing: number;
  contentWidth: number;
  font: ReaderFont;
  theme: ReaderTheme;
  mode: ReaderMode;
}

const storageKey = "novel.reader.settings.v1";
const defaults: ReaderSettings = {
  fontSize: 18,
  lineHeight: 2.05,
  letterSpacing: 0,
  paragraphSpacing: 1.35,
  contentWidth: 780,
  font: "serif",
  theme: "light",
  mode: "scroll",
};

function loadSettings(): ReaderSettings {
  try {
    const saved = JSON.parse(localStorage.getItem(storageKey) ?? "{}") as Partial<ReaderSettings>;
    return { ...defaults, ...saved };
  } catch {
    return { ...defaults };
  }
}

const settings = reactive<ReaderSettings>(loadSettings());

watch(settings, (value) => localStorage.setItem(storageKey, JSON.stringify(value)), {
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

export function useReaderSettings() {
  return { settings, style, reset };
}
