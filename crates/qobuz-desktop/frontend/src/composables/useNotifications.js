import { watch } from 'vue'

let lastTitle = ''

/**
 * Send a system notification when the track changes.
 * Uses the Notification API with graceful fallback if unavailable.
 */
export function useTrackNotifications(playerState) {
  watch(() => playerState.value?.title, (title) => {
    if (!title || title === lastTitle) return
    lastTitle = title
    try {
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(title, { body: playerState.value?.artist || '' })
      } else if ('Notification' in window && Notification.permission !== 'denied') {
        Notification.requestPermission().then(p => {
          if (p === 'granted') {
            new Notification(title, { body: playerState.value?.artist || '' })
          }
        })
      }
    } catch (_) {}
  })
}
