<script setup lang="ts">
import { ref, useId } from "vue";
import { useDialogFocus } from "../../composables/useDialogFocus";
import { useReaderSettings } from "../../composables/reader/useReaderSettings";
import NovelAppearanceSettings from "./NovelAppearanceSettings.vue";
import type { ReaderKind } from "../../types/reader";

interface ReaderSettingsDrawerProps {
  kind: ReaderKind;
  title: string;
  previousDisabled?: boolean;
  nextDisabled?: boolean;
}

const props = defineProps<ReaderSettingsDrawerProps>();

const emit = defineEmits<{
  previous: [];
  next: [];
}>();

const visible = defineModel<boolean>({ required: true });
const { settings, theme, reset } = useReaderSettings(props.kind);
const panel = ref<HTMLElement | null>(null);
const titleId = useId();
const { rememberFocus, restoreFocus, trapFocus } = useDialogFocus(() => panel.value);

const focusPanel = (): void => {
  panel.value?.querySelector<HTMLElement>("[aria-label='关闭阅读设置']")?.focus();
};
</script>

<template>
  <var-popup
    v-model:show="visible"
    class="reader-settings-drawer"
    position="bottom"
    :z-index="2000"
    teleport="body"
    :default-style="false"
    :aria-labelledby="titleId"
    @open="rememberFocus"
    @opened="focusPanel"
    @closed="restoreFocus"
  >
    <div ref="panel" class="settings-panel" @keydown="trapFocus">
      <div class="settings-handle" aria-hidden="true"></div>
      <div class="settings-title">
        <h2 :id="titleId">阅读设置</h2>
        <var-button text round :elevation="false" aria-label="关闭阅读设置" @click="visible = false"><var-icon name="window-close" /></var-button>
      </div>
      <p class="reader-settings-chapter">{{ title }}</p>
      <nav class="reader-chapter-nav" aria-label="章节导航">
        <var-button type="info" tonal :elevation="false" :disabled="previousDisabled" @click="emit('previous')"><var-icon name="arrow-left" aria-hidden="true" />上一章</var-button>
        <var-button type="info" tonal :elevation="false" :disabled="nextDisabled" @click="emit('next')">下一章<var-icon name="arrow-right" aria-hidden="true" /></var-button>
      </nav>
      <div class="reader-settings-reset">
        <var-button text :elevation="false" @click="reset">
          恢复默认
          <var-icon name="refresh" aria-hidden="true" />
        </var-button>
      </div>

      <div class="reader-setting-label">阅读模式</div>
      <var-radio-group v-model="settings.mode" aria-label="阅读模式">
        <var-radio checked-value="scroll">滚动阅读</var-radio>
        <var-radio checked-value="paged">分页阅读</var-radio>
      </var-radio-group>

      <template v-if="settings.mode === 'paged'">
        <div class="reader-setting-label">翻页方向</div>
        <var-radio-group v-model="settings.pageTurnDirection" aria-label="翻页方向">
          <var-radio checked-value="left-previous">从左至右</var-radio>
          <var-radio checked-value="left-next">从右至左</var-radio>
        </var-radio-group>
      </template>

      <div class="reader-setting-label">背景主题</div>
      <var-radio-group v-model="theme" aria-label="背景主题">
        <var-radio checked-value="paper">纸张</var-radio>
        <var-radio checked-value="light">明亮</var-radio>
        <var-radio checked-value="night">夜间</var-radio>
      </var-radio-group>

      <NovelAppearanceSettings v-if="kind === 'novel'" />
    </div>
  </var-popup>
</template>
