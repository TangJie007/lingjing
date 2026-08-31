<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

defineProps<{
  fencesCollapsed: boolean;
  itemCount?: number;
}>();

const emit = defineEmits<{
  toggleFences: [];
  openSettings: [];
  disableOrganize: [];
}>();

const open = ref(false);

function expand() {
  open.value = true;
}

function collapse() {
  open.value = false;
}

function onToggleFences() {
  emit("toggleFences");
  collapse();
}

function onOpenSettings() {
  emit("openSettings");
  collapse();
}

function onDisableOrganize() {
  emit("disableOrganize");
  collapse();
}

function onDocPointerDown(e: PointerEvent) {
  const t = e.target as HTMLElement | null;
  if (!t?.closest?.(".dynamic-island")) collapse();
}

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") collapse();
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocPointerDown, true);
  window.addEventListener("keydown", onKey);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocPointerDown, true);
  window.removeEventListener("keydown", onKey);
});
</script>

<template>
  <div
    class="dynamic-island no-drag"
    :class="{ open }"
    role="toolbar"
    aria-label="灵镜桌面整理"
    @contextmenu.stop.prevent
  >
    <button
      v-if="!open"
      type="button"
      class="island-pill"
      @click="expand"
    >
      <span class="island-brand">灵镜</span>
      <span class="island-sep" aria-hidden="true" />
      <span class="island-status">{{
        fencesCollapsed ? "已收起" : itemCount != null ? `${itemCount} 项` : "整理中"
      }}</span>
    </button>

    <div v-else class="island-panel">
      <button type="button" class="island-action" @click="onToggleFences">
        {{ fencesCollapsed ? "展开围栏" : "收起围栏" }}
      </button>
      <button type="button" class="island-action" @click="onOpenSettings">
        打开设置
      </button>
      <button
        type="button"
        class="island-action danger"
        @click="onDisableOrganize"
      >
        关闭整理
      </button>
    </div>
  </div>
</template>
