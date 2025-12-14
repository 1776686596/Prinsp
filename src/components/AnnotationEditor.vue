<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import type { Annotation, ToolType, Point, Selection } from '../types'

const props = defineProps<{
  imageData: string
  selection: Selection
  annotations: Annotation[]
  currentTool: ToolType
  currentColor: string
  lineWidth: number
}>()

const emit = defineEmits<{
  'add-annotation': [annotation: Annotation]
}>()

const canvas = ref<HTMLCanvasElement>()
const ctx = ref<CanvasRenderingContext2D>()
const isDrawing = ref(false)
const startPoint = ref<Point>({ x: 0, y: 0 })
const currentPoints = ref<Point[]>([])
const textInput = ref('')
const textPosition = ref<Point | null>(null)

onMounted(() => {
  if (canvas.value) {
    ctx.value = canvas.value.getContext('2d')!
    redraw()
  }
})

watch(() => props.annotations, redraw, { deep: true })

function redraw() {
  if (!ctx.value || !canvas.value) return
  const c = ctx.value
  c.clearRect(0, 0, canvas.value.width, canvas.value.height)
  
  props.annotations.forEach(a => drawAnnotation(a))
}

function drawAnnotation(a: Annotation) {
  if (!ctx.value) return
  const c = ctx.value
  c.strokeStyle = a.style.color
  c.fillStyle = a.style.color
  c.lineWidth = a.style.lineWidth

  switch (a.type) {
    case 'rect':
      if (a.points.length >= 2) {
        const [p1, p2] = a.points
        c.strokeRect(p1.x, p1.y, p2.x - p1.x, p2.y - p1.y)
      }
      break
    case 'arrow':
      if (a.points.length >= 2) {
        drawArrow(c, a.points[0], a.points[1])
      }
      break
    case 'text':
      if (a.points.length >= 1 && a.text) {
        c.font = `${a.style.fontSize || 16}px sans-serif`
        c.fillText(a.text, a.points[0].x, a.points[0].y)
      }
      break
    case 'mosaic':
      if (a.points.length >= 2) {
        drawMosaic(c, a.points[0], a.points[1])
      }
      break
  }
}

function drawArrow(c: CanvasRenderingContext2D, from: Point, to: Point) {
  const headLen = 15
  const angle = Math.atan2(to.y - from.y, to.x - from.x)
  
  c.beginPath()
  c.moveTo(from.x, from.y)
  c.lineTo(to.x, to.y)
  c.stroke()
  
  c.beginPath()
  c.moveTo(to.x, to.y)
  c.lineTo(to.x - headLen * Math.cos(angle - Math.PI / 6), to.y - headLen * Math.sin(angle - Math.PI / 6))
  c.lineTo(to.x - headLen * Math.cos(angle + Math.PI / 6), to.y - headLen * Math.sin(angle + Math.PI / 6))
  c.closePath()
  c.fill()
}

function drawMosaic(c: CanvasRenderingContext2D, from: Point, to: Point) {
  const blockSize = 10
  const x = Math.min(from.x, to.x)
  const y = Math.min(from.y, to.y)
  const w = Math.abs(to.x - from.x)
  const h = Math.abs(to.y - from.y)
  
  for (let i = 0; i < w; i += blockSize) {
    for (let j = 0; j < h; j += blockSize) {
      const gray = Math.floor(Math.random() * 100) + 100
      c.fillStyle = `rgb(${gray},${gray},${gray})`
      c.fillRect(x + i, y + j, blockSize, blockSize)
    }
  }
}

function getPos(e: MouseEvent): Point {
  const rect = canvas.value!.getBoundingClientRect()
  return { x: e.clientX - rect.left, y: e.clientY - rect.top }
}

function onMouseDown(e: MouseEvent) {
  if (props.currentTool === 'select') return
  if (props.currentTool === 'text') {
    textPosition.value = getPos(e)
    return
  }
  isDrawing.value = true
  startPoint.value = getPos(e)
  currentPoints.value = [startPoint.value]
}

function onMouseMove(e: MouseEvent) {
  if (!isDrawing.value || !ctx.value) return
  const pos = getPos(e)
  currentPoints.value = [startPoint.value, pos]
  
  redraw()
  drawAnnotation({
    id: 'temp',
    type: props.currentTool,
    points: currentPoints.value,
    style: { color: props.currentColor, lineWidth: props.lineWidth }
  })
}

function onMouseUp() {
  if (!isDrawing.value) return
  isDrawing.value = false
  
  if (currentPoints.value.length >= 2) {
    emit('add-annotation', {
      id: Date.now().toString(),
      type: props.currentTool,
      points: [...currentPoints.value],
      style: { color: props.currentColor, lineWidth: props.lineWidth }
    })
  }
  currentPoints.value = []
}

function submitText() {
  if (textPosition.value && textInput.value) {
    emit('add-annotation', {
      id: Date.now().toString(),
      type: 'text',
      points: [textPosition.value],
      style: { color: props.currentColor, lineWidth: props.lineWidth, fontSize: 16 },
      text: textInput.value
    })
    textInput.value = ''
    textPosition.value = null
  }
}
</script>

<template>
  <div class="editor">
    <canvas
      ref="canvas"
      :width="selection.width"
      :height="selection.height"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
    />
    <div v-if="textPosition" class="text-input" :style="{ left: textPosition.x + 'px', top: textPosition.y + 'px' }">
      <input
        v-model="textInput"
        autofocus
        placeholder="输入文字..."
        @keyup.enter="submitText"
        @keyup.esc="textPosition = null"
      />
    </div>
  </div>
</template>

<style scoped>
.editor {
  position: relative;
}

canvas {
  cursor: crosshair;
}

.text-input {
  position: absolute;
  z-index: 10;
}

.text-input input {
  padding: 4px 8px;
  border: 1px solid #666;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.8);
  color: #fff;
  outline: none;
}
</style>