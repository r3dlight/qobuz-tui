<template>
  <div class="track-list" v-if="tracks.length > 0">
    <table>
      <thead>
        <tr>
          <th class="num">#</th>
          <th>Title</th>
          <th>Artist</th>
          <th v-if="showAlbum">Album</th>
          <th class="dur">Time</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(track, i) in tracks" :key="track.id" @dblclick="emit('play', i)" class="track-row">
          <td class="num">{{ track.track_number || i + 1 }}</td>
          <td class="title">{{ track.title }}</td>
          <td class="artist">{{ track.performer?.name || 'Unknown' }}</td>
          <td v-if="showAlbum" class="album">{{ track.album?.title || '' }}</td>
          <td class="dur">{{ formatDuration(track.duration) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
  <p class="empty" v-else>No tracks.</p>
</template>

<script setup>
defineProps({
  tracks: { type: Array, default: () => [] },
  showAlbum: { type: Boolean, default: false }
})
const emit = defineEmits(['play'])

function formatDuration(s) {
  if (!s) return '0:00'
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`
}
</script>

<style scoped>
table { width: 100%; border-collapse: collapse; }
thead th {
  text-align: left;
  color: #6495ed;
  font-size: 0.8rem;
  font-weight: 600;
  padding: 0.5rem 0.8rem;
  border-bottom: 1px solid #2a2a3a;
}
.track-row {
  cursor: pointer;
  transition: background 0.15s;
}
.track-row:hover { background: #1e1e2e; }
.track-row:nth-child(even) { background: #16161f; }
.track-row:nth-child(even):hover { background: #1e1e2e; }
td { padding: 0.6rem 0.8rem; font-size: 0.9rem; }
.num { color: #46465a; width: 40px; }
.title { color: #e6e6f0; }
.artist { color: #9696a8; }
.album { color: #5a5a6e; }
.dur { color: #5a5a6e; width: 60px; text-align: right; }
.empty { color: #46465a; text-align: center; padding: 2rem; }
</style>
