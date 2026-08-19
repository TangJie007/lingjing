<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  runMigration,
  type MigrationPlan,
  type MigrationProgress,
} from "../composables/useSettings";

const props = defineProps<{
  plan: MigrationPlan;
  open: boolean;
}>();
const emit = defineEmits<{
  (e: "close"): void;
  (e: "done", report: { copied: number; skipped: number; failed: number; errors: string[]; cleanedStale?: number }): void;
}>();

const busy = ref(false);
const progress = ref<MigrationProgress>({ done: 0, total: props.plan.files.length, relPath: "" });
const lastReport = ref<{ copied: number; skipped: number; failed: number; errors: string[]; cleanedStale?: number } | null>(null);
let unlisten: UnlistenFn | undefined;

onMounted(async () => {
  unlisten = await listen<MigrationProgress>("library-migration-progress", (ev) => {
    progress.value = ev.payload;
  });
});
onUnmounted(() => {
  unlisten?.();
});

const ratio = (() => {
  if (props.plan.files.length === 0) return 1;
  return progress.value.done / Math.max(1, progress.value.total);
})();

async function migrate(keepOriginals: boolean) {
  if (busy.value) return;
  busy.value = true;
  try {
    const report = await runMigration(keepOriginals);
    lastReport.value = report;
    emit("done", report);
  } catch (e) {
    lastReport.value = { copied: 0, skipped: 0, failed: 1, errors: [String(e)] };
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="open" class="mm-mask" @click.self="emit('close')">
    <div class="mm-card" role="dialog" aria-modal="true" aria-label="迁移壁纸路径">
      <header>
        <h3>迁移到新路径</h3>
        <p>将本地库索引与媒体文件复制到目标位置。原文件可选择保留或迁移后删除。</p>
      </header>
      <ul class="mm-notes">
        <li><strong>会迁移</strong>：library.json、library/ 下的媒体文件</li>
        <li><strong>不会迁移</strong>：settings.json、favorites.json、last_wallpaper.json（仍留在应用数据目录）</li>
        <li>选择「迁移」（不保留原文件）时，还会清理应用数据目录中的旧 library 副本</li>
      </ul>
      <div class="mm-paths">
        <div>
          <span class="lab">原路径</span>
          <code>{{ plan.fromDir }}</code>
        </div>
        <div>
          <span class="lab">新路径</span>
          <code>{{ plan.toDir }}</code>
        </div>
      </div>
      <div class="mm-stat">
        <span>共 {{ plan.files.length }} 个文件 / {{ (plan.files.reduce((s, f) => s + f.size, 0) / 1048576).toFixed(1) }} MB</span>
      </div>
      <div v-if="lastReport" class="mm-result">
        <div v-if="lastReport.copied">已迁移 {{ lastReport.copied }} 个</div>
        <div v-if="lastReport.skipped">跳过 {{ lastReport.skipped }} 个（目标已存在）</div>
        <div v-if="lastReport.failed">失败 {{ lastReport.failed }} 个：{{ lastReport.errors.slice(0, 3).join("；") }}</div>
        <div v-if="lastReport.cleanedStale">已清理旧目录 {{ lastReport.cleanedStale }} 项</div>
      </div>
      <div class="mm-progress" v-else>
        <div class="bar" :style="{ transform: `scaleX(${ratio})` }" />
        <div class="lab">{{ progress.done }} / {{ progress.total }}<span v-if="progress.relPath"> · {{ progress.relPath }}</span></div>
      </div>
      <footer>
        <button class="btn ghost" :disabled="busy" @click="emit('close')">取消</button>
        <button class="btn" :disabled="busy" @click="migrate(true)">迁移并保留原文件</button>
        <button class="btn primary" :disabled="busy" @click="migrate(false)">迁移</button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.mm-mask {
  position: fixed;
  inset: 0;
  background: rgba(15, 17, 24, 0.45);
  backdrop-filter: blur(6px);
  display: grid;
  place-items: center;
  z-index: 80;
}
.mm-card {
  width: 520px;
  max-width: 92vw;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--r-lg);
  box-shadow: var(--sh-win);
  padding: 22px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}
header h3 { font-size: 16px; font-weight: 700; }
header p { font-size: 12.5px; color: var(--text-2); margin-top: 6px; }
.mm-notes {
  margin: 0;
  padding-left: 18px;
  font-size: 12px;
  color: var(--text-2);
  line-height: 1.6;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.mm-notes strong { color: var(--text); font-weight: 600; }
.mm-paths { display: flex; flex-direction: column; gap: 8px; font-size: 12.5px; }
.mm-paths .lab { color: var(--text-3); margin-right: 8px; }
.mm-paths code {
  background: var(--surface-2);
  padding: 4px 8px;
  border-radius: 6px;
  word-break: break-all;
  display: block;
  margin-top: 2px;
}
.mm-stat { font-size: 12.5px; color: var(--text-2); }
.mm-progress { display: flex; flex-direction: column; gap: 6px; font-size: 12px; color: var(--text-2); }
.mm-progress .bar {
  height: 6px;
  border-radius: 999px;
  background: var(--primary);
  transform-origin: left center;
  transition: transform var(--dur-base) var(--ease);
  width: 100%;
  background: var(--border);
  position: relative;
}
.mm-progress .bar::after {
  content: "";
  position: absolute;
  inset: 0;
  background: var(--primary);
  border-radius: inherit;
  transform-origin: left center;
  transform: scaleX(var(--rx, 0));
}
.mm-result { font-size: 12.5px; color: var(--text-2); display: flex; flex-direction: column; gap: 4px; }
footer { display: flex; justify-content: flex-end; gap: 8px; }
.btn {
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text);
  border-radius: var(--r-md);
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: transform var(--dur-fast) var(--ease-spring), background var(--dur-fast) var(--ease);
}
.btn:hover { background: var(--surface-2); }
.btn:active { transform: scale(0.97); }
.btn:disabled { opacity: 0.55; cursor: default; }
.btn.primary { background: var(--primary); color: #fff; border-color: var(--primary); }
.btn.primary:hover { background: var(--primary-hover); border-color: var(--primary-hover); }
</style>
