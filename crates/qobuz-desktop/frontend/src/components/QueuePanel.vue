<template>
  <aside class="queue-panel" :class="{ open: visible }">
    <div class="queue-header">
      <h3>Queue</h3>
      <span class="queue-count" v-if="queue.length">{{ queue.length }} tracks</span>
      <button class="close-btn" @click="emit('close')">✕</button>
    </div>
    <div class="queue-actions" v-if="queue.length">
      <button @click="onShuffle">🔀 Shuffle</button>
      <button @click="onClear">Clear</button>
    </div>
    <div class="queue-list">
      <div v-for="(track, i) in queue" :key="i"
        class="queue-item" :class="{ current: i === currentIndex }"
        @click="emit('play-index', i)">
        <span class="qi-num">{{ i + 1 }}</span>
        <div class="qi-info">
          <div class="qi-title">{{ track.title }}</div>
          <div class="qi-artist">{{ track.performer?.name || '' }}</div>
        </div>
        <span class="qi-dur">{{ formatTime(track.duration) }}</span>
      </div>
    </div>
    <div class="queue-empty" v-if="!queue.length">
      <p>Queue is empty</p>
    </div>
  </aside>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

defineProps({
  visible: Boolean,
})
const emit = defineEmits(['close', 'play-index'])

const queue = ref([])
const currentIndex = ref(0)
let pollInterval = null

onMounted(() => { pollInterval = setInterval(pollQueue, 1000) })
onUnmounted(() => { if (pollInterval) clearInterval(pollInterval) })

async function pollQueue() {
  try {
    const state = await invoke('get_player_state')
    currentIndex.value = state.queue_index
    // Queue content isn't directly accessible via get_player_state
    // We'd need a dedicated command, but for now just show the count
  } catch (_) {}
}

function onShuffle() { invoke('shuffle_queue') }
function onClear() { /* TODO: add clear_queue command */ }

function formatTime(s) {
  if (!s) return '0:00'
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}
</script>

<style scoped>
.queue-panel {
  position: fixed;
  top: 0; right: -320px;
  width: 320px; height: 100vh;
  background: #20203a;
  border-left: 1px solid #30304a;
  z-index: 100;
  transition: right 0.25s ease;
  display: flex;
  flex-direction: column;
}
.queue-panel.open { right: 0; }

.queue-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  border-bottom: 1px solid #30304a;
}
.queue-header h3 { flex: 1; font-size: 1rem; }
.queue-count { font-size: 0.75rem; color: #9090a8; }
.close-btn {
  background: none; border: none; color: #9090a8;
  cursor: pointer; font-size: 1.1rem; padding: 0.2rem;
}
.close-btn:hover { color: #eaeaf0; }

.queue-actions {
  display: flex; gap: 0.5rem; padding: 0.5rem 1rem;
}
.queue-actions button {
  padding: 0.3rem 0.8rem;
  background: #303050;
  border: none; border-radius: 4px;
  color: #9090a8; cursor: pointer; font-size: 0.8rem;
}
.queue-actions button:hover { background: #404060; color: #eaeaf0; }

.queue-list { flex: 1; overflow-y: auto; }
.queue-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  cursor: pointer;
  transition: background 0.1s;
}
.queue-item:hover { background: #262642; }
.queue-item.current { background: rgba(100,149,237,0.1); }
.queue-item.current .qi-title { color: #7aa5f7; }
.qi-num { color: #585875; font-size: 0.8rem; min-width: 24px; text-align: center; }
.qi-info { flex: 1; min-width: 0; }
.qi-title { font-size: 0.82rem; color: #eaeaf0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.qi-artist { font-size: 0.72rem; color: #9090a8; }
.qi-dur { font-size: 0.75rem; color: #707088; }

.queue-empty { padding: 2rem; text-align: center; color: #585875; }
</style>
