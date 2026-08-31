<script setup lang="ts">
</script>

<template>
  <div class="main pet-page">
    <div class="placeholder">
      <div class="pet-icon" aria-hidden="true">
        <svg viewBox="0 0 64 64" width="72" height="72">
          <ellipse cx="32" cy="38" rx="18" ry="16" fill="currentColor" opacity="0.9" />
          <circle cx="20" cy="22" r="7" fill="currentColor" />
          <circle cx="44" cy="22" r="7" fill="currentColor" />
          <circle cx="26" cy="36" r="2.2" fill="var(--bg, #fff)" />
          <circle cx="38" cy="36" r="2.2" fill="var(--bg, #fff)" />
          <ellipse cx="32" cy="42" rx="3" ry="2" fill="var(--bg, #fff)" opacity="0.85" />
        </svg>
      </div>
      <h3>桌宠管理</h3>
      <p>在此启用、更换与配置桌面宠物。完整能力将随后续版本接入。</p>
    </div>
  </div>
</template>

<style scoped>
.pet-page .placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
}
.pet-page h3 {
  margin: 0;
}
.pet-page p {
  margin: 0;
  color: var(--text-2);
}
.pet-icon {
  color: var(--accent, #3d8bfd);
  line-height: 0;
}
</style>
