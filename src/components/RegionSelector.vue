<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { Selection, Point } from '../types'

const props = defineProps<{
  imageData: string
}>()

const emit = defineEmits<{
  'select': [selection: Selection]
  'fullscreen': []
  'cancel': []
}>()

const isSelecting = ref(false)
const startPoint = ref<Point>({ x: 0, y: 0 })
const endPoint = ref<Point>({ x: 0, y: 0 })
const imgSize = ref({ width: 0, height: 0 })

const selection = computed<Selection>(() => ({
  x: Math.min(startPoint.value.x, endPoint.value.x),
  y: Math.min(startPoint.value.y, endPoint.value.y),
  width: Math.abs(endPoint.value.x - startPoint.value.x),
  height: Math.abs(endPoint.value.y - startPoint.value.y)
}))

const hasSelection = computed(() => selection.value.width > 10 && selection.value.height > 10)

function onMouseDown(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('.capture-toolbar')) return
  isSelecting.value = true
  startPoint.value = { x: e.clientX, y: e.clientY }
  endPoint.value = { x: e.clientX, y: e.clientY }
}

function onMouseMove(e: MouseEvent) {
  if (isSelecting.value) {
    endPoint.value = { x: e.clientX, y: e.clientY }
  }
}

function onMouseUp() {
  if (isSelecting.value && hasSelection.value) {
    emit('select', selection.value)
  }
  isSelecting.value = false
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    emit('cancel')
  }
}

function captureFullscreen() {
  emit('fullscreen')
}

onMounted(() => {
  const img = new Image()
  img.onload = () => {
    imgSize.value = { width: img.width, height: img.height }
  }
  img.src = 'data:image/png;base64,' + props.imageData
})
</script>

<template>
  <div
    class="selector"
    tabindex="0"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseup="onMouseUp"
    @keydown="onKeyDown"
  >
    <img :src="'data:image/png;base64,' + imageData" class="background" />
    <div class="overlay" />
    <div
      v-if="hasSelection"
      class="selection"
      :style="{
        left: selection.x + 'px',
        top: selection.y + 'px',
        width: selection.width + 'px',
        height: selection.height + 'px'
      }"
    >
      <div class="size-info">{{ selection.width }} × {{ selection.height }}</div>
    </div>
    
    <div class="capture-toolbar">
      <div class="toolbar-tip">拖动选择区域，或点击下方按钮</div>
      <div class="toolbar-buttons">
        <button @click="captureFullscreen" class="fullscreen-btn">🖥 全屏截图</button>
        <button @click="emit('cancel')" class="cancel-btn">✕ 取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.selector {
  position: fixed;
  inset: 0;
  cursor: crosshair;
  outline: none;
}

.background {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
}

.selection {
  position: absolute;
  border: 2px solid #4a9eff;
  background: transparent;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
}

.size-info {
  position: absolute;
  top: -24px;
  left: 0;
  padding: 2px 8px;
  background: rgba(0, 0, 0, 0.7);
  color: #fff;
  font-size: 12px;
  border-radius: 4px;
  white-space: nowrap;
}

.capture-toolbar {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(40, 40, 40, 0.95);
  border-radius: 12px;
  padding: 12px 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  z-index: 100;
}

.toolbar-tip {
  color: #aaa;
  font-size: 13px;
}

.toolbar-buttons {
  display: flex;
  gap: 12px;
}

.toolbar-buttons button {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  transition: background 0.2s;
}

.fullscreen-btn {
  background: #4a9eff;
  color: #fff;
}

.fullscreen-btn:hover {
  background: #3a8eef;
}

.cancel-btn {
  background: #555;
  color: #fff;
}

.cancel-btn:hover {
  background: #666;
}
</style>