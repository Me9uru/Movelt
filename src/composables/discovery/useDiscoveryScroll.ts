import { nextTick, onMounted } from "vue";
import { onBeforeRouteLeave } from "vue-router";

interface ScrollCache {
  scrollTop: number;
}

/** Retains the document position even when navigation has no saved history entry. */
export const useDiscoveryScroll = (
  cache: ScrollCache,
  enabled: () => boolean,
): void => {
  onBeforeRouteLeave(() => {
    if (enabled()) cache.scrollTop = window.scrollY;
  });

  onMounted(() => {
    if (!enabled() || cache.scrollTop <= 0) return;
    void nextTick(() => {
      requestAnimationFrame(() => window.scrollTo({ top: cache.scrollTop }));
    });
  });
};
