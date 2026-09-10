<script setup lang="ts">
import { useRoute, useRouter } from "vue-router";
import { useNovelReader } from "../../composables/reader/useNovelReader";
import ReaderToolbar from "../../components/reader/ReaderToolbar.vue";
import ReaderSettingsDrawer from "../../components/reader/ReaderSettingsDrawer.vue";
import ReaderBoundarySwitch from "../../components/reader/ReaderBoundarySwitch.vue";
import ReaderPageControls from "../../components/reader/ReaderPageControls.vue";
import NovelChapterContent from "../../components/reader/NovelChapterContent.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import ErrorState from "../../components/common/ErrorState.vue";

const route = useRoute();
const router = useRouter();
const {
  settings, style, bookId, readerDocument, loading, error, load, changeChapter,
  readerRoot, pageViewport, previewVisible, settingsVisible, currentPage, pageCount,
  isSpread, pageLabel, hasPreviousChapter, hasNextChapter, setReaderContent,
  updatePagination, handleReaderClick, handlePointerDown, handlePointerUp, cancelPointer, handlePageScroll,
  requestPreviousChapter, requestNextChapter, goToPage,
} = useNovelReader();
</script>

<template>
  <ReaderToolbar
    :title="readerDocument?.title || '小说阅读'"
    :settings-disabled="loading || !readerDocument"
    @back="router.push({ name: 'novel-detail', params: { bookId }, query: route.query })"
    @settings="settingsVisible = true"
  />
  <LoadingOverlay v-if="loading && !readerDocument" inline visible label="正在加载章节" />
  <ErrorState v-else-if="error" title="章节加载失败" :message="error" :loading="loading" @retry="load" />
  <article
    v-else-if="readerDocument"
    :ref="(element) => { readerRoot = element as HTMLElement | null; }"
    class="book-reader"
    :class="[
      `book-reader--${settings.mode}`,
      { 'book-reader--chapter-font': Boolean(readerDocument.fontUrl) },
    ]"
    :style="style"
    @click="handleReaderClick"
  >
    <ReaderBoundarySwitch
      v-if="settings.mode === 'scroll'"
      :has-previous="hasPreviousChapter"
      :has-next="hasNextChapter"
      :disabled="loading"
      @previous="requestPreviousChapter"
      @next="requestNextChapter"
    >
      <div class="reader-body">
        <NovelChapterContent
          :document="readerDocument"
          heading-class="reader-heading"
          @ready="setReaderContent"
          @layout-change="updatePagination(true)"
          @preview-visible="previewVisible = $event"
        />

        <var-divider>本章结束</var-divider>
      </div>
    </ReaderBoundarySwitch>

    <div
      v-else
      class="paged-reader"
      :class="{ 'paged-reader--spread': isSpread }"
    >
      <div
        :ref="(element) => { pageViewport = element as HTMLElement | null; }"
        class="page-viewport"
        tabindex="0"
        aria-label="分页阅读区域，可使用左右方向键翻页"
        @pointerdown="handlePointerDown"
        @scroll="handlePageScroll"
        @pointerup="handlePointerUp"
        @pointercancel="cancelPointer"
      >
        <NovelChapterContent
          :document="readerDocument"
          heading-class="paged-heading"
          @ready="setReaderContent"
          @layout-change="updatePagination(true)"
          @preview-visible="previewVisible = $event"
        />
        <p class="chapter-end">— 本章结束 —</p>
      </div>

      <ReaderPageControls
        :label="`${isSpread ? '双页' : '单页'} · ${pageLabel}`"
        :previous-disabled="loading || (currentPage === 0 && !hasPreviousChapter)"
        :next-disabled="loading || (currentPage >= pageCount - 1 && !hasNextChapter)"
        @previous="goToPage(currentPage - 1)"
        @next="goToPage(currentPage + 1)"
      />
    </div>

    <ReaderSettingsDrawer
      v-model="settingsVisible"
      kind="novel"
      :title="readerDocument.title"
      :previous-disabled="!hasPreviousChapter || loading"
      :next-disabled="!hasNextChapter || loading"
      @previous="changeChapter(-1)"
      @next="changeChapter(1)"
    />

  </article>
</template>
