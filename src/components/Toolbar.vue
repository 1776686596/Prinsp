<script setup lang="ts">
import type { ToolType } from '../types'

const props = defineProps<{
  currentTool: ToolType
  currentColor: string
  canUndo: boolean
  canRedo: boolean
}>()

const emit = defineEmits<{
  'update:currentTool': [tool: ToolType]
  'update:currentColor': [color: string]
  'undo': []
  'redo': []
  'confirm': []
  'cancel': []
  'ocr': []
}>()

const tools: { type: ToolType; icon: string; label: string }[] = [
  { type: 'rect', icon: '▢', label: '矩形' },
  { type: 'arrow', icon: '→', label: '箭头' },
  { type: 'text', icon: 'T', label: '文字' },
  { type: 'mosaic', icon: '▦', label: '马赛克' },
]

const colors = ['#ff0000', '#00ff00', '#0000ff', '#ffff00', '#ff00ff', '#000000', '#ffffff']
</script>

<template>
  <div class="toolbar">
    <div class="tool-group">
      <button
        v-for="tool in tools"
        :key="tool.type"
        :class="{ active: currentTool === tool.type }"
        :title="tool.label"
        @click="emit('update:currentTool', tool.type)"
      >
        {{ tool.icon }}
      </button>
    </div>
    <div class="divider" />
    <div class="color-group">
      <button
        v-for="color in colors"
        :key="color"
        class="color-btn"
        :class="{ active: currentColor === color }"
        :style="{ backgroundColor: color }"
        @click="emit('update:currentColor', color)"
      />
    </div>
    <div class="divider" />
    <div class="action-group">
      <button :disabled="!canUndo" title="撤销" @click="emit('undo')">↩</button>
      <button :disabled="!canRedo" title="重做" @click="emit('redo')">↪</button>
    </div>
    <div class="divider" />
    <div class="ocr-group">
      <button class="ocr" title="文字识别 (OCR)" @click="emit('ocr')">📝</button>
    </div>
    <div class="divider" />
    <div class="confirm-group">
      <button class="cancel" title="取消" @click="emit('cancel')">✕</button>
      <button class="confirm" title="完成" @click="emit('confirm')">✓</button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(40, 40, 40, 0.95);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tool-group, .color-group, .action-group, .confirm-group, .ocr-group {
  display: flex;
  gap: 4px;
}

button {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: #fff;
  font-size: 16px;
  cursor: pointer;
  transition: background 0.2s;
}

button:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
}

button.active {
  background: rgba(255, 255, 255, 0.2);
}

button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.color-btn {
  width: 24px;
  height: 24px;
  border: 2px solid transparent;
  border-radius: 50%;
}

.color-btn.active {
  border-color: #fff;
}

.divider {
  width: 1px;
  height: 24px;
  background: rgba(255, 255, 255, 0.2);
}

.cancel { color: #ff6b6b; }
.confirm { color: #51cf66; }
.ocr { color: #ffd43b; }
</style>