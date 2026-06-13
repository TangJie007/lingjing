<script setup lang="ts">
import { useId } from 'vue'

const model = defineModel<boolean>({ default: false })

defineProps<{
  disabled?: boolean
}>()

const filterId = `toggle-goo-${useId().replace(/:/g, '')}`
</script>

<template>
  <label class="toggle-container" :class="{ 'is-disabled': disabled }">
    <input
      v-model="model"
      type="checkbox"
      class="toggle-input"
      role="switch"
      :disabled="disabled"
    />
    <svg
      class="toggle-svg"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 292 142"
      aria-hidden="true"
    >
      <defs>
        <filter :id="filterId">
          <feGaussianBlur in="SourceGraphic" stdDeviation="10" result="blur" />
          <feColorMatrix
            in="blur"
            mode="matrix"
            values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 18 -7"
            result="goo"
          />
        </filter>
      </defs>
      <path
        d="M71 142C31.7878 142 0 110.212 0 71C0 31.7878 31.7878 0 71 0C110.212 0 119 30 146 30C173 30 182 0 221 0C260 0 292 31.7878 292 71C292 110.212 260.212 142 221 142C181.788 142 173 112 146 112C119 112 110.212 142 71 142Z"
        class="toggle-background"
      />
      <rect class="toggle-icon on" x="64" y="39" width="12" height="64" rx="6" />
      <path
        class="toggle-icon off"
        fill-rule="evenodd"
        d="M221 91C232.046 91 241 82.0457 241 71C241 59.9543 232.046 51 221 51C209.954 51 201 59.9543 201 71C201 82.0457 209.954 91 221 91ZM221 103C238.673 103 253 88.6731 253 71C253 53.3269 238.673 39 221 39C203.327 39 189 53.3269 189 71C189 88.6731 203.327 103 221 103Z"
      />
      <g :filter="`url(#${filterId})`">
        <rect class="toggle-circle-center" x="13" y="42" width="116" height="58" rx="29" fill="#fff" />
        <rect class="toggle-circle left" x="14" y="14" width="114" height="114" rx="58" fill="#fff" />
        <rect class="toggle-circle right" x="164" y="14" width="114" height="114" rx="58" fill="#fff" />
      </g>
    </svg>
  </label>
</template>

<style scoped>
.toggle-container {
  --toggle-active: var(--color-primary, #008336);
  --toggle-inactive: #d4ddd8;
  --toggle-off-icon: #eef3f0;
  position: relative;
  display: inline-flex;
  flex-shrink: 0;
  aspect-ratio: 292 / 142;
  height: 1.875rem;
  cursor: pointer;
}

.toggle-container.is-disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.toggle-input {
  appearance: none;
  margin: 0;
  position: absolute;
  z-index: 1;
  inset: 0;
  width: 100%;
  height: 100%;
  cursor: inherit;
  border-radius: var(--radius-full);
}

.toggle-input:focus-visible {
  outline: 2px solid rgba(0, 131, 54, 0.35);
  outline-offset: 2px;
}

.toggle-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
  display: block;
}

.toggle-background {
  fill: var(--toggle-inactive);
  transition: fill 0.4s var(--ease-out-quart);
}

.toggle-input:checked + .toggle-svg .toggle-background {
  fill: var(--toggle-active);
}

.toggle-circle-center {
  transform-origin: center;
  transition: transform 0.55s var(--ease-out-expo);
}

.toggle-input:checked + .toggle-svg .toggle-circle-center {
  transform: translateX(150px);
}

.toggle-circle {
  transform-origin: center;
  transition: transform 0.4s var(--ease-out-quart);
  backface-visibility: hidden;
}

.toggle-circle.left {
  transform: scale(1);
}

.toggle-input:checked + .toggle-svg .toggle-circle.left {
  transform: scale(0);
}

.toggle-circle.right {
  transform: scale(0);
}

.toggle-input:checked + .toggle-svg .toggle-circle.right {
  transform: scale(1);
}

.toggle-icon {
  transition: fill 0.35s ease;
}

.toggle-icon.on {
  fill: var(--toggle-inactive);
}

.toggle-input:checked + .toggle-svg .toggle-icon.on {
  fill: #ffffff;
}

.toggle-icon.off {
  fill: var(--toggle-off-icon);
}

.toggle-input:checked + .toggle-svg .toggle-icon.off {
  fill: var(--toggle-active);
}
</style>
