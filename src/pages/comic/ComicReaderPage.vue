<script setup lang="ts">
import { useComicReader } from "../../composables/reader/useComicReader";
import ReaderToolbar from "../../components/reader/ReaderToolbar.vue";
import ReaderSettingsDrawer from "../../components/reader/ReaderSettingsDrawer.vue";
import ReaderBoundarySwitch from "../../components/reader/ReaderBoundarySwitch.vue";
import ReaderPageControls from "../../components/reader/ReaderPageControls.vue";
import ComicPageImage from "../../components/reader/ComicPageImage.vue";
import LoadingOverlay from "../../components/common/LoadingOverlay.vue";
import ErrorState from "../../components/common/ErrorState.vue";

const {
  settings, comic, pages, loading, error, settingsVisible, pageList, currentPage,
  hasPreviousChapter, hasNextChapter, load, returnToDetail, handleReaderClick,
  chapterOffset, changePage, handleImageLoad,
} = useComicReader();
</script>

<template>
  <ReaderToolbar
    :title="comic?.title || '漫画阅读'"
    :settings-disabled="loading"
    @back="returnToDetail"
    @settings="settingsVisible = true"
  />
  <article
    class="book-reader comic-reader"
    :class="[`book-reader--${settings.mode}`]"
    @click="handleReaderClick"
  >
    <LoadingOverlay
      v-if="loading"
      visible
      label="正在加载漫画章节"
    />
    <ErrorState
      v-if="error"
      title="漫画页面加载失败"
      :message="error"
      :loading="loading"
      @retry="load"
    />
    <ReaderBoundarySwitch
      v-else
      :has-previous="hasPreviousChapter"
      :has-next="hasNextChapter"
      :disabled="loading"
      @previous="chapterOffset(-1)"
      @next="chapterOffset(1)"
    >
      <section :ref="(element) => { pageList = element as HTMLElement | null; }" class="comic-reader-pages">
        <p v-if="!loading && pages.length === 0" class="comic-reader-pending">
          正在获取章节页码…
        </p>
        <template v-if="settings.mode === 'scroll'">
          <ComicPageImage
            v-for="(page, index) in pages"
            :key="index"
            :url="page"
            :index="index"
            @load="handleImageLoad"
          />
        </template>
        <ComicPageImage
          v-else-if="pages.length"
          :key="currentPage"
          :url="pages[currentPage] ?? ''"
          :index="currentPage"
          @load="handleImageLoad"
        />
      </section>
    </ReaderBoundarySwitch>
    <ReaderPageControls
      v-if="settings.mode === 'paged' && pages.length"
      :label="`${currentPage + 1} / ${pages.length}`"
      :previous-disabled="loading || (currentPage === 0 && !hasPreviousChapter)"
      :next-disabled="loading || (currentPage === pages.length - 1 && !hasNextChapter)"
      @previous="changePage(-1)"
      @next="changePage(1)"
    />
    <ReaderSettingsDrawer
      v-model="settingsVisible"
      kind="comic"
      :title="comic?.title || '漫画阅读'"
      :previous-disabled="loading || !hasPreviousChapter"
      :next-disabled="loading || !hasNextChapter"
      @previous="chapterOffset(-1)"
      @next="chapterOffset(1)"
    />
  </article>
</template>
