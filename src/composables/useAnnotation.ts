import { ref, computed } from 'vue'
import type { Annotation, ToolType } from '../types'

export function useAnnotation() {
  const annotations = ref<Annotation[]>([])
  const history = ref<Annotation[][]>([[]])
  const historyIndex = ref(0)
  const currentTool = ref<ToolType>('select')
  const currentColor = ref('#ff0000')
  const currentLineWidth = ref(3)

  function addAnnotation(annotation: Annotation) {
    annotations.value.push(annotation)
    saveHistory()
  }

  function saveHistory() {
    history.value = history.value.slice(0, historyIndex.value + 1)
    history.value.push([...annotations.value])
    historyIndex.value = history.value.length - 1
  }

  function undo() {
    if (historyIndex.value > 0) {
      historyIndex.value--
      annotations.value = [...history.value[historyIndex.value]]
    }
  }

  function redo() {
    if (historyIndex.value < history.value.length - 1) {
      historyIndex.value++
      annotations.value = [...history.value[historyIndex.value]]
    }
  }

  function clear() {
    annotations.value = []
    saveHistory()
  }

  const canUndo = computed(() => historyIndex.value > 0)
  const canRedo = computed(() => historyIndex.value < history.value.length - 1)

  return {
    annotations,
    currentTool,
    currentColor,
    currentLineWidth,
    addAnnotation,
    undo,
    redo,
    clear,
    canUndo,
    canRedo
  }
}