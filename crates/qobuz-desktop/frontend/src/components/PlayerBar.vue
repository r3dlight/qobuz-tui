<template>
  <div class="player-bar">
    <div class="now-playing">
      <img v-if="state.cover_url" :src="state.cover_url" class="cover-art" alt="" />
      <div class="cover-placeholder" v-else-if="state.title">♫</div>
      <div class="track-info" v-if="state.title">
        <div class="title">{{ state.title }}</div>
        <div class="artist">{{ state.artist || '' }}
          <span class="quality" v-if="state.quality">[{{ state.quality }}]</span>
        </div>
      </div>
      <div class="track-info empty" v-else>Nothing playing</div>
    </div>

    <div class="center-section">
      <div class="controls">
        <button @click="shuffle" title="Shuffle">🔀</button>
        <button @click="previous" title="Previous">⏮</button>
        <button @click="togglePause" class="play-btn" :title="state.is_playing ? 'Pause' : 'Play'">
          {{ state.is_loading ? '◌' : state.is_playing ? '⏸' : '▶' }}
        </button>
        <button @click="next" title="Next">⏭</button>
        <button @click="toggleLoop" title="Loop" :class="{ active: state.loop_mode > 0 }">
          {{ state.loop_mode === 1 ? '🔂' : '🔁' }}
        </button>
      </div>
      <div class="progress-section" v-if="state.title">
        <span class="time">{{ formatTime(state.elapsed) }}</span>
        <div class="progress-bar" @click="onSeek">
          <div class="progress-fill" :style="{ width: (state.progress * 100) + '%' }"></div>
        </div>
        <span class="time">{{ formatTime(state.duration) }}</span>
      </div>
    </div>

    <div class="right-section">
      <div class="queue-info" v-if="state.queue_len > 1">
        {{ state.queue_index + 1 }}/{{ state.queue_len }}
      </div>
      <span class="loop-badge" v-if="state.loop_mode === 1">TRACK</span>
      <span class="loop-badge" v-if="state.loop_mode === 2">ALL</span>
      <div class="volume-section">
        <span class="vol-icon">♪</span>
        <input type="range" min="0" max="100" :value="Math.round(state.volume * 100)"
          @input="onVolume" class="volume-slider" />
        <span class="vol-pct">{{ Math.round(state.volume * 100) }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const state = ref({
  is_playing: false, is_loading: false, title: null, artist: null,
  duration: 0, elapsed: 0, progress: 0, volume: 0.8, quality: null,
  seekable: false, queue_len: 0, queue_index: 0, loop_mode: 0, cover_url: null,
})

let pollInterval = null
onMounted(() => { pollInterval = setInterval(pollState, 500) })
onUnmounted(() => { if (pollInterval) clearInterval(pollInterval) })

async function pollState() {
  try { state.value = await invoke('get_player_state') } catch (_) {}
}
async function togglePause() { await invoke('pause') }
async function next() { try { await invoke('next_track') } catch (_) {} }
async function previous() { try { await invoke('previous_track') } catch (_) {} }
async function shuffle() { await invoke('shuffle_queue') }
async function toggleLoop() { state.value.loop_mode = await invoke('toggle_loop') }
async function onVolume(e) { await invoke('set_volume', { volume: parseInt(e.target.value) / 100 }) }

async function onSeek(e) {
  if (!state.value.seekable || !state.value.duration) return
  const rect = e.currentTarget.getBoundingClientRect()
  const ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  await invoke('seek', { positionSecs: Math.floor(ratio * state.value.duration) })
}

function formatTime(s) {
  if (!s) return '0:00'
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}
</script>

<style scoped>
.player-bar {
  display: grid;
  grid-template-columns: minmax(200px, 1fr) 2fr minmax(180px, 1fr);
  align-items: center;
  gap: 1rem;
  padding: 0.6rem 1.5rem;
  background: #191924;
  border-top: 1px solid #2a2a3a;
  min-height: 80px;
}

/* Left: now playing */
.now-playing { display: flex; align-items: center; gap: 0.8rem; }
.cover-art { width: 50px; height: 50px; border-radius: 4px; object-fit: cover; }
.cover-placeholder {
  width: 50px; height: 50px; background: #2a2a3a; border-radius: 4px;
  display: flex; align-items: center; justify-content: center;
  font-size: 1.2rem; color: #46465a;
}
.title { font-weight: 600; font-size: 0.85rem; color: #e6e6f0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 200px; }
.artist { font-size: 0.75rem; color: #9696a8; }
.quality { color: #00d2d3; margin-left: 0.4rem; }
.empty { color: #46465a; font-size: 0.85rem; }

/* Center: controls + progress */
.center-section { display: flex; flex-direction: column; align-items: center; gap: 0.3rem; }
.controls { display: flex; gap: 0.3rem; align-items: center; }
.controls button {
  background: none; border: none; color: #9696a8; font-size: 1rem;
  cursor: pointer; padding: 0.3rem 0.5rem; border-radius: 4px;
}
.controls button:hover { color: #e6e6f0; background: #2a2a3a; }
.controls button.active { color: #aa78ff; }
.play-btn { font-size: 1.4rem !important; }
.progress-section { display: flex; align-items: center; gap: 0.5rem; width: 100%; }
.time { font-size: 0.7rem; color: #78788c; min-width: 32px; text-align: center; }
.progress-bar {
  flex: 1; height: 4px; background: #2a2a3a; border-radius: 2px;
  cursor: pointer; position: relative;
}
.progress-bar:hover { height: 6px; }
.progress-fill { height: 100%; background: #00d2d3; border-radius: 2px; transition: width 0.3s linear; }

/* Right: queue + volume */
.right-section { display: flex; align-items: center; gap: 0.6rem; justify-content: flex-end; }
.queue-info { font-size: 0.75rem; color: #78788c; }
.loop-badge { font-size: 0.65rem; color: #aa78ff; background: #2a2a3a; padding: 0.15rem 0.4rem; border-radius: 3px; }
.volume-section { display: flex; align-items: center; gap: 0.3rem; }
.vol-icon { color: #78788c; font-size: 0.85rem; }
.vol-pct { font-size: 0.7rem; color: #78788c; min-width: 28px; }
.volume-slider {
  -webkit-appearance: none; width: 70px; height: 4px;
  background: #2a2a3a; border-radius: 2px; outline: none;
}
.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none; width: 10px; height: 10px;
  background: #00d2d3; border-radius: 50%; cursor: pointer;
}
</style>
