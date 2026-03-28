import { onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * Global keyboard shortcuts for the desktop app.
 * @param {Object} callbacks - { onSearch, onToggleQueue, onToggleTheme }
 */
export function useKeyboard(callbacks = {}) {
  function handler(e) {
    // Don't capture when typing in an input
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
      // Except Escape which always works
      if (e.key !== 'Escape') return
    }

    switch (e.key) {
      case ' ':
        e.preventDefault()
        invoke('pause')
        break
      case 'ArrowRight':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          invoke('next_track').catch(() => {})
        }
        break
      case 'ArrowLeft':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          invoke('previous_track').catch(() => {})
        }
        break
      case 'ArrowUp':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          invoke('get_player_state').then(s => {
            invoke('set_volume', { volume: Math.min(1, s.volume + 0.05) })
          })
        }
        break
      case 'ArrowDown':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          invoke('get_player_state').then(s => {
            invoke('set_volume', { volume: Math.max(0, s.volume - 0.05) })
          })
        }
        break
      case 'f':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          callbacks.onSearch?.()
        }
        break
      case 'q':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          callbacks.onToggleQueue?.()
        }
        break
      case 's':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          invoke('shuffle_queue')
        }
        break
      case 'r':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          invoke('toggle_loop')
        }
        break
      case 't':
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault()
          callbacks.onToggleTheme?.()
        }
        break
    }
  }

  onMounted(() => window.addEventListener('keydown', handler))
  onUnmounted(() => window.removeEventListener('keydown', handler))
}
