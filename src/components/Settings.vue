<script setup lang="ts">
import { ref, onMounted } from 'vue'

const emit = defineEmits<{
  'close': []
  'save': [shortcut: string]
}>()

const props = defineProps<{
  currentShortcut: string
}>()

const shortcut = ref(props.currentShortcut)
const recording = ref(false)
const keys = ref<string[]>([])

function startRecording() {
  recording.value = true
  keys.value = []
}

function onKeyDown(e: KeyboardEvent) {
  if (!recording.value) return
  e.preventDefault()
  
  const key = []
  if (e.ctrlKey) key.push('Ctrl')
  if (e.shiftKey) key.push('Shift')
  if (e.altKey) key.push('Alt')
  if (e.metaKey) key.push('Super')
  
  if (e.key !== 'Control' && e.key !== 'Shift' && e.key !== 'Alt' && e.key !== 'Meta') {
    key.push(e.key.toUpperCase())
  }
  
  if (key.length > 1) {
    shortcut.value = key.join('+')
    recording.value = false
  }
}

function save() {
  emit('save', shortcut.value)
  emit('close')
}
</script>

<template>
  <div class="settings-overlay" @click.self="emit('close')">
    <div class="settings-panel">
      <h3>设置</h3>
      
      <div class="setting-item">
        <label>截图快捷键</label>
        <div class="shortcut-input">
          <input
            :value="recording ? '请按下快捷键...' : shortcut"
            readonly
            @click="startRecording"
            @keydown="onKeyDown"
            :class="{ recording }"
          />
          <button @click="startRecording">修改</button>
        </div>
      </div>
      
      <div class="actions">
        <button class="cancel" @click="emit('close')">取消</button>
        <button class="save" @click="save">保存</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-panel {
  background: #2a2a2a;
  border-radius: 12px;
  padding: 24px;
  min-width: 360px;
  color: #fff;
}

h3 {
  margin: 0 0 20px;
  font-size: 18px;
}

.setting-item {
  margin-bottom: 20px;
}

.setting-item label {
  display: block;
  margin-bottom: 8px;
  color: #aaa;
  font-size: 14px;
}

.shortcut-input {
  display: flex;
  gap: 8px;
}

.shortcut-input input {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid #444;
  border-radius: 6px;
  background: #1a1a1a;
  color: #fff;
  font-size: 14px;
  cursor: pointer;
}

.shortcut-input input.recording {
  border-color: #4a9eff;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.shortcut-input button {
  padding: 10px 16px;
  border: none;
  border-radius: 6px;
  background: #444;
  color: #fff;
  cursor: pointer;
}

.shortcut-input button:hover {
  background: #555;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.actions button {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.cancel {
  background: #444;
  color: #fff;
}

.save {
  background: #4a9eff;
  color: #fff;
}

.cancel:hover { background: #555; }
.save:hover { background: #3a8eef; }
</style>