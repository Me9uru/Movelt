import {
  computed,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
  type ComputedRef,
  type Ref,
} from "vue";

const DEFAULT_BATCH_SIZE = 6;
const LOAD_AHEAD_PX = 160;

interface ProgressiveRenderState {
  visibleCount: Ref<number>;
  setSentinel: (element: unknown) => void;
  hasMore: ComputedRef<boolean>;
}

/** Mounts list content in scroll-driven batches without changing API pagination. */
export const useProgressiveRender = (
  itemKeys: () => readonly string[],
  enabled: () => boolean = () => true,
  batchSize = DEFAULT_BATCH_SIZE,
): ProgressiveRenderState => {
  const visibleCount = ref(batchSize);
  const sentinel = ref<HTMLElement | null>(null);
  const hasMore = computed(
    () => enabled() && visibleCount.value < itemKeys().length,
  );
  let lastBatchScrollTop = 0;
  let scrollFrame: number | undefined;

  const reset = (): void => {
    visibleCount.value = batchSize;
    lastBatchScrollTop = window.scrollY;
  };

  const appendBatchIfNeeded = (): void => {
    scrollFrame = undefined;
    if (!hasMore.value || !sentinel.value) return;

    const scrollTop = window.scrollY;
    if (scrollTop < lastBatchScrollTop) {
      lastBatchScrollTop = scrollTop;
      return;
    }
    if (scrollTop === lastBatchScrollTop) return;
    if (
      sentinel.value.getBoundingClientRect().top >
      window.innerHeight + LOAD_AHEAD_PX
    ) {
      return;
    }

    visibleCount.value = Math.min(
      visibleCount.value + batchSize,
      itemKeys().length,
    );
    lastBatchScrollTop = scrollTop;
  };

  const handleScroll = (): void => {
    if (scrollFrame != null) return;
    scrollFrame = window.requestAnimationFrame(appendBatchIfNeeded);
  };

  const setSentinel = (element: unknown): void => {
    sentinel.value = element instanceof HTMLElement ? element : null;
  };

  watch(
    itemKeys,
    (nextKeys, previousKeys) => {
      const isAppend =
        nextKeys.length >= previousKeys.length &&
        previousKeys.every((key, index) => nextKeys[index] === key);
      if (!isAppend) reset();
    },
    { flush: "post" },
  );

  onMounted(() => {
    lastBatchScrollTop = window.scrollY;
    window.addEventListener("scroll", handleScroll, { passive: true });
  });

  onBeforeUnmount(() => {
    window.removeEventListener("scroll", handleScroll);
    if (scrollFrame != null) window.cancelAnimationFrame(scrollFrame);
  });

  return { visibleCount, setSentinel, hasMore };
};
