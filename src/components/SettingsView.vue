<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { showToast } from "../composables/useToast";
import {
  getAppPaths,
  loadSettings,
  setLibraryDir,
  useSettings,
  type AppPaths,
  type MigrationPlan,
} from "../composables/useSettings";
import { engineSetVolume } from "../composables/useEngine";
import MigrationModal from "./MigrationModal.vue";

const settings = useSettings();
const paths = ref<AppPaths | null>(null);
const migration = ref<MigrationPlan | null>(null);

const volPct = computed(() => Math.round(settings.value.defaultVolume * 100));

let suppressVol = false;

onMounted(async () => {
  await loadSettings();
  paths.value = await getAppPaths();
});

watch(volPct, async (v) => {
  if (suppressVol) return;
  try {
    await engineSetVolume(v / 100, settings.value.defaultVolume <= 0);
  } catch {
    /* ignore */
  }
});

function flip(key: "autostart" | "hideIconsOnDoubleClick" | "pauseOnFullscreen" | "sound") {
  if (key === "autostart") settings.value.autostart = !settings.value.autostart;
  if (key === "hideIconsOnDoubleClick") {
    settings.value.hideIconsOnDoubleClick = !settings.value.hideIconsOnDoubleClick;
    if (settings.value.hideIconsOnDoubleClick) {
      showToast("双击桌面隐藏图标将依赖后续系统接入");
    }
  }
  if (key === "pauseOnFullscreen") {
    settings.value.pauseOnFullscreen = !settings.value.pauseOnFullscreen;
  }
  if (key === "sound") {
    settings.value.soundOn = !settings.value.soundOn;
    showToast(settings.value.soundOn ? "音效已开启" : "音效已关闭");
  }
}

function flipPauseReason(key: "pauseOnBattery" | "pauseOnRdp") {
  settings.value[key] = !settings.value[key];
}

function onVolInput(e: Event) {
  const v = Number((e.target as HTMLInputElement).value);
  settings.value.defaultVolume = Math.max(0, Math.min(100, v)) / 100;
}

async function pickLibraryDir() {
  try {
    const picked = await openDialog({ directory: true, multiple: false });
    if (!picked || typeof picked !== "string") return;
    const plan = await setLibraryDir(picked);
    migration.value = plan;
    paths.value = await getAppPaths();
    showToast(`已选择新路径，共 ${plan.files.length} 个文件待迁移`);
  } catch (e) {
    showToast(e instanceof Error ? e.message : String(e));
  }
}

function closeMigration() {
  migration.value = null;
}

function onMigrated(report: { copied: number; skipped: number; failed: number; errors: string[]; cleanedStale?: number }) {
  const parts: string[] = [];
  if (report.copied) parts.push(`迁移 ${report.copied}`);
  if (report.skipped) parts.push(`跳过 ${report.skipped}`);
  if (report.failed) parts.push(`失败 ${report.failed}`);
  if (report.cleanedStale) parts.push(`清理旧目录 ${report.cleanedStale} 项`);
  showToast(parts.length ? `已完成：${parts.join("，")}` : "迁移已取消");
  migration.value = null;
  paths.value = null;
  void getAppPaths().then((p) => (paths.value = p));
  void loadSettings();
}
</script>

<template>
  <div class="main" style="overflow-y: auto;">
    <div class="set-group">
      <h3>基本设置</h3>
      <div class="set-row">
        <div class="lead">
          <div class="t">开机启动动态壁纸</div>
          <div class="d">系统启动时自动加载上一次壁纸</div>
        </div>
        <div
          class="toggle"
          :class="{ on: settings.autostart }"
          role="switch"
          tabindex="0"
          :aria-checked="settings.autostart"
          aria-label="开机启动动态壁纸"
          @click="flip('autostart')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('autostart'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">鼠标双击隐藏桌面图标</div>
          <div class="d">双击桌面空白处隐藏 / 显示图标</div>
        </div>
        <div
          class="toggle"
          :class="{ on: settings.hideIconsOnDoubleClick }"
          role="switch"
          tabindex="0"
          :aria-checked="settings.hideIconsOnDoubleClick"
          aria-label="鼠标双击隐藏桌面图标"
          @click="flip('hideIconsOnDoubleClick')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('hideIconsOnDoubleClick'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">其他程序全屏时变为静态</div>
          <div class="d">节省资源，游戏 / 观影更流畅</div>
        </div>
        <div
          class="toggle"
          :class="{ on: settings.pauseOnFullscreen }"
          role="switch"
          tabindex="0"
          :aria-checked="settings.pauseOnFullscreen"
          aria-label="其他程序全屏时变为静态"
          @click="flip('pauseOnFullscreen')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('pauseOnFullscreen'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">界面点击音效</div>
          <div class="d">点击导航与操作时的轻量反馈音</div>
        </div>
        <div
          class="toggle"
          :class="{ on: settings.soundOn }"
          role="switch"
          tabindex="0"
          :aria-checked="settings.soundOn"
          aria-label="界面点击音效"
          @click="flip('sound')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('sound'); } }"
        />
      </div>
    </div>

    <div class="set-group">
      <h3>自动暂停</h3>
      <div class="set-row">
        <div class="lead">
          <div class="t">切到电池时暂停</div>
          <div class="d">笔记本断电时自动暂停桌面壁纸</div>
        </div>
        <div
          class="toggle"
          :class="{ on: settings.pauseOnBattery }"
          role="switch"
          tabindex="0"
          :aria-checked="settings.pauseOnBattery"
          aria-label="切到电池时暂停"
          @click="flipPauseReason('pauseOnBattery')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flipPauseReason('pauseOnBattery'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">远程桌面时暂停</div>
          <div class="d">RDP / 远程会话期间自动暂停</div>
        </div>
        <div
          class="toggle"
          :class="{ on: settings.pauseOnRdp }"
          role="switch"
          tabindex="0"
          :aria-checked="settings.pauseOnRdp"
          aria-label="远程桌面时暂停"
          @click="flipPauseReason('pauseOnRdp')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flipPauseReason('pauseOnRdp'); } }"
        />
      </div>
    </div>

    <div class="set-group">
      <h3>壁纸路径</h3>
      <p class="path-note">
        更改路径后，<strong>library.json 与媒体文件</strong>会迁移到新目录。
        <strong>settings / favorites / last_wallpaper</strong> 仍保存在系统应用数据目录，不会被迁移。
      </p>
      <div class="path-ctrl">
        <input
          :value="paths?.libraryDir ?? '加载中…'"
          readonly
          aria-label="壁纸路径"
        />
        <div
          class="btn"
          role="button"
          tabindex="0"
          @click="pickLibraryDir"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); pickLibraryDir(); } }"
        >更改路径</div>
      </div>
      <div class="path-ctrl" style="margin-top: 10px;">
        <label class="toggle" :class="{ on: settings.importCopyToData }" role="switch" tabindex="0" :aria-checked="settings.importCopyToData" aria-label="复制到应用数据目录" @click="settings.importCopyToData = !settings.importCopyToData" @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); settings.importCopyToData = !settings.importCopyToData; } }" />
        <span style="font-size: 13px;">导入时复制到应用数据目录（关闭后仅记录原始路径）</span>
      </div>
    </div>

    <div class="set-group">
      <h3>控件设置</h3>
      <div class="set-row">
        <div class="lead">
          <div class="t">默认播放音量</div>
          <div class="d">仅对带声音的动态壁纸生效</div>
        </div>
        <div class="slider" :style="{ '--vol': `${volPct}%` }" aria-label="默认播放音量">
          <i :style="{ width: `${volPct}%` }" />
          <input
            type="range"
            min="0"
            max="100"
            step="1"
            :value="volPct"
            aria-label="默认播放音量滑块"
            @input="onVolInput"
          />
        </div>
      </div>
    </div>

    <footer-note>设置项已接入真实系统设置（部分依赖系统策略）</footer-note>

    <MigrationModal
      v-if="migration"
      :plan="migration"
      :open="true"
      @close="closeMigration"
      @done="onMigrated"
    />
  </div>
</template>

<style scoped>
footer-note {
  display: block;
  text-align: center;
  font-size: 11px;
  color: var(--text-3);
  margin-top: 12px;
}
.path-ctrl {
  display: flex;
  align-items: center;
  gap: 10px;
}
.path-note {
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.55;
  margin-bottom: 10px;
}
.path-note strong {
  color: var(--text);
  font-weight: 600;
}
.path-ctrl input {
  flex: 1;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  padding: 9px 12px;
  font: inherit;
  color: var(--text);
}
.path-ctrl input:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-soft);
  outline: none;
}
.slider {
  position: relative;
  width: 160px;
  height: 6px;
  border-radius: 6px;
  background: var(--border);
}
.slider i {
  position: absolute;
  inset: 0 auto 0 0;
  background: var(--primary);
  border-radius: 6px;
  width: var(--vol);
  transition: width var(--dur-fast) var(--ease);
}
.slider input[type=range] {
  position: absolute;
  inset: -6px 0;
  width: 100%;
  height: 18px;
  background: transparent;
  cursor: pointer;
  opacity: 0;
}
.slider::after {
  content: "";
  position: absolute;
  left: calc(var(--vol) - 9px);
  top: -6px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: var(--sh-md);
  border: 1px solid var(--border);
  pointer-events: none;
  transition: left var(--dur-fast) var(--ease);
}
</style>
