import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { NovelChapterContent } from "../../domain/novel";
import type { ReadPosition } from "../../domain/reader";
import type { NovelReaderSettings } from "../../types/reader";
import { getReaderDocument, getReaderOverview } from "../../services/novel";
import { createLatestRequest } from "../../utils/latestRequest";
import { getErrorMessage } from "../../utils/error";

export const useNovelChapter = (settings: NovelReaderSettings, beforeChange: () => void) => {
  const route = useRoute();
  const router = useRouter();
  const bookId = computed(() => typeof route.params.bookId === "string" ? route.params.bookId : "");
  const chapterId = computed(() => typeof route.params.chapterId === "string" ? route.params.chapterId : "");
  const readerDocument = ref<NovelChapterContent | null>(null);
  const resumePosition = ref<ReadPosition | null>(null);
  const chapterIds = ref<string[]>([]);
  const chapterEntry = ref<"default" | "next" | "previous">("default");
  const loading = ref(true);
  const error = ref("");
  const loadedBookId = ref("");
  const chapterIndex = computed(() => chapterIds.value.indexOf(chapterId.value));
  const hasPreviousChapter = computed(() => chapterIndex.value > 0);
  const hasNextChapter = computed(() => chapterIndex.value >= 0 && chapterIndex.value < chapterIds.value.length - 1);
  const requests = createLatestRequest();
  let requestedChapter: string | null = null;

  const load = async (): Promise<void> => {
    const isCurrent = requests.start();
    const id = bookId.value;
    const chapter = chapterId.value;
    const convert = settings.convert;
    const localPosition = loadedBookId.value === id ? resumePosition.value : null;
    loading.value = true;
    error.value = "";
    readerDocument.value = null;
    try {
      if (!id || !chapter) throw new Error("缺少书籍或章节编号");
      const [overview, content] = await Promise.all([
        getReaderOverview(id),
        getReaderDocument(id, chapter, convert),
      ]);
      if (!isCurrent()) return;
      loadedBookId.value = id;
      chapterIds.value = overview.chapters.map((item) => item.id);
      resumePosition.value = localPosition?.chapterId === chapter ? localPosition : overview.readPosition;
      readerDocument.value = content;
    } catch (value) {
      if (isCurrent()) error.value = getErrorMessage(value, "无法加载章节");
    } finally {
      if (isCurrent()) loading.value = false;
    }
  };

  const changeChapter = (offset: number): void => {
    if (loading.value) return;
    const nextId = chapterIds.value[chapterIndex.value + offset];
    if (!nextId) return;
    beforeChange();
    requestedChapter = nextId;
    chapterEntry.value = offset > 0 ? "next" : "previous";
    void router.replace({
      name: "novel-reader",
      params: { bookId: bookId.value, chapterId: nextId },
      query: route.query,
    });
  };

  onMounted(() => void load());
  watch([bookId, chapterId, () => settings.convert], ([book, chapter], [previousBook]) => {
    beforeChange();
    if (book !== previousBook || chapter !== requestedChapter) chapterEntry.value = "default";
    requestedChapter = null;
    void load();
  });
  onBeforeUnmount(requests.invalidate);

  return {
    bookId, chapterId, loadedBookId, readerDocument, resumePosition, chapterEntry,
    loading, error, hasPreviousChapter, hasNextChapter, load, changeChapter,
  };
};
