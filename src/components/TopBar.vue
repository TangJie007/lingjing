<script setup lang="ts">
import { inject, ref, type Ref } from "vue";

const search = inject<Ref<string>>("topbarSearch", ref(""));
</script>

<template>
  <div class="topbar">
    <div class="search">
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="11" cy="11" r="7" />
        <path d="M21 21l-4.3-4.3" />
      </svg>
      <template v-if="!search">
        <span class="hint">大家都在搜：</span>
        <span class="kw">极光</span>
        <span class="hint">· 二次元 · 赛博城市</span>
      </template>
      <input
        v-model="search"
        type="search"
        class="search-input"
        :class="{ 'is-empty': !search }"
        aria-label="搜索壁纸"
      />
    </div>
  </div>
</template>
