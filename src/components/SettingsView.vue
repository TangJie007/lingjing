<script setup lang="ts">
import { ref } from "vue";
import { soundOn, playClick } from "../composables/useAudio";
import { showToast } from "../composables/useToast";

// 4 个 toggle
const t1 = ref(true); // 开机启动
const t2 = ref(false); // 双击隐藏
const t3 = ref(true); // 全屏静态
// t4 用全局 soundOn

const vol = ref(60);

function flip(key: "t1" | "t2" | "t3" | "sound") {
  if (key === "t1") t1.value = !t1.value;
  else if (key === "t2") t2.value = !t2.value;
  else if (key === "t3") t3.value = !t3.value;
  else if (key === "sound") {
    soundOn.value = !soundOn.value;
    showToast(soundOn.value ? "音效已开启" : "音效已关闭");
    if (soundOn.value) playClick(660);
  }
}
</script>

<template>
  <div class="settings flex-1 overflow-y-auto px-7 py-5">
    <div class="set-group">
      <h3>基本设置</h3>
      <div class="set-row">
        <div class="lead">
          <div class="t">开机启动动态壁纸</div>
          <div class="d">系统启动时自动加载上一次壁纸</div>
        </div>
        <div
          class="toggle"
          :class="t1 ? 'on' : ''"
          role="switch"
          tabindex="0"
          :aria-checked="t1"
          :aria-label="'开机启动动态壁纸'"
          @click="flip('t1')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('t1'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">鼠标双击隐藏桌面图标</div>
          <div class="d">双击桌面空白处隐藏 / 显示图标</div>
        </div>
        <div
          class="toggle"
          :class="t2 ? 'on' : ''"
          role="switch"
          tabindex="0"
          :aria-checked="t2"
          aria-label="鼠标双击隐藏桌面图标"
          @click="flip('t2')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('t2'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">其他程序全屏时变为静态</div>
          <div class="d">节省资源，游戏 / 观影更流畅</div>
        </div>
        <div
          class="toggle"
          :class="t3 ? 'on' : ''"
          role="switch"
          tabindex="0"
          :aria-checked="t3"
          aria-label="其他程序全屏时变为静态"
          @click="flip('t3')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('t3'); } }"
        />
      </div>
      <div class="set-row">
        <div class="lead">
          <div class="t">界面点击音效</div>
          <div class="d">点击导航与操作时的轻量反馈音</div>
        </div>
        <div
          class="toggle"
          :class="soundOn ? 'on' : ''"
          role="switch"
          tabindex="0"
          :aria-checked="soundOn"
          aria-label="界面点击音效"
          @click="flip('sound')"
          @keydown="(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); flip('sound'); } }"
        />
      </div>
    </div>

    <div class="set-group">
      <h3>壁纸路径</h3>
      <div class="path-ctrl">
        <input
          value="C:\Users\Admin\AppData\Roaming\LingJing\Wallpapers"
          readonly
          aria-label="壁纸路径"
        />
        <div class="btn" role="button" tabindex="0" @click="showToast('更改路径开发中')">更改路径</div>
      </div>
    </div>

    <div class="set-group">
      <h3>控件设置</h3>
      <div class="set-row">
        <div class="lead">
          <div class="t">默认播放音量</div>
          <div class="d">仅对带声音的动态壁纸生效</div>
        </div>
        <div class="slider" :style="{ '--vol': `${vol}%` }" aria-label="默认播放音量">
          <i :style="{ width: `${vol}%` }" />
        </div>
      </div>
    </div>

    <div class="phase-note">设置项将在 Phase 3 接入真实系统设置</div>
  </div>
</template>

<style scoped>
.settings {
  background: var(--bg);
  color: var(--text);
}

.set-group {
  margin-bottom: 26px;
}
.set-group h3 {
  font-size: 15px;
  font-weight: 700;
  margin-bottom: 6px;
}

.set-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 0;
  border-bottom: 1px solid var(--border);
}
.set-row .lead {
  flex: 1;
}
.set-row .lead .t {
  font-size: 14px;
  font-weight: 600;
}
.set-row .lead .d {
  font-size: 12px;
  color: var(--text-3);
  margin-top: 2px;
}

.toggle {
  width: 44px;
  height: 26px;
  border-radius: var(--r-pill);
  background: var(--border-strong);
  position: relative;
  cursor: pointer;
  flex-shrink: 0;
  transition: background var(--dur-base) var(--ease);
}
.toggle::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  box-shadow: var(--sh-sm);
  transition: left var(--dur-base) var(--ease);
}
.toggle.on {
  background: var(--primary);
}
.toggle.on::after {
  left: 21px;
}

.path-ctrl {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 10px;
}
.path-ctrl input {
  flex: 1;
  font-size: 13px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface-2);
  color: var(--text);
  transition: border-color var(--dur-fast) var(--ease),
    box-shadow var(--dur-fast) var(--ease);
}
.path-ctrl input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--primary-soft);
}
.path-ctrl .btn {
  font-size: 13px;
  padding: 10px 16px;
  border-radius: var(--r-md);
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text);
  cursor: pointer;
  transition: transform var(--dur-fast) var(--ease),
    background var(--dur-fast) var(--ease);
}
.path-ctrl .btn:hover {
  background: var(--surface-2);
}
.path-ctrl .btn:active {
  transform: scale(0.96);
}

.slider {
  width: 160px;
  height: 6px;
  border-radius: 6px;
  background: var(--border);
  position: relative;
}
.slider i {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background: var(--primary);
  border-radius: 6px;
  transition: width var(--dur-fast) var(--ease);
}
.slider::after {
  content: "";
  position: absolute;
  left: calc(var(--vol, 60%) - 9px);
  top: -6px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  box-shadow: var(--sh-md);
  border: 1px solid var(--border);
  transition: left var(--dur-fast) var(--ease);
}

.phase-note {
  text-align: center;
  color: var(--text-3);
  font-size: 11px;
  margin-top: 32px;
  margin-bottom: 24px;
}
</style>
