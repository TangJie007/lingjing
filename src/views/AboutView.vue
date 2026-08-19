<script setup lang="ts">
import AppLogo from "../components/AppLogo.vue";
</script>

<template>
  <div class="main about-page">
    <div class="placeholder">
      <AppLogo :size="72" />
      <h3>关于灵镜</h3>
      <p>动态壁纸客户端 · Phase 3 引擎已接入</p>
    </div>
  </div>
</template>

<style scoped>
.about-page .placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
}
.about-page h3 {
  margin: 0;
}
.about-page p {
  margin: 0;
  color: var(--text-2);
}
</style>
