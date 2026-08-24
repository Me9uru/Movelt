import { useReaderSettingsStore } from "../stores/readerSettings";
import type { ReaderKind } from "../stores/readerSettings";

export type {
  PageTurnDirection,
  ReaderConvert,
  ReaderFont,
  ReaderKind,
  ReaderMode,
  ReaderSettings,
  ReaderTheme,
} from "../stores/readerSettings";

export function useReaderSettings(kind: ReaderKind) {
  const store = useReaderSettingsStore();
  const isNovel = kind === "novel";
  return {
    settings: isNovel ? store.novelSettings : store.mangaSettings,
    style: isNovel ? store.novelStyle : store.mangaStyle,
    reset: () => store.reset(kind),
  };
}
