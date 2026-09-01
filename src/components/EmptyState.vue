<script setup lang="ts">
import emptyUrl from "../assets/svg/empty.svg";

withDefaults(
  defineProps<{
    title?: string;
    description?: string;
    size?: number;
  }>(),
  {
    size: 168,
  },
);
</script>

<template>
  <div class="empty-state">
    <img
      class="empty-illust"
      :src="emptyUrl"
      :width="size"
      :height="size"
      alt=""
      aria-hidden="true"
      draggable="false"
    />
    <h3 v-if="title" class="empty-title">{{ title }}</h3>
    <p v-if="description || $slots.default" class="empty-desc">
      <slot>{{ description }}</slot>
    </p>
  </div>
</template>

<style scoped>
.empty-state {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 24px 16px;
  text-align: center;
}
.empty-illust {
  display: block;
  object-fit: contain;
  flex-shrink: 0;
  user-select: none;
  pointer-events: none;
}
.empty-title {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--text);
}
.empty-desc {
  margin: 0;
  max-width: 360px;
  font-size: 13px;
  line-height: 1.55;
  color: var(--text-3);
}
</style>
