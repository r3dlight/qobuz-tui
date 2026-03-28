<template>
  <div class="player-bar">
    <div class="now-playing">
      <div class="track-info" v-if="state.title">
        <div class="title">{{ state.title }}</div>
        <div class="artist">{{ state.artist || '' }}
          <span class="quality" v-if="state.quality">[{{ state.quality }}]</span>
          <span class="queue-info" v-if="state.queue_len > 1">{{ state.queue_index + 1 }}/{{ state.queue_len }}</span>
          <span class="loop-badge" v-if="state.loop_mode === 1">LOOP:TRACK</span>
          <span class="loop-badge" v-if="state.loop_mode === 2">LOOP:ALL</span>
        </div>
      </div>
      <div class="track-info empty" v-else>Nothing playing</div>
    </div>

    <div class="controls">
      <button @click="previous" title="Previous">⏮</button>
      <button @click="togglePause" class="play-btn" :title="state.is_playing ? 'Pause' : 'Play'">
        {{ state.is_loading ? '◌' : state.is_playing ? '⏸' : '▶' }}
      </button>
      <button @click="next" title="Next">⏭</button>
      <button @click="shuffle" title="Shuffle">🔀</button>
      <button @click="toggleLoop" title="Loop" :class="{ active: state.loop_mode > 0 }">🔁</button>
    </div>

    <div class="progress-section" v-if="state.title">
      <span class="time">{{ formatTime(state.elapsed) }}</span>
      <div class="progress-bar" @click="onSeek">
        <div class="progress-fill" :style="{ width: (state.progress * 100) + '%' }"></div>
      </div>
      <span class="time">{{ formatTime(state.duration) }}</span>
    </div>

    <div class="volume-section">
      <span class="vol-icon">♪</span>
      <input type="range" min="0" max="100" :value="Math.round(state.volume * 100)"
        @input="onVolume" class="volume-slider" />
      <span class="vol-pct">{{ Math.round(state.volume * 100) }}%</span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const state = ref({
  is_playing: false,
  is_loading: false,
  title: null,
  artist: null,
  duration: 0,
  elapsed: 0,
  progress: 0,
  volume: 0.8,
  quality: null,
  seekable: false,
  queue_len: 0,
  queue_index: 0,
  loop_mode: 0,
})

let pollInterval = null

onMounted(() => {
  pollInterval = setInterval(pollState, 500)
})

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval)
})

async function pollState() {
  try {
    state.value = await invoke('get_player_state')
  } catch (e) { /* ignore poll errors */ }
}

async function togglePause() { await invoke('pause') }
async function next() { await invoke('next_track') }
async function previous() { await invoke('previous_track') }
async function shuffle() { await invoke('shuffle_queue') }

async function toggleLoop() {
  state.value.loop_mode = await invoke('toggle_loop')
}

async function onVolume(e) {
  const vol = parseInt(e.target.value) / 100
  await invoke('set_volume', { volume: vol })
}

async function onSeek(e) {
  if (!state.value.seekable || !state.value.duration) return
  const rect = e.currentTarget.getBoundingClientRect()
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  const positionSecs = Math.floor(ratio * state.value.duration)
  await invoke('seek', { positionSecs })
}

function formatTime(s) {
  if (!s) return '0:00'
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}
</script>

<style scoped>
.player-bar {
  display: grid;
  grid-template-columns: 1fr auto 2fr auto;
  align-items: center;
  gap: 1rem;
  padding: 0.8rem 1.5rem;
  background: #191924;
  border-top: 1px solid #2a2a3a;
}
.now-playing { min-width: 200px; }
.title { font-weight: 600; font-size: 0.9rem; color: #e6e6f0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.artist { font-size: 0.8rem; color: #9696a8; }
.quality { color: #00d2d3; margin-left: 0.5rem; }
.queue-info { color: #78788c; margin-left: 0.5rem; }
.loop-badge { color: #aa78ff; margin-left: 0.5rem; font-size: 0.75rem; }
.empty { color: #46465a; }

.controls { display: flex; gap: 0.3rem; align-items: center; }
.controls button {
  background: none;
  border: none;
  color: #9696a8;
  font-size: 1.1rem;
  cursor: pointer;
  padding: 0.4rem;
  border-radius: 4px;
}
.controls button:hover { color: #e6e6f0; background: #2a2a3a; }
.controls button.active { color: #aa78ff; }
.play-btn { font-size: 1.4rem !important; }

.progress-section { display: flex; align-items: center; gap: 0.5rem; }
.time { font-size: 0.75rem; color: #78788c; min-width: 35px; }
.progress-bar {
  flex: 1;
  height: 4px;
  background: #2a2a3a;
  border-radius: 2px;
  cursor: pointer;
  position: relative;
}
.progress-bar:hover { height: 6px; }
.progress-fill {
  height: 100%;
  background: #00d2d3;
  border-radius: 2px;
  transition: width 0.3s linear;
}

.volume-section { display: flex; align-items: center; gap: 0.4rem; min-width: 140px; }
.vol-icon { color: #78788c; }
.vol-pct { font-size: 0.75rem; color: #78788c; min-width: 30px; }
.volume-slider {
  -webkit-appearance: none;
  width: 80px;
  height: 4px;
  background: #2a2a3a;
  border-radius: 2px;
  outline: none;
}
.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  background: #00d2d3;
  border-radius: 50%;
  cursor: pointer;
}
</style>
