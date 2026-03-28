<template>
  <div class="track-list" v-if="tracks.length > 0">
    <table>
      <thead>
        <tr>
          <th class="num">#</th>
          <th>Title</th>
          <th>Artist</th>
          <th v-if="showAlbum">Album</th>
          <th class="dur">Duration</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(track, i) in tracks" :key="track.id || i"
          @dblclick="emit('play', i)"
          class="track-row"
          :class="{ playing: isCurrentTrack(track) }">
          <td class="num">
            <span class="num-text">{{ track.track_number || i + 1 }}</span>
            <button class="play-btn" @click="emit('play', i)" title="Play">
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            </button>
          </td>
          <td class="title-cell">
            <span class="track-title">{{ track.title }}</span>
          </td>
          <td class="artist">{{ track.performer?.name || 'Unknown' }}</td>
          <td v-if="showAlbum" class="album">{{ track.album?.title || '' }}</td>
          <td class="dur">{{ formatDuration(track.duration) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
  <div class="empty-state" v-else>
    <svg viewBox="0 0 24 24" fill="currentColor" width="48" height="48">
      <path d="M12 3v10.55A4 4 0 1 0 14 17V7h4V3h-6Z"/>
    </svg>
    <p>No tracks</p>
  </div>
</template>

<script setup>
import { inject } from 'vue'

defineProps({
  tracks: { type: Array, default: () => [] },
  showAlbum: { type: Boolean, default: false }
})
const emit = defineEmits(['play'])

function isCurrentTrack(track) {
  return false // TODO: compare with player state
}

function formatDuration(s) {
  if (!s) return '0:00'
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}
</script>

<style scoped>
table { width: 100%; border-collapse: collapse; }
thead th {
  text-align: left;
  color: #707088;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.6rem 0.8rem;
  border-bottom: 1px solid #262642;
}
.track-row {
  cursor: default;
  transition: background 0.1s;
  border-radius: 4px;
}
.track-row:hover { background: #262642; }
.track-row:nth-child(even) { background: rgba(255,255,255,0.01); }
.track-row:nth-child(even):hover { background: #262642; }
.track-row.playing { background: rgba(100,149,237,0.08); }
.track-row.playing .track-title { color: #6495ed; }

td { padding: 0.55rem 0.8rem; font-size: 0.88rem; }

.num {
  width: 40px;
  text-align: center;
  position: relative;
}
.num-text { color: #585875; }
.play-btn {
  display: none;
  background: none; border: none; cursor: pointer;
  color: #eaeaf0; width: 20px; height: 20px; padding: 0;
}
.play-btn svg { width: 16px; height: 16px; }
.track-row:hover .num-text { display: none; }
.track-row:hover .play-btn { display: inline-flex; align-items: center; justify-content: center; }

.title-cell { max-width: 300px; }
.track-title { color: #eaeaf0; font-weight: 500; }
.artist { color: #9090a8; }
.album { color: #707088; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.dur { color: #707088; width: 70px; text-align: right; font-variant-numeric: tabular-nums; }

.empty-state {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; padding: 3rem; color: #4a4a65; gap: 0.5rem;
}
.empty-state p { font-size: 0.9rem; }
</style>
