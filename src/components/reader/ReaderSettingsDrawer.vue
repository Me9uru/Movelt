<script setup lang="ts">
import { ref, useId, watchPostEffect } from "vue";
import { useReaderSettings } from "../../composables/reader/useReaderSettings";
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
const { settings, reset } = useReaderSettings(props.kind);
const panel = ref<HTMLElement | null>(null);
const titleId = useId();
let opener: HTMLElement | null = null;

const rememberFocus = (): void => {
  opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
};
const focusPanel = (): void => {
  panel.value?.querySelector<HTMLElement>("[aria-label='关闭阅读设置']")?.focus();
  syncSliderAccessibility();
};
// Varlet 3 reports thumb percentages as aria-valuenow; expose the setting's
// actual units so keyboard and screen-reader users receive the same value.
const syncSliderAccessibility = (): void => {
  const values = [
    [settings.fontSize, `${settings.fontSize} 像素`],
    [settings.lineHeight, `${settings.lineHeight.toFixed(1)} 倍`],
    [settings.letterSpacing, `${settings.letterSpacing.toFixed(1)} 像素`],
    [settings.paragraphSpacing, `${settings.paragraphSpacing.toFixed(1)} 倍`],
    [settings.contentWidth, `${settings.contentWidth} 像素`],
  ] as const;
  panel.value?.querySelectorAll<HTMLElement>(".reader-setting-range").forEach((range, index) => {
    const slider = range.querySelector("[role='slider']");
    const value = values[index];
    if (!slider || !value) return;
    slider.setAttribute("aria-label", range.dataset.label ?? "");
    slider.setAttribute("aria-valuenow", String(value[0]));
    slider.setAttribute("aria-valuetext", value[1]);
  });
};
watchPostEffect(syncSliderAccessibility);

const restoreFocus = (): void => {
  if (opener?.isConnected) opener.focus({ preventScroll: true });
};
const trapFocus = (event: KeyboardEvent): void => {
  if (event.key !== "Tab") return;
  const controls = Array.from(panel.value?.querySelectorAll<HTMLElement>(
    "button:not(:disabled), [tabindex='0'], input:not(:disabled)",
  ) ?? []).filter((element) => element.getClientRects().length > 0);
  const first = controls[0];
  const last = controls[controls.length - 1];
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last?.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first?.focus();
  }
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
      <var-radio-group v-model="settings.theme" aria-label="背景主题">
        <var-radio checked-value="paper">纸张</var-radio>
        <var-radio checked-value="light">明亮</var-radio>
        <var-radio checked-value="night">夜间</var-radio>
      </var-radio-group>

      <template v-if="kind === 'novel'">
        <div class="reader-setting-label">正文字体</div>
        <var-radio-group v-model="settings.font" aria-label="正文字体">
          <var-radio checked-value="serif">衬线</var-radio>
          <var-radio checked-value="sans">无衬线</var-radio>
        </var-radio-group>

        <div class="reader-setting-label">文字转换</div>
        <var-radio-group v-model="settings.convert" aria-label="文字转换">
          <var-radio checked-value="original">原文</var-radio>
          <var-radio checked-value="t2s">繁转简</var-radio>
          <var-radio checked-value="s2t">简转繁</var-radio>
        </var-radio-group>

        <div class="reader-setting-range" data-label="字体大小" role="group" aria-label="字体大小">
          <div class="reader-setting-label"><span>字体大小</span><output>{{ settings.fontSize }} px</output></div>
          <var-slider v-model="settings.fontSize" :min="14" :max="30" :step="1" />
        </div>
        <div class="reader-setting-range" data-label="行间距" role="group" aria-label="行间距">
          <div class="reader-setting-label"><span>行间距</span><output>{{ settings.lineHeight.toFixed(1) }} 倍</output></div>
          <var-slider v-model="settings.lineHeight" :min="1.4" :max="2.6" :step="0.1" />
        </div>
        <div class="reader-setting-range" data-label="字间距" role="group" aria-label="字间距">
          <div class="reader-setting-label"><span>字间距</span><output>{{ settings.letterSpacing.toFixed(1) }} px</output></div>
          <var-slider v-model="settings.letterSpacing" :min="0" :max="4" :step="0.2" />
        </div>
        <div class="reader-setting-range" data-label="段间距" role="group" aria-label="段间距">
          <div class="reader-setting-label"><span>段间距</span><output>{{ settings.paragraphSpacing.toFixed(1) }} 倍</output></div>
          <var-slider v-model="settings.paragraphSpacing" :min="0.6" :max="2.4" :step="0.1" />
        </div>
        <div class="reader-setting-range" data-label="阅读宽度" role="group" aria-label="阅读宽度">
          <div class="reader-setting-label"><span>阅读宽度</span><output>{{ settings.contentWidth }} px</output></div>
          <var-slider v-model="settings.contentWidth" :min="560" :max="1100" :step="20" />
        </div>
      </template>
    </div>
  </var-popup>
</template>
