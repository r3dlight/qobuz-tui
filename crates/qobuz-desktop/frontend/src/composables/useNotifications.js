import { ref, watch } from 'vue'

let lastTitle = ''

/**
 * Send a system notification when the track changes.
 * @param {import('vue').Ref} playerState - reactive player state
 */
export function useTrackNotifications(playerState) {
  watch(() => playerState.value?.title, async (title) => {
    if (!title || title === lastTitle) return
    lastTitle = title
    try {
      const { sendNotification, isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification')
      let permitted = await isPermissionGranted()
      if (!permitted) {
        const result = await requestPermission()
        permitted = result === 'granted'
      }
      if (permitted) {
        sendNotification({
          title: title,
          body: playerState.value?.artist || '',
        })
      }
    } catch (_) {}
  })
}
