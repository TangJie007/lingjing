<script setup lang="ts">
import { ref } from "vue";
import { soundOn, playClick } from "../composables/useAudio";
import { showToast } from "../composables/useToast";

const t1 = ref(true);
const t2 = ref(false);
const t3 = ref(true);
const vol = ref(60);

function flip(key: "t1" | "t2" | "t3" | "sound") {
  if (key === "t1") t1.value = !t1.value;
  else if (key === "t2") t2.value = !t2.value;
  else if (key === "t3") t3.value = !t3.value;
  else {
    soundOn.value = !soundOn.value;
    showToast(soundOn.value ? "音效已开启" : "音效已关闭");
    if (soundOn.value) playClick(660);
  }
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
          :class="{ on: t1 }"
          role="switch"
          tabindex="0"
          :aria-checked="t1"
          aria-label="开机启动动态壁纸"
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
          :class="{ on: t2 }"
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
          :class="{ on: t3 }"
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
          :class="{ on: soundOn }"
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
        <input value="C:\Users\Admin\AppData\Roaming\LingJing\Wallpapers" readonly aria-label="壁纸路径" />
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
  </div>
</template>
