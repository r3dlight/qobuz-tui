<template>
  <div v-if="playlist">
    <div class="playlist-header">
      <h2>{{ playlist.name }}</h2>
      <p class="meta">
        <span v-if="playlist.owner">{{ playlist.owner.name }}</span>
        <span v-if="playlist.tracks_count"> · {{ playlist.tracks_count }} tracks</span>
      </p>
      <div class="actions">
        <button class="btn" @click="emit('back')">← Back</button>
        <button class="btn primary" @click="playAll">▶ Play all</button>
      </div>
    </div>
    <TrackList :tracks="tracks" @play="onPlay" show-album />
  </div>
  <p class="loading" v-else>Loading playlist...</p>
</template>

<script setup>
import { computed } from 'vue'
import TrackList from './TrackList.vue'

const props = defineProps({ playlist: Object })
const emit = defineEmits(['back', 'play-tracks'])

const tracks = computed(() => props.playlist?.tracks?.items || [])

function onPlay(index) { emit('play-tracks', tracks.value, index) }
function playAll() { emit('play-tracks', tracks.value, 0) }
</script>

<style scoped>
.playlist-header { margin-bottom: 1.5rem; }
h2 { font-size: 1.5rem; margin-bottom: 0.3rem; }
.meta { color: #9090a8; font-size: 0.9rem; margin-bottom: 1rem; }
.actions { display: flex; gap: 0.5rem; }
.btn {
  padding: 0.5rem 1rem;
  background: #353555;
  border: none;
  border-radius: 6px;
  color: #eaeaf0;
  cursor: pointer;
  font-size: 0.85rem;
}
.btn:hover { background: #3a3a4a; }
.btn.primary { background: #6495ed; color: #1a1a2e; font-weight: 600; }
.btn.primary:hover { background: #7ba5f7; }
.loading { color: #ffc832; text-align: center; padding: 2rem; }
</style>
