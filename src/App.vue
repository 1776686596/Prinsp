<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { useScreenshot } from './composables/useScreenshot'
import { useAnnotation } from './composables/useAnnotation'
import RegionSelector from './components/RegionSelector.vue'
import AnnotationEditor from './components/AnnotationEditor.vue'
import Toolbar from './components/Toolbar.vue'
import Settings from './components/Settings.vue'
import type { Selection } from './types'

type Mode = 'idle' | 'selecting' | 'editing'

const TOOLBAR_WIDTH = 580
const TOOLBAR_HEIGHT = 48
const TOOLBAR_GAP = 10
const OCR_WIDTH = 400
const OCR_HEIGHT = 200
const OCR_GAP = 10

const mode = ref<Mode>('idle')
const selection = ref<Selection>({ x: 0, y: 0, width: 0, height: 0 })
const croppedImage = ref('')
const showSettings = ref(false)
const shortcut = ref(localStorage.getItem('shortcut') || 'Ctrl+Shift+A')
const ocrResult = ref('')
const showOcrResult = ref(false)
const ocrLoading = ref(false)

const { screenshotData, copyToClipboard } = useScreenshot()
const { annotations, currentTool, currentColor, currentLineWidth, addAnnotation, undo, redo, clear, canUndo, canRedo } = useAnnotation()

const toolbarPosition = computed(() => {
  const screenW = window.innerWidth
  const screenH = window.innerHeight
  const sel = selection.value
  const bottomSpace = screenH - (sel.y + sel.height)
  const topSpace = sel.y

  let top: number
  if (bottomSpace >= TOOLBAR_HEIGHT + TOOLBAR_GAP) {
    top = sel.y + sel.height + TOOLBAR_GAP
  } else if (topSpace >= TOOLBAR_HEIGHT + TOOLBAR_GAP) {
    top = sel.y - TOOLBAR_HEIGHT - TOOLBAR_GAP
  } else {
    top = Math.max(0, Math.min(sel.y + sel.height + TOOLBAR_GAP, screenH - TOOLBAR_HEIGHT))
  }

  const margin = 8

  // 判断选区是否靠近右边缘（右侧空间不足放下工具栏）
  if (screenW - sel.x < TOOLBAR_WIDTH + margin) {
    // 使用 right 定位，贴着屏幕右边缘
    return { right: `${margin}px`, left: 'auto', top: `${top}px` }
  }

  // 正常情况，左对齐到选区
  let left = sel.x
  if (left < margin) left = margin

  return { left: `${left}px`, right: 'auto', top: `${top}px` }
})

const ocrPosition = computed(() => {
  const screenW = window.innerWidth
  const screenH = window.innerHeight
  const sel = selection.value
  const toolbarTop = parseFloat(toolbarPosition.value.top)
  const isToolbarBelow = toolbarTop > sel.y

  let top: number
  if (isToolbarBelow) {
    top = toolbarTop + TOOLBAR_HEIGHT + OCR_GAP
  } else {
    top = sel.y + sel.height + TOOLBAR_GAP
  }
  if (top + OCR_HEIGHT > screenH) {
    top = Math.max(0, screenH - OCR_HEIGHT)
  }

  const margin = 8

  // 与工具栏相同的左右定位逻辑
  if (screenW - sel.x < OCR_WIDTH + margin) {
    return { right: `${margin}px`, left: 'auto', top: `${top}px` }
  }

  let left = sel.x
  if (left < margin) left = margin

  return { left: `${left}px`, right: 'auto', top: `${top}px` }
})

async function startCapture() {
  // 隐藏窗口并截屏
  screenshotData.value = await invoke<string>('capture_screen_hidden')
  // 全屏显示窗口
  await invoke('show_window_fullscreen')
  mode.value = 'selecting'
}

function onFullscreen() {
  const img = new Image()
  img.onload = () => {
    selection.value = { x: 0, y: 0, width: img.width, height: img.height }
    croppedImage.value = screenshotData.value
    mode.value = 'editing'
  }
  img.src = 'data:image/png;base64,' + screenshotData.value
}

function onSelect(sel: Selection) {
  selection.value = sel
  cropImage()
  mode.value = 'editing'
}

function cropImage() {
  const img = new Image()
  img.onload = () => {
    const canvas = document.createElement('canvas')
    const s = selection.value
    canvas.width = s.width
    canvas.height = s.height
    const ctx = canvas.getContext('2d')!
    ctx.drawImage(img, s.x, s.y, s.width, s.height, 0, 0, s.width, s.height)
    croppedImage.value = canvas.toDataURL('image/png').split(',')[1]
  }
  img.src = 'data:image/png;base64,' + screenshotData.value
}

async function confirm() {
  await invoke('restore_window')
  const canvas = document.createElement('canvas')
  canvas.width = selection.value.width
  canvas.height = selection.value.height
  const ctx = canvas.getContext('2d')!

  const img = new Image()
  img.src = 'data:image/png;base64,' + croppedImage.value
  await new Promise(r => img.onload = r)
  ctx.drawImage(img, 0, 0)

  annotations.value.forEach(a => {
    ctx.strokeStyle = a.style.color
    ctx.fillStyle = a.style.color
    ctx.lineWidth = a.style.lineWidth

    if (a.type === 'rect' && a.points.length >= 2) {
      const [p1, p2] = a.points
      ctx.strokeRect(p1.x, p1.y, p2.x - p1.x, p2.y - p1.y)
    } else if (a.type === 'arrow' && a.points.length >= 2) {
      const [from, to] = a.points
      const headLen = 15
      const angle = Math.atan2(to.y - from.y, to.x - from.x)
      ctx.beginPath()
      ctx.moveTo(from.x, from.y)
      ctx.lineTo(to.x, to.y)
      ctx.stroke()
      ctx.beginPath()
      ctx.moveTo(to.x, to.y)
      ctx.lineTo(to.x - headLen * Math.cos(angle - Math.PI / 6), to.y - headLen * Math.sin(angle - Math.PI / 6))
      ctx.lineTo(to.x - headLen * Math.cos(angle + Math.PI / 6), to.y - headLen * Math.sin(angle + Math.PI / 6))
      ctx.closePath()
      ctx.fill()
    } else if (a.type === 'text' && a.text) {
      ctx.font = `${a.style.fontSize || 16}px sans-serif`
      ctx.fillText(a.text, a.points[0].x, a.points[0].y)
    } else if (a.type === 'mosaic' && a.points.length >= 2) {
      const [from, to] = a.points
      const blockSize = 10
      const x = Math.min(from.x, to.x)
      const y = Math.min(from.y, to.y)
      const w = Math.abs(to.x - from.x)
      const h = Math.abs(to.y - from.y)
      for (let i = 0; i < w; i += blockSize) {
        for (let j = 0; j < h; j += blockSize) {
          const gray = Math.floor(Math.random() * 100) + 100
          ctx.fillStyle = `rgb(${gray},${gray},${gray})`
          ctx.fillRect(x + i, y + j, blockSize, blockSize)
        }
      }
    }
  })

  const finalData = canvas.toDataURL('image/png').split(',')[1]
  await copyToClipboard(finalData)

  const savePath = await save({
    defaultPath: `screenshot_${Date.now()}.png`,
    filters: [{ name: 'PNG', extensions: ['png'] }]
  })

  if (savePath) {
    try {
      await invoke('save_image_to_file', { base64Data: finalData, path: savePath })
    } catch (e) {
      console.error('Save error:', e)
    }
  }

  reset()
  await invoke('hide_window')
}

async function cancel() {
  await invoke('restore_window')
  reset()
  await invoke('hide_window')
}

async function performOcr() {
  if (ocrLoading.value) return
  ocrLoading.value = true
  try {
    const text = await invoke<string>('ocr_image', { base64Data: croppedImage.value })
    ocrResult.value = text || '(未识别到文字)'
    showOcrResult.value = true
    if (text) await invoke('copy_text_to_clipboard', { text })
  } catch (e) {
    ocrResult.value = '识别失败: ' + e
    showOcrResult.value = true
  } finally {
    ocrLoading.value = false
  }
}

function reset() {
  mode.value = 'idle'
  clear()
  screenshotData.value = ''
  croppedImage.value = ''
  ocrResult.value = ''
  showOcrResult.value = false
}

function saveShortcut(newShortcut: string) {
  shortcut.value = newShortcut
  localStorage.setItem('shortcut', newShortcut)
  applyGlobalShortcut()
}

onMounted(async () => {
  await applyGlobalShortcut()

  await listen('start-capture', () => {
    startCapture()
  })

  await listen('open-settings', () => {
    showSettings.value = true
  })

  // ESC 键取消截图
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && (mode.value === 'selecting' || mode.value === 'editing')) {
      cancel()
    }
  })
})

async function applyGlobalShortcut() {
  try {
    const normalized = shortcut.value
      .split('+')
      .map(p => p.trim())
      .filter(Boolean)
      .join('+')
    await invoke('register_global_shortcut', { shortcut: normalized })
  } catch (e) {
    alert('注册全局快捷键失败: ' + e)
  }
}
</script>

<template>
  <div class="app">
    <div v-if="mode === 'idle'" class="idle">
      <h2>PrinSp 截图工具</h2>
      <p>左键点击托盘图标或按 {{ shortcut }} 开始截图</p>
      <div class="buttons">
        <button @click="startCapture">开始截图</button>
        <button class="settings-btn" @click="showSettings = true">⚙ 设置</button>
      </div>
    </div>

    <RegionSelector
      v-if="mode === 'selecting'"
      :image-data="screenshotData"
      @select="onSelect"
      @fullscreen="onFullscreen"
      @cancel="cancel"
    />

    <div v-if="mode === 'editing'" class="editing">
      <img :src="'data:image/png;base64,' + screenshotData" class="full-bg" />
      <div class="dim-overlay">
        <div class="selection-highlight" :style="{
          left: selection.x + 'px',
          top: selection.y + 'px',
          width: selection.width + 'px',
          height: selection.height + 'px'
        }"></div>
      </div>
      <div class="editor-container" :style="{ left: selection.x + 'px', top: selection.y + 'px' }">
        <AnnotationEditor
          :image-data="croppedImage"
          :selection="{ x: 0, y: 0, width: selection.width, height: selection.height }"
          :annotations="annotations"
          :current-tool="currentTool"
          :current-color="currentColor"
          :line-width="currentLineWidth"
          @add-annotation="addAnnotation"
        />
      </div>
      <div class="toolbar-container" :style="toolbarPosition">
        <Toolbar
          :current-tool="currentTool"
          :current-color="currentColor"
          :can-undo="canUndo"
          :can-redo="canRedo"
          @update:current-tool="currentTool = $event"
          @update:current-color="currentColor = $event"
          @undo="undo"
          @redo="redo"
          @confirm="confirm"
          @cancel="cancel"
          @ocr="performOcr"
        />
      </div>
      <div v-if="ocrLoading" class="ocr-result" :style="ocrPosition">
        <span>识别中...</span>
      </div>
      <div v-else-if="showOcrResult" class="ocr-result" :style="ocrPosition">
        <div class="ocr-header">
          <span>识别结果</span>
          <button @click="showOcrResult = false">✕</button>
        </div>
        <pre>{{ ocrResult }}</pre>
      </div>
    </div>

    <Settings
      v-if="showSettings"
      :current-shortcut="shortcut"
      @close="showSettings = false"
      @save="saveShortcut"
    />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  overflow: hidden;
}

.app {
  width: 100vw;
  height: 100vh;
}

.idle {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  background: #1a1a1a;
  color: #fff;
}

.buttons {
  display: flex;
  gap: 12px;
}

.idle button {
  padding: 12px 24px;
  font-size: 16px;
  border: none;
  border-radius: 8px;
  background: #4a9eff;
  color: #fff;
  cursor: pointer;
}

.idle button:hover {
  background: #3a8eef;
}

.settings-btn {
  background: #444 !important;
}

.settings-btn:hover {
  background: #555 !important;
}

.editing {
  position: fixed;
  inset: 0;
}

.full-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.dim-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
}

.selection-highlight {
  position: absolute;
  background: transparent;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.5);
}

.editor-container {
  position: absolute;
  border: 2px solid #4a9eff;
}

.toolbar-container {
  position: absolute;
  max-width: calc(100vw - 16px);
}

.ocr-result {
  position: absolute;
  max-width: 400px;
  max-height: 200px;
  background: rgba(40, 40, 40, 0.95);
  border-radius: 8px;
  padding: 8px;
  color: #fff;
  overflow: auto;
}

.ocr-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.ocr-header button {
  background: none;
  border: none;
  color: #ff6b6b;
  cursor: pointer;
}

.ocr-result pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
