import { computed, toRef, type ComputedRef, type Ref } from "vue";
import { useReaderSettingsStore } from "../../stores/readerSettings";
import type { ComicReaderSettings, ReaderKind, NovelReaderSettings, ReaderTheme } from "../../types/reader";

export type {
  PageTurnDirection,
  ReaderConvert,
  ReaderFont,
  ReaderKind,
  ReaderMode,
  NovelReaderSettings,
  ReaderTheme,
} from "../../types/reader";

interface SettingsResult<T> {
  settings: T;
  theme: Ref<ReaderTheme>;
  reset: () => void;
}

export function useReaderSettings(kind: "novel"): SettingsResult<NovelReaderSettings> & { style: ComputedRef<Record<string, string>>; };
export function useReaderSettings(kind: "comic"): SettingsResult<ComicReaderSettings>;
export function useReaderSettings(kind: ReaderKind): SettingsResult<ComicReaderSettings>;
export function useReaderSettings(kind: ReaderKind): SettingsResult<ComicReaderSettings> & { style?: ComputedRef<Record<string, string>>; } {
  const store = useReaderSettingsStore();
  const isNovel = kind === "novel";
  return {
    settings: isNovel ? store.novelSettings : store.comicSettings,
    theme: toRef(store, "theme"),
    ...(isNovel ? { style: computed(() => store.novelStyle) } : {}),
    reset: () => store.reset(kind),
  };
}
