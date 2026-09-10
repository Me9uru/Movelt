<script setup lang="ts">
import { ref, watchPostEffect } from "vue";
import { useReaderSettings } from "../../composables/reader/useReaderSettings";
const { settings } = useReaderSettings("novel");
const panel = ref<HTMLElement | null>(null);
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

</script>

<template>
  <div ref="panel" class="novel-appearance-settings">
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
  </div>
</template>

<style scoped>
.novel-appearance-settings {
  display: grid;
  gap: var(--space-2);
}
</style>
