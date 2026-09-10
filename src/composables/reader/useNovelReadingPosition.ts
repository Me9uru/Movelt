import { ref, type Ref } from "vue";
import type { NovelChapterContent } from "../../domain/novel";
import type { ReadPosition } from "../../domain/reader";
import type { NovelReaderSettings } from "../../types/reader";
import { saveReadPosition } from "../../services/novel";
import { createReaderProgressSaver } from "../../utils/readerProgress";
import { showError } from "../../utils/error";

interface PositionOptions {
  settings: NovelReaderSettings;
  readerDocument: Ref<NovelChapterContent | null>;
  loadedBookId: Ref<string>;
  resumePosition: Ref<ReadPosition | null>;
  chapterEntry: Ref<"default" | "next" | "previous">;
  readerContent: Ref<HTMLElement | null>;
  pageViewport: Ref<HTMLElement | null>;
  currentPage: Ref<number>;
  pageCount: Ref<number>;
  pageLocation: Ref<number>;
  pageStep: () => number;
}

export const useNovelReadingPosition = ({
  settings, readerDocument, loadedBookId, resumePosition, chapterEntry,
  readerContent, pageViewport, currentPage, pageCount, pageLocation, pageStep,
}: PositionOptions) => {
  const hasRestoredServerPosition = ref(false);
  const saveProgress = createReaderProgressSaver("novel", (value) => showError(value, "保存阅读进度失败"));

  const visibleXPath = (): string => {
    const root = readerContent.value;
    if (!root) return "//*";
    const nodes = [
      ...root.querySelectorAll<HTMLElement>("p, img, li, h1, h2, h3, blockquote"),
    ];
    const viewport = pageViewport.value;
    const target =
      settings.mode === "paged" && viewport
        ? (() => {
          const bounds = viewport.getBoundingClientRect();
          // CSS 多栏分页下，每段都处于相同的纵向坐标；要按横向视口选择当前页的首个元素。
          return nodes.find((node) => {
            const rect = node.getBoundingClientRect();
            return rect.right > bounds.left && rect.left < bounds.right;
          });
        })()
        : nodes.find((node) => node.getBoundingClientRect().bottom >= 0);
    if (!target) return "//*";
    const path: string[] = [];
    let current: Element | null = target;
    while (current && current !== root) {
      const siblings = [...(current.parentElement?.children ?? [])].filter(
        (item) => item.tagName === current!.tagName,
      );
      path.unshift(
        `${current.tagName.toLowerCase()}[${siblings.indexOf(current) + 1}]`,
      );
      current = current.parentElement;
    }
    return `//*${path.length ? `/${path.join("/")}` : ""}`;
  };

  const restoreServerPosition = () => {
    if (hasRestoredServerPosition.value || chapterEntry.value !== "default") return;
    const position =
      resumePosition.value?.chapterId === readerDocument.value?.chapterId
        ? resumePosition.value
        : readerDocument.value?.readPosition?.chapterId ===
          readerDocument.value?.serverChapterId
          ? readerDocument.value?.readPosition ?? null
          : null;
    const root = readerContent.value;
    if (!position || !root?.isConnected || !position.position) return;
    try {
      const target = document.evaluate(
        position.position.startsWith("//*") ? `.${position.position.slice(3) || ""}` : position.position,
        root,
        null,
        XPathResult.FIRST_ORDERED_NODE_TYPE,
        null,
      ).singleNodeValue;
      if (!(target instanceof HTMLElement) || !root.contains(target)) return;
      target.scrollIntoView({ block: "start", behavior: "auto" });
      if (settings.mode === "paged" && pageViewport.value) {
        currentPage.value = Math.round(
          pageViewport.value.scrollLeft / pageStep(),
        );
        pageLocation.value =
          pageCount.value > 1 ? currentPage.value / (pageCount.value - 1) : 0;
      }
      hasRestoredServerPosition.value = true;
    } catch {
      // Old or malformed server positions must not block chapter rendering.
    }
  };


  const recordProgress = (): void => {
    const content = readerDocument.value;
    if (!content || !loadedBookId.value || !readerContent.value?.isConnected) return;
    const xpath = visibleXPath();
    resumePosition.value = { chapterId: content.chapterId, position: xpath };
    const book = loadedBookId.value;
    saveProgress(book, () => saveReadPosition(book, content.serverChapterId, xpath));
  };
  return { recordProgress, restoreServerPosition, hasRestoredServerPosition };
};
