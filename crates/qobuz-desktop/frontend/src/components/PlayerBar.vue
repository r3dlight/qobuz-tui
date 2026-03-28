<template>
  <div class="player-bar" :class="{ active: state.title }">
    <!-- Left: Now playing -->
    <div class="now-playing">
      <img v-if="state.cover_url" :src="state.cover_url" class="cover-art" alt="" />
      <div class="cover-placeholder" v-else-if="state.title">
        <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
          <path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6Z"/>
        </svg>
      </div>
      <div class="track-info" v-if="state.title">
        <div class="title">{{ state.title }}</div>
        <div class="meta">
          <span class="artist">{{ state.artist || '' }}</span>
          <span class="quality-badge" v-if="state.quality">{{ state.quality }}</span>
        </div>
      </div>
      <div class="track-info empty" v-else>
        <div class="title">Nothing playing</div>
        <div class="meta">Select a track to start</div>
      </div>
    </div>

    <!-- Center: Controls + Progress -->
    <div class="center">
      <div class="controls">
        <button class="ctrl-btn small" @click="shuffle" title="Shuffle" :class="{ dimmed: !state.queue_len }">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M10.59 9.17 5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"/></svg>
        </button>
        <button class="ctrl-btn" @click="previous" title="Previous">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6 8.5 6V6z"/></svg>
        </button>
        <button class="ctrl-btn play" @click="togglePause" :title="state.is_playing ? 'Pause' : 'Play'">
          <svg v-if="state.is_loading" viewBox="0 0 24 24" fill="currentColor" class="spin"><path d="M12 4V1L8 5l4 4V6c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z"/></svg>
          <svg v-else-if="state.is_playing" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
        </button>
        <button class="ctrl-btn" @click="next" title="Next">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
        </button>
        <button class="ctrl-btn small" @click="toggleLoop" title="Loop"
          :class="{ active: state.loop_mode > 0 }">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M7 7h10v3l4-4-4-4v3H5v6h2V7zm10 10H7v-3l-4 4 4 4v-3h12v-6h-2v4z"/></svg>
          <span class="loop-dot" v-if="state.loop_mode === 1">1</span>
        </button>
      </div>
      <div class="progress-row" v-if="state.title">
        <span class="time">{{ formatTime(state.elapsed) }}</span>
        <div class="progress-track" @click="onSeek" @mouseenter="showThumb = true" @mouseleave="showThumb = false">
          <div class="progress-fill" :style="{ width: (state.progress * 100) + '%' }">
            <div class="progress-thumb" v-show="showThumb || dragging"></div>
          </div>
        </div>
        <span class="time">{{ formatTime(state.duration) }}</span>
      </div>
    </div>

    <!-- Right: Mini + Queue + Volume -->
    <div class="right">
      <button class="mini-btn" @click="toggleMiniPlayer" title="Mini player">
        <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16"><path d="M19 11h-8v6h8v-6zm4 8V4.98C23 3.88 22.1 3 21 3H3c-1.1 0-2 .88-2 1.98V19c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2zm-2 .02H3V4.97h18v14.05z"/></svg>
      </button>
      <div class="queue-badge" v-if="state.queue_len > 1">
        <svg viewBox="0 0 24 24" fill="currentColor" width="14" height="14"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"/></svg>
        {{ state.queue_index + 1 }} / {{ state.queue_len }}
      </div>
      <span class="loop-label" v-if="state.loop_mode === 1">TRACK</span>
      <span class="loop-label" v-if="state.loop_mode === 2">ALL</span>
      <div class="volume">
        <button class="vol-btn" @click="toggleMute">
          <svg v-if="state.volume === 0" viewBox="0 0 24 24" fill="currentColor"><path d="M16.5 12A4.5 4.5 0 0 0 14 8.18v1.7l2.4 2.4c.06-.27.1-.54.1-.82zm2 0c0 .94-.2 1.82-.54 2.64l1.51 1.51A8.796 8.796 0 0 0 20.5 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3 3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06a8.99 8.99 0 0 0 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4 9.91 6.09 12 8.18V4z"/></svg>
          <svg v-else-if="state.volume < 0.5" viewBox="0 0 24 24" fill="currentColor"><path d="M18.5 12A4.5 4.5 0 0 0 16 8.18v7.64c1.5-.73 2.5-2.25 2.5-3.82zM5 9v6h4l5 5V4L9 9H5z"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="currentColor"><path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0 0 14 8.18v7.64c1.5-.73 2.5-2.25 2.5-3.82zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/></svg>
        </button>
        <input type="range" min="0" max="100" :value="Math.round(state.volume * 100)"
          @input="onVolume" class="volume-slider" />
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
const showThumb = ref(false)
const dragging = ref(false)
let lastVolume = 0.8
let pollInterval = null

onMounted(() => { pollInterval = setInterval(pollState, 500) })
onUnmounted(() => { if (pollInterval) clearInterval(pollInterval) })

async function pollState() {
  try { state.value = await invoke('get_player_state') } catch (_) {}
}
async function togglePause() { await invoke('pause') }
async function toggleMiniPlayer() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const { LogicalSize } = await import('@tauri-apps/api/dpi')
    const win = getCurrentWindow()
    const size = await win.innerSize()
    if (size.width > 400) {
      await win.setSize(new LogicalSize(380, 120))
      await win.setAlwaysOnTop(true)
    } else {
      await win.setSize(new LogicalSize(1200, 800))
      await win.setAlwaysOnTop(false)
    }
  } catch (_) {}
}
async function next() { try { await invoke('next_track') } catch (_) {} }
async function previous() { try { await invoke('previous_track') } catch (_) {} }
async function shuffle() { await invoke('shuffle_queue') }
async function toggleLoop() { state.value.loop_mode = await invoke('toggle_loop') }
async function onVolume(e) { await invoke('set_volume', { volume: parseInt(e.target.value) / 100 }) }
async function toggleMute() {
  if (state.value.volume > 0) {
    lastVolume = state.value.volume
    await invoke('set_volume', { volume: 0 })
  } else {
    await invoke('set_volume', { volume: lastVolume || 0.8 })
  }
}

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
  grid-template-columns: minmax(220px, 1fr) 2fr minmax(200px, 1fr);
  align-items: center;
  gap: 1.5rem;
  padding: 0.7rem 1.5rem;
  background: #20203a;
  border-top: 1px solid #303050;
  min-height: 88px;
}

/* Left */
.now-playing { display: flex; align-items: center; gap: 0.8rem; }
.cover-art { width: 56px; height: 56px; border-radius: 6px; object-fit: cover; box-shadow: 0 2px 8px rgba(0,0,0,0.4); }
.cover-placeholder {
  width: 56px; height: 56px; background: #303050; border-radius: 6px;
  display: flex; align-items: center; justify-content: center; color: #585875;
}
.title { font-weight: 600; font-size: 0.9rem; color: #eaeaf0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 220px; }
.meta { display: flex; align-items: center; gap: 0.5rem; }
.artist { font-size: 0.8rem; color: #a8a8be; }
.quality-badge { font-size: 0.65rem; color: #00d2d3; background: rgba(0,210,211,0.1); padding: 0.1rem 0.4rem; border-radius: 3px; }
.empty .title { color: #707088; }
.empty .meta { color: #4a4a65; font-size: 0.8rem; }

/* Center */
.center { display: flex; flex-direction: column; align-items: center; gap: 0.4rem; }
.controls { display: flex; gap: 0.2rem; align-items: center; }
.ctrl-btn {
  background: none; border: none; color: #a8a8be; cursor: pointer;
  width: 36px; height: 36px; border-radius: 50%; display: flex;
  align-items: center; justify-content: center; transition: all 0.15s;
}
.ctrl-btn svg { width: 20px; height: 20px; }
.ctrl-btn:hover { color: #eaeaf0; background: rgba(255,255,255,0.05); }
.ctrl-btn.small { width: 30px; height: 30px; }
.ctrl-btn.small svg { width: 16px; height: 16px; }
.ctrl-btn.active { color: #00d2d3; }
.ctrl-btn.dimmed { opacity: 0.3; }
.ctrl-btn.play {
  width: 42px; height: 42px; background: #eaeaf0; color: #1a1a2e;
  margin: 0 0.3rem;
}
.ctrl-btn.play:hover { background: #fff; transform: scale(1.05); }
.ctrl-btn.play svg { width: 22px; height: 22px; }
.loop-dot {
  position: absolute; font-size: 0.5rem; font-weight: 700;
  bottom: 2px; right: 2px; color: #00d2d3;
}
.ctrl-btn.small { position: relative; }

.progress-row { display: flex; align-items: center; gap: 0.5rem; width: 100%; max-width: 600px; }
.time { font-size: 0.7rem; color: #9090a8; min-width: 35px; text-align: center; font-variant-numeric: tabular-nums; }
.progress-track {
  flex: 1; height: 4px; background: #353555; border-radius: 4px;
  cursor: pointer; position: relative; transition: height 0.1s;
}
.progress-track:hover { height: 6px; }
.progress-fill {
  height: 100%; background: #00d2d3; border-radius: 4px;
  position: relative; transition: width 0.3s linear;
}
.progress-thumb {
  position: absolute; right: -5px; top: 50%; transform: translateY(-50%);
  width: 10px; height: 10px; background: #fff; border-radius: 50%;
  box-shadow: 0 0 4px rgba(0,0,0,0.4);
}

/* Right */
.right { display: flex; align-items: center; gap: 0.7rem; justify-content: flex-end; }
.queue-badge {
  display: flex; align-items: center; gap: 0.3rem;
  font-size: 0.75rem; color: #9090a8; background: #262642;
  padding: 0.2rem 0.6rem; border-radius: 4px;
}
.loop-label {
  font-size: 0.6rem; font-weight: 700; letter-spacing: 0.05em;
  color: #aa78ff; background: rgba(170,120,255,0.1);
  padding: 0.15rem 0.4rem; border-radius: 3px;
}
.mini-btn {
  background: none; border: none; color: #9090a8; cursor: pointer;
  padding: 0.3rem; border-radius: 4px; display: flex; align-items: center;
}
.mini-btn:hover { color: #eaeaf0; background: rgba(255,255,255,0.05); }
.volume { display: flex; align-items: center; gap: 0.3rem; }
.vol-btn {
  background: none; border: none; color: #9090a8; cursor: pointer;
  width: 28px; height: 28px; display: flex; align-items: center; justify-content: center;
}
.vol-btn svg { width: 18px; height: 18px; }
.vol-btn:hover { color: #eaeaf0; }
.volume-slider {
  -webkit-appearance: none; width: 80px; height: 4px;
  background: #353555; border-radius: 4px; outline: none;
}
.volume-slider:hover { height: 5px; }
.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none; width: 12px; height: 12px;
  background: #eaeaf0; border-radius: 50%; cursor: pointer;
  transition: transform 0.1s;
}
.volume-slider::-webkit-slider-thumb:hover { transform: scale(1.3); }

@keyframes spin { to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }
</style>
