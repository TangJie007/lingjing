<script setup lang="ts">
import { ref } from "vue";
import AppLogo from "./AppLogo.vue";

const props = defineProps<{ open: boolean; saving?: boolean }>();
const emit = defineEmits<{ (e: "confirm", autostart: boolean): void }>();

const autostart = ref(false);

function onConfirm() {
  if (props.saving) return;
  emit("confirm", autostart.value);
}
</script>

<template>
  <Teleport to=".window">
    <div v-if="open" class="fr-mask">
      <div class="fr-card" role="dialog" aria-modal="true" aria-label="欢迎使用灵镜">
        <header>
          <div class="fr-title">
            <AppLogo :size="28" decorative />
            <h3>欢迎使用灵镜</h3>
          </div>
        </header>

        <p class="fr-desc">
          首次使用可设置是否在系统启动时自动加载上一次壁纸。你也可以稍后在设置中更改。
        </p>

        <div class="fr-row">
          <div class="fr-lead">
            <div class="t">开机启动动态壁纸</div>
            <div class="d">系统启动时自动加载上一次壁纸</div>
          </div>
          <button
            type="button"
            class="toggle"
            :class="{ on: autostart }"
            role="switch"
            tabindex="0"
            :aria-checked="autostart"
            aria-label="开机启动动态壁纸"
            @click="autostart = !autostart"
            @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); autostart = !autostart; } }"
          />
        </div>

        <footer>
          <button type="button" class="fr-confirm" :disabled="saving" @click="onConfirm">
            {{ saving ? "保存中…" : "确认" }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.fr-mask {
  position: absolute;
  inset: 0;
  background: rgba(15, 17, 24, 0.45);
  backdrop-filter: blur(6px);
  display: grid;
  place-items: center;
  z-index: 1100;
  border-radius: inherit;
}
.fr-card {
  width: 420px;
  max-width: 92vw;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--sh-win);
  padding: 20px 22px 22px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.fr-title {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}
header h3 {
  font-size: 16px;
  font-weight: 700;
  margin: 0;
}
.fr-desc {
  margin: 0 0 16px;
  font-size: 13px;
  line-height: 1.55;
  color: var(--text-2);
}
.fr-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface-2);
}
.fr-lead .t {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}
.fr-lead .d {
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.45;
}
.toggle {
  flex: 0 0 auto;
  width: 44px;
  height: 24px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface);
  position: relative;
  cursor: pointer;
  transition: background 0.18s ease, border-color 0.18s ease;
}
.toggle::after {
  content: "";
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
  transition: transform 0.18s ease;
}
.toggle.on {
  background: var(--primary);
  border-color: var(--primary);
}
.toggle.on::after {
  transform: translateX(20px);
}
footer {
  display: flex;
  justify-content: flex-end;
  margin-top: 18px;
}
.fr-confirm {
  min-width: 96px;
  height: 36px;
  padding: 0 18px;
  border: none;
  border-radius: var(--r-md);
  background: var(--primary);
  color: #fff;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
}
.fr-confirm:hover {
  filter: brightness(1.05);
}
.fr-confirm:disabled {
  opacity: 0.65;
  cursor: wait;
}
</style>
