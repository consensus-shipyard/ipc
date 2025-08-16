/**
 * Clipboard utilities
 */

/**
 * Copy text to clipboard with fallback support for older browsers
 * @param text - The text to copy
 * @returns Promise that resolves to true if successful
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    // Try modern clipboard API first
    await navigator.clipboard.writeText(text)
    return true
  } catch (err) {
    console.error('Failed to copy to clipboard:', err)

    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = text
    textArea.style.position = 'fixed'
    textArea.style.top = '0'
    textArea.style.left = '0'
    textArea.style.width = '2em'
    textArea.style.height = '2em'
    textArea.style.padding = '0'
    textArea.style.border = 'none'
    textArea.style.outline = 'none'
    textArea.style.boxShadow = 'none'
    textArea.style.background = 'transparent'

    document.body.appendChild(textArea)
    textArea.focus()
    textArea.select()

    try {
      const successful = document.execCommand('copy')
      document.body.removeChild(textArea)
      return successful
    } catch (fallbackErr) {
      console.error('Fallback copy failed:', fallbackErr)
      document.body.removeChild(textArea)
      return false
    }
  }
}

/**
 * Composable-style clipboard handler with feedback
 */
export function useClipboard() {
  const copyingItem = ref<string | null>(null)
  const copyTimeout = ref<number | null>(null)

  const copy = async (text: string, itemId: string = 'default'): Promise<boolean> => {
    const success = await copyToClipboard(text)

    if (success) {
      copyingItem.value = itemId

      // Clear any existing timeout
      if (copyTimeout.value !== null) {
        clearTimeout(copyTimeout.value)
      }

      // Reset after 2 seconds
      copyTimeout.value = window.setTimeout(() => {
        copyingItem.value = null
        copyTimeout.value = null
      }, 2000)
    }

    return success
  }

  const isCopying = (itemId: string = 'default'): boolean => {
    return copyingItem.value === itemId
  }

  // Cleanup on unmount
  onUnmounted(() => {
    if (copyTimeout.value !== null) {
      clearTimeout(copyTimeout.value)
    }
  })

  return {
    copy,
    isCopying,
    copyingItem: readonly(copyingItem)
  }
}

// Import Vue's reactivity functions if using the composable
import { ref, readonly, onUnmounted } from 'vue'
