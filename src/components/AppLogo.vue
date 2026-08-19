<script setup lang="ts">
import logoUrl from "../assets/community.png";

withDefaults(
  defineProps<{
    size?: number;
    alt?: string;
    decorative?: boolean;
  }>(),
  {
    size: 22,
    alt: "灵镜 LINGJING",
    decorative: false,
  },
);
</script>

<template>
  <img
    class="app-logo"
    :src="logoUrl"
    :alt="decorative ? '' : alt"
    :aria-hidden="decorative ? 'true' : undefined"
    :width="size"
    :height="size"
    :style="{ width: `${size}px`, height: `${size}px` }"
  />
</template>

<style scoped>
.app-logo {
  border-radius: 50%;
  object-fit: cover;
  flex-shrink: 0;
  display: block;
}
</style>
