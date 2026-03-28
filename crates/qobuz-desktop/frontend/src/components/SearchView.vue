<template>
  <div>
    <div class="search-bar">
      <input v-model="query" @keydown.enter="doSearch" placeholder="Search tracks, albums..." autofocus />
    </div>
    <div class="mode-tabs">
      <button :class="{ active: mode === 'tracks' }" @click="mode = 'tracks'">Tracks</button>
      <button :class="{ active: mode === 'albums' }" @click="mode = 'albums'">Albums</button>
    </div>
    <p class="status" v-if="loading">Searching...</p>
    <TrackList v-if="mode === 'tracks'" :tracks="tracks" @play="onPlayTrack" show-album />
    <AlbumGrid v-else :albums="albums" @open="id => emit('open-album', id)" />
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import TrackList from './TrackList.vue'
import AlbumGrid from './AlbumGrid.vue'

const emit = defineEmits(['open-album', 'play-tracks'])
const query = ref('')
const mode = ref('tracks')
const tracks = ref([])
const albums = ref([])
const loading = ref(false)

async function doSearch() {
  if (!query.value.trim()) return
  loading.value = true
  try {
    const results = await invoke('search', { query: query.value, limit: 50, offset: 0 })
    tracks.value = results.tracks?.items || []
    albums.value = results.albums?.items || []
  } catch (e) { console.error(e) }
  loading.value = false
}

function onPlayTrack(index) {
  emit('play-tracks', tracks.value, index)
}
</script>

<style scoped>
.search-bar { margin-bottom: 1rem; }
.search-bar input {
  width: 100%;
  padding: 0.8rem 1rem;
  background: #1e1e2e;
  border: 1px solid #2a2a3a;
  border-radius: 8px;
  color: #e6e6f0;
  font-size: 1rem;
  outline: none;
}
.search-bar input:focus { border-color: #6495ed; }
.mode-tabs { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
.mode-tabs button {
  padding: 0.4rem 1rem;
  background: #1e1e2e;
  border: 1px solid #2a2a3a;
  border-radius: 6px;
  color: #78788c;
  cursor: pointer;
  font-size: 0.85rem;
}
.mode-tabs button.active { background: #6495ed; color: #12121a; border-color: #6495ed; }
.status { color: #ffc832; font-size: 0.9rem; }
</style>
