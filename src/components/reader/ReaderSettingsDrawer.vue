<script setup lang="ts">
import { useReaderSettings } from "../../composables/reader/useReaderSettings";
import type { ReaderKind } from "../../types/reader";

interface ReaderSettingsDrawerProps {
  kind: ReaderKind;
  title: string;
  titleClick?: () => void;
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
</script>

<template>
  <Teleport to="body">
    <Transition name="reader-chapter-nav">
      <nav v-if="visible" class="reader-chapter-nav" aria-label="章节导航">
        <var-button
          type="primary"
          tonal
          round
          :disabled="previousDisabled"
          :title="previousDisabled ? '已是第一话' : '上一话'"
          aria-label="上一话"
          @click="emit('previous')"
          ><var-icon name="arrow-left"
        /></var-button>
        <strong
          v-if="!titleClick"
          class="reader-chapter-nav-title"
          :title="title"
          >{{ title }}</strong
        >
        <var-button
          v-else
          text
          class="reader-chapter-nav-title"
          :title="title"
          @click="titleClick"
        >
          {{ title }}
        </var-button>
        <var-button
          type="primary"
          tonal
          round
          :disabled="nextDisabled"
          :title="nextDisabled ? '已是最后一话' : '下一话'"
          aria-label="下一话"
          @click="emit('next')"
          ><var-icon name="arrow-right"
        /></var-button>
      </nav>
    </Transition>
  </Teleport>

  <var-popup
    v-model:show="visible"
    class="reader-settings-drawer"
    position="bottom"
    :z-index="2000"
    teleport="body"
  >
    <div class="settings-panel">
      <div class="settings-handle" aria-hidden="true"></div>
      <div class="settings-title">
        <strong>阅读设置</strong>
        <var-button v-if="kind === 'novel'" text size="small" @click="reset">
          恢复默认
          <var-icon name="restore" />
        </var-button>
      </div>

      <label>阅读模式</label>
      <var-radio-group v-model="settings.mode">
        <var-radio checked-value="scroll">滚动阅读</var-radio>
        <var-radio checked-value="paged">分页阅读</var-radio>
      </var-radio-group>

      <template v-if="settings.mode === 'paged'">
        <label>翻页方向</label>
        <var-radio-group v-model="settings.pageTurnDirection">
          <var-radio checked-value="left-previous">从左至右</var-radio>
          <var-radio checked-value="left-next">从右至左</var-radio>
        </var-radio-group>
      </template>

      <label>背景主题</label>
      <var-radio-group v-model="settings.theme">
        <var-radio checked-value="paper">纸张</var-radio>
        <var-radio checked-value="light">明亮</var-radio>
        <var-radio checked-value="night">夜间</var-radio>
      </var-radio-group>

      <template v-if="kind === 'novel'">
        <label>正文字体</label>
        <var-radio-group v-model="settings.font">
          <var-radio checked-value="serif">衬线</var-radio>
          <var-radio checked-value="sans">无衬线</var-radio>
        </var-radio-group>

        <label>文字转换</label>
        <var-radio-group v-model="settings.convert">
          <var-radio checked-value="original">原文</var-radio>
          <var-radio checked-value="t2s">繁转简</var-radio>
          <var-radio checked-value="s2t">简转繁</var-radio>
        </var-radio-group>

        <label
          ><span>字体大小</span><b>{{ settings.fontSize }} px</b></label
        >
        <var-slider v-model="settings.fontSize" :min="14" :max="30" :step="1" />

        <label
          ><span>行间距</span
          ><b>{{ settings.lineHeight.toFixed(1) }} 倍</b></label
        >
        <var-slider
          v-model="settings.lineHeight"
          :min="1.4"
          :max="2.6"
          :step="0.1"
        />

        <label
          ><span>字间距</span
          ><b>{{ settings.letterSpacing.toFixed(1) }} px</b></label
        >
        <var-slider
          v-model="settings.letterSpacing"
          :min="0"
          :max="4"
          :step="0.2"
        />

        <label
          ><span>段间距</span
          ><b>{{ settings.paragraphSpacing.toFixed(1) }} 倍</b></label
        >
        <var-slider
          v-model="settings.paragraphSpacing"
          :min="0.6"
          :max="2.4"
          :step="0.1"
        />

        <label
          ><span>阅读宽度</span><b>{{ settings.contentWidth }} px</b></label
        >
        <var-slider
          v-model="settings.contentWidth"
          :min="560"
          :max="1100"
          :step="20"
        />
      </template>
    </div>
  </var-popup>
</template>
