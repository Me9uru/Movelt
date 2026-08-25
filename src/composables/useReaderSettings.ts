import { computed } from "vue";
import { useReaderSettingsStore } from "../stores/readerSettings";
import type { ReaderKind } from "../types/reader";

export type {
  PageTurnDirection,
  ReaderConvert,
  ReaderFont,
  ReaderKind,
  ReaderMode,
  ReaderSettings,
  ReaderTheme,
} from "../types/reader";

export function useReaderSettings(kind: ReaderKind) {
  const store = useReaderSettingsStore();
  const isNovel = kind === "novel";
  return {
    settings: isNovel ? store.novelSettings : store.mangaSettings,
    style: computed(() => (isNovel ? store.novelStyle : store.mangaStyle)),
    reset: () => store.reset(kind),
  };
}
