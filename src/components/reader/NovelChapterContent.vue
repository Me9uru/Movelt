<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from "vue";
import type { NovelChapterContent } from "../../domain/novel";

const props = defineProps<{
  document: NovelChapterContent;
  headingClass: string;
}>();

const emit = defineEmits<{
  ready: [element: HTMLElement];
  "layout-change": [];
  "preview-visible": [visible: boolean];
}>();

const content = ref<HTMLElement | null>(null);
const previewImageUrl = ref<string | null>(null);
const footnoteHtml = ref("");
const footnoteVisible = ref(false);
let contentResizeObserver: ResizeObserver | undefined;
const chapterFontStyle = document.createElement("style");

const hasLeadingHeading = computed(() => {
  const body = new DOMParser().parseFromString(
    props.document.html,
    "text/html",
  ).body;
  return /^H[1-6]$/.test(body.firstElementChild?.tagName ?? "");
});

const updateChapterFont = (fontUrl: string | null): void => {
  chapterFontStyle.textContent = fontUrl
    ? `@font-face { font-family: "movel-chapter"; font-display: block; src: url(${JSON.stringify(fontUrl)}); }`
    : "";
}

const prepareFootnotes = (element: HTMLElement): void => {
  element
    .querySelectorAll<HTMLAnchorElement>("a.duokan-footnote")
    .forEach((footnote) => {
      if (footnote.dataset.movelFootnoteReady) return;
      footnote.dataset.movelFootnoteReady = "true";

      const targetId = footnote.getAttribute("href")?.replace(/^#/, "");
      if (!targetId) return;
      const note = element.querySelector<HTMLElement>(
        `#${CSS.escape(targetId)}`,
      );
      if (!note) return;

      note.hidden = true;
      footnote.removeAttribute("href");
      footnote
        .querySelectorAll<HTMLImageElement>("img.footnote")
        .forEach((image) => {
          image.replaceWith(document.createTextNode("*"));
        });
      footnote.setAttribute("aria-label", "查看注释");
      footnote.setAttribute("role", "button");
      footnote.tabIndex = 0;
      footnote.addEventListener("keydown", (event) => {
        if (event.key !== "Enter" && event.key !== " ") return;
        event.preventDefault();
        event.stopPropagation();
        footnote.click();
      });
      footnote.title = note.textContent?.trim() || "查看注释";
      footnote.addEventListener("click", (event) => {
        event.preventDefault();
        event.stopPropagation();
        // 章节 HTML 已在 Rust 边界清洗。保留注释片段的标记，避免实体和排版在弹窗中被当作纯文本显示。
        footnoteHtml.value = note.innerHTML || "暂无注释内容";
        footnoteVisible.value = true;
      });
    });
}

const observeContent = (): void => {
  const element = content.value;
  if (!element) return;
  prepareFootnotes(element);
  contentResizeObserver?.disconnect();
  contentResizeObserver?.observe(element);
  element
    .querySelectorAll("img")
    .forEach((image) => contentResizeObserver?.observe(image));
  emit("ready", element);
  emit("layout-change");
}

const handleImageLoad = (event: Event): void => {
  if (event.target instanceof HTMLImageElement) emit("layout-change");
}

const handleImageClick = (event: MouseEvent): void => {
  const image = event.target;
  if (!(image instanceof HTMLImageElement)) return;
  if (!image.closest(".illus, .illu, .duokan-image-single, .image-preview"))
    return;
  event.stopPropagation();
  previewImageUrl.value = image.currentSrc || image.src;
  emit("preview-visible", true);
}

const updatePreviewVisible = (visible: boolean): void => {
  if (!visible) previewImageUrl.value = null;
  emit("preview-visible", visible);
}

const handleLinkClick = (event: MouseEvent): void => {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const link = target.closest<HTMLAnchorElement>("a[href]");
  if (!link) return;
  const href = link.getAttribute("href");
  if (!href) return;
  if (href.startsWith("#")) {
    event.preventDefault();
    document
      .getElementById(href.slice(1))
      ?.scrollIntoView({ block: "start", behavior: "smooth" });
    return;
  }
  try {
    const url = new URL(href, window.location.href);
    if (url.protocol !== "http:" && url.protocol !== "https:") return;
    event.preventDefault();
    window.open(url.href, "_blank", "noopener,noreferrer");
  } catch {
    event.preventDefault();
  }
}

watch(
  () => props.document.html,
  () => void nextTick(observeContent),
  { immediate: true },
);
watch(
  () => props.document.fontUrl,
  (fontUrl) => {
    updateChapterFont(fontUrl);
    void document.fonts.ready.then(() => emit("layout-change"));
  },
  { immediate: true },
);

onMounted(() => {
  document.head.append(chapterFontStyle);
  contentResizeObserver = new ResizeObserver(() => emit("layout-change"));
  void nextTick(observeContent);
});

onBeforeUnmount(() => {
  contentResizeObserver?.disconnect();
  chapterFontStyle.remove();
});
</script>

<template>
  <header v-if="!hasLeadingHeading" :class="headingClass">
    <h1>{{ document.title }}</h1>
  </header>
  <div
    ref="content"
    class="reader-content"
    v-html="document.html"
    @click="handleLinkClick"
    @click.capture="handleImageClick"
    @load.capture="handleImageLoad"
  />

  <var-image-preview
    :show="Boolean(previewImageUrl)"
    :images="previewImageUrl ? [previewImageUrl] : []"
    @update:show="updatePreviewVisible($event)"
  />

  <var-dialog
    :show="footnoteVisible"
    title="注释"
    :cancel-button="false"
    confirm-button-text="关闭"
    :dialog-class="
      document.fontUrl
        ? 'reader-footnote-dialog reader-footnote-dialog--chapter-font'
        : 'reader-footnote-dialog'
    "
    @update:show="footnoteVisible = $event"
    @confirm="footnoteVisible = false"
  >
    <div class="reader-footnote-content" v-html="footnoteHtml" />
  </var-dialog>
</template>
