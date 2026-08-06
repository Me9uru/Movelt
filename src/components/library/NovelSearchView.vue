<script setup lang="ts">
import { Collection, Search } from "@element-plus/icons-vue";
import type { NovelSourceInfo, NovelSummary } from "../../services/novel";

defineProps<{
  sourceOptions: NovelSourceInfo[];
  results: NovelSummary[];
  loading: boolean;
}>();

const query = defineModel<string>("query", { required: true });
const selectedSource = defineModel<string>("selectedSource", { required: true });

const emit = defineEmits<{
  search: [];
  sourceChange: [];
  openNovel: [novel: NovelSummary];
}>();
</script>

<template>
  <section class="search-view">
    <div class="hero">
      <form class="search-box" @submit.prevent="emit('search')">
        <el-select v-model="selectedSource" class="source-select" size="large" aria-label="小说来源"
          @change="emit('sourceChange')">
          <el-option v-for="source in sourceOptions" :key="source.id" :label="source.name" :value="source.id" />
        </el-select>
        <el-input v-model="query" :prefix-icon="Search" size="large" clearable :disabled="loading" aria-label="小说名或作者"
          placeholder="输入小说名或作者" />
        <el-button native-type="submit" type="primary" size="large" :loading="loading"
          :disabled="loading || !selectedSource || !query.trim()">
          搜索作品
        </el-button>
      </form>
    </div>

    <section class="library-section">
      <div v-if="results.length" class="section-heading">
        <div>
          <h2>找到的作品</h2>
        </div>
        <el-tag round effect="plain">{{ results.length }} 本</el-tag>
      </div>

      <div v-if="loading" class="result-grid" aria-label="正在搜索">
        <el-card v-for="item in 6" :key="item" shadow="never" class="book-card skeleton-card">
          <el-skeleton animated>
            <template #template>
              <el-skeleton-item variant="image" class="skeleton-cover" />
              <el-skeleton-item variant="h3" style="width: 82%" />
              <el-skeleton-item variant="text" style="width: 48%" />
            </template>
          </el-skeleton>
        </el-card>
      </div>

      <el-empty v-else-if="results.length === 0" :image-size="112" description="输入书名或作者开始搜索">
        <p class="empty-tip">首次加载书库索引可能需要十几秒，请稍候。</p>
      </el-empty>

      <div v-else class="result-grid">
        <el-card v-for="novel in results" :key="`${novel.source}:${novel.id}`" class="book-card"
          :class="{ 'book-card--disabled': loading }" shadow="hover" :tabindex="loading ? -1 : 0"
          :aria-disabled="loading" @click="emit('openNovel', novel)" @keydown.enter="emit('openNovel', novel)">
          <el-image v-if="novel.cover_url" class="book-cover" :src="novel.cover_url" :alt="novel.title" fit="cover"
            lazy>
            <template #error>
              <div class="cover-placeholder">
                <el-icon>
                  <Collection />
                </el-icon>
              </div>
            </template>
          </el-image>
          <div v-else class="cover-placeholder">
            <el-icon>
              <Collection />
            </el-icon>
          </div>
          <div class="book-meta">
            <strong>{{ novel.title }}</strong>
            <span>查看详情 <span aria-hidden="true">→</span></span>
          </div>
        </el-card>
      </div>
    </section>
  </section>
</template>
