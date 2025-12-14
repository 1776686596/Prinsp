import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useScreenshot() {
  const screenshotData = ref<string>('')
  const loading = ref(false)
  const error = ref<string>('')

  async function captureScreen() {
    loading.value = true
    error.value = ''
    try {
      console.log('Capturing screen...')
      screenshotData.value = await invoke<string>('capture_screen')
      console.log('Screenshot captured, size:', screenshotData.value.length)
    } catch (e) {
      error.value = String(e)
      console.error('Screenshot error:', e)
      alert('截图失败: ' + e)
    } finally {
      loading.value = false
    }
  }

  async function copyToClipboard(base64Data: string) {
    try {
      await invoke('copy_to_clipboard', { base64Data })
      console.log('Copied to clipboard')
    } catch (e) {
      console.error('Clipboard error:', e)
      alert('复制到剪贴板失败: ' + e)
    }
  }

  return { screenshotData, loading, error, captureScreen, copyToClipboard }
}