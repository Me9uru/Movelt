<script setup lang="ts">
import { RefreshLeft } from "@element-plus/icons-vue";
import { useReaderSettings, type ReaderKind } from "../../composables/useReaderSettings";

const props = defineProps<{
  kind: ReaderKind;
}>();

const visible = defineModel<boolean>({ required: true });
const { settings, reset } = useReaderSettings(props.kind);
</script>

<template>
  <el-drawer
    v-model="visible"
    class="reader-settings-drawer"
    direction="btt"
    :size="kind === 'novel'
      ? 'min(680px, calc(100dvh - env(safe-area-inset-top) - 138px))'
      : 'min(360px, calc(100dvh - env(safe-area-inset-top) - 138px))'"
    :with-header="false"
    :z-index="2000"
    append-to-body
  >
    <div class="settings-panel">
      <div class="settings-handle" aria-hidden="true"></div>
      <div class="settings-title">
        <strong>阅读设置</strong>
        <el-button
          v-if="kind === 'novel'"
          :icon="RefreshLeft"
          text
          size="small"
          @click="reset"
        >
          恢复默认
        </el-button>
      </div>

      <label>阅读模式</label>
      <el-radio-group v-model="settings.mode" size="small">
        <el-radio-button value="scroll">滚动阅读</el-radio-button>
        <el-radio-button value="paged">分页阅读</el-radio-button>
      </el-radio-group>

      <template v-if="settings.mode === 'paged'">
        <label>翻页点击方向</label>
        <el-radio-group v-model="settings.pageTurnDirection" size="small">
          <el-radio-button value="left-previous">左边上一页 · 右边下一页</el-radio-button>
          <el-radio-button value="left-next">左边下一页 · 右边上一页</el-radio-button>
        </el-radio-group>
      </template>

      <label>背景主题</label>
      <el-radio-group v-model="settings.theme" size="small">
        <el-radio-button value="paper">纸张</el-radio-button>
        <el-radio-button value="light">明亮</el-radio-button>
        <el-radio-button value="night">夜间</el-radio-button>
      </el-radio-group>

      <template v-if="kind === 'novel'">
        <label>正文字体</label>
        <el-radio-group v-model="settings.font" size="small">
          <el-radio-button value="serif">衬线</el-radio-button>
          <el-radio-button value="sans">无衬线</el-radio-button>
        </el-radio-group>

        <label>文字转换</label>
        <el-radio-group v-model="settings.convert" size="small">
          <el-radio-button value="original">原文</el-radio-button>
          <el-radio-button value="t2s">繁转简</el-radio-button>
          <el-radio-button value="s2t">简转繁</el-radio-button>
        </el-radio-group>

        <label><span>字体大小</span><b>{{ settings.fontSize }} px</b></label>
        <el-slider v-model="settings.fontSize" :min="14" :max="30" :step="1" />

        <label><span>行间距</span><b>{{ settings.lineHeight.toFixed(1) }} 倍</b></label>
        <el-slider v-model="settings.lineHeight" :min="1.4" :max="2.6" :step="0.1" />

        <label><span>字间距</span><b>{{ settings.letterSpacing.toFixed(1) }} px</b></label>
        <el-slider v-model="settings.letterSpacing" :min="0" :max="4" :step="0.2" />

        <label><span>段间距</span><b>{{ settings.paragraphSpacing.toFixed(1) }} 倍</b></label>
        <el-slider v-model="settings.paragraphSpacing" :min="0.6" :max="2.4" :step="0.1" />

        <label><span>阅读宽度</span><b>{{ settings.contentWidth }} px</b></label>
        <el-slider v-model="settings.contentWidth" :min="560" :max="1100" :step="20" />
      </template>
    </div>
  </el-drawer>
</template>
