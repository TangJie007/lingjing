<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import AppLogo from "./AppLogo.vue";
import { showToast } from "../composables/useToast";
import iconLogin from "../assets/svg/login.svg";
import iconMin from "../assets/svg/min.svg";
import iconClose from "../assets/svg/close.svg";
import logoTxt from "../assets/logo-txt.jpeg";

defineProps<{
  onlineEnabled?: boolean;
  loggedIn?: boolean;
  userLabel?: string;
}>();

const emit = defineEmits<{ (e: "login"): void }>();

async function onDrag(e: MouseEvent) {
  if (e.button !== 0) return;
  const target = e.target as HTMLElement | null;
  if (target?.closest(".win-act, .app-logo, .win-logo-txt")) return;
  await invoke("start_drag");
}

async function minimize() {
  try {
    await invoke("minimize_main");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

async function hideToTray() {
  try {
    await invoke("hide_main");
    showToast("已隐藏到系统托盘，点击托盘图标可恢复");
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}
</script>

<template>
  <div class="win-bar" @mousedown="onDrag">
    <div class="win-brand">
      <AppLogo class="win-logo" :size="22" decorative />
      <img
        class="win-logo-txt"
        :src="logoTxt"
        alt="灵镜 LINGJING"
        draggable="false"
      />
    </div>
    <span class="win-spacer" />
    <div class="win-acts">
      <button
        v-if="onlineEnabled"
        type="button"
        class="win-act icon-btn login"
        :class="{ 'is-logged-in': loggedIn }"
        :aria-label="loggedIn ? `已登录：${userLabel}` : '登录灵境社区'"
        :title="loggedIn ? userLabel : '登录'"
        @mousedown.stop
        @click.stop="emit('login')"
      >
        <img :src="iconLogin" alt="" class="win-act-icon" draggable="false" />
        <span v-if="loggedIn" class="win-act-label">{{ userLabel }}</span>
      </button>
      <button
        type="button"
        class="win-act icon-btn"
        aria-label="最小化"
        title="最小化"
        @mousedown.stop
        @click.stop="minimize"
      >
        <img :src="iconMin" alt="" class="win-act-icon" draggable="false" />
      </button>
      <button
        type="button"
        class="win-act icon-btn close"
        aria-label="隐藏到托盘"
        title="隐藏到托盘"
        @mousedown.stop
        @click.stop="hideToTray"
      >
        <img :src="iconClose" alt="" class="win-act-icon" draggable="false" />
      </button>
    </div>
  </div>
</template>
