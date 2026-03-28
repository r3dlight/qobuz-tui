<template>
  <div>
    <div class="search-bar">
      <svg class="search-icon" viewBox="0 0 24 24" fill="currentColor"><path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
      <input ref="searchInput" v-model="query" @keydown.enter="doSearch" placeholder="Search tracks, albums, artists..." autofocus />
      <span class="result-count" v-if="totalResults > 0">{{ totalResults }} results</span>
    </div>
    <div class="mode-tabs">
      <button :class="{ active: mode === 'tracks' }" @click="mode = 'tracks'">
        Tracks <span class="count" v-if="tracks.length">({{ tracks.length }})</span>
      </button>
      <button :class="{ active: mode === 'albums' }" @click="mode = 'albums'">
        Albums <span class="count" v-if="albums.length">({{ albums.length }})</span>
      </button>
    </div>
    <div class="loading" v-if="loading">
      <div class="spinner"></div>
      <span>Searching...</span>
    </div>
    <template v-else-if="hasSearched">
      <TrackList v-if="mode === 'tracks'" :tracks="tracks" @play="onPlayTrack" show-album />
      <AlbumGrid v-else :albums="albums" @open="id => emit('open-album', id)" />
    </template>
    <div class="welcome" v-else>
      <svg viewBox="0 0 24 24" fill="currentColor" width="64" height="64">
        <path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
      </svg>
      <h2>Search Qobuz</h2>
      <p>Find your favorite tracks, albums, and artists</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import TrackList from './TrackList.vue'
import AlbumGrid from './AlbumGrid.vue'

const emit = defineEmits(['open-album', 'play-tracks'])
const query = ref('')
const mode = ref('tracks')
const tracks = ref([])
const albums = ref([])
const loading = ref(false)
const hasSearched = ref(false)
const totalResults = computed(() => tracks.value.length + albums.value.length)
const searchInput = ref(null)

// Debounced instant search
let debounceTimer = null
watch(query, (val) => {
  clearTimeout(debounceTimer)
  if (val.trim().length >= 2) {
    debounceTimer = setTimeout(doSearch, 350)
  }
})

// Expose focus for keyboard shortcut
defineExpose({ focus: () => searchInput.value?.focus() })

async function doSearch() {
  if (!query.value.trim()) return
  loading.value = true
  hasSearched.value = true
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
.search-bar {
  position: relative;
  margin-bottom: 1rem;
  display: flex;
  align-items: center;
}
.search-icon {
  position: absolute; left: 12px;
  width: 20px; height: 20px; color: #707088;
}
.search-bar input {
  width: 100%;
  padding: 0.8rem 1rem 0.8rem 2.5rem;
  background: #262642;
  border: 1px solid #353555;
  border-radius: 10px;
  color: #eaeaf0;
  font-size: 1rem;
  outline: none;
  transition: border-color 0.2s;
}
.search-bar input:focus { border-color: #6495ed; }
.result-count {
  position: absolute; right: 12px;
  font-size: 0.75rem; color: #707088;
}
.mode-tabs { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
.mode-tabs button {
  padding: 0.45rem 1.2rem;
  background: #262642;
  border: 1px solid #353555;
  border-radius: 8px;
  color: #9090a8;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.15s;
}
.mode-tabs button:hover { border-color: #3a3a4a; color: #bec0cd; }
.mode-tabs button.active { background: #6495ed; color: #1a1a2e; border-color: #6495ed; font-weight: 600; }
.count { opacity: 0.7; }
.loading { display: flex; align-items: center; gap: 0.7rem; color: #ffc832; padding: 2rem; justify-content: center; }
.spinner {
  width: 18px; height: 18px; border: 2px solid #ffc832; border-top-color: transparent;
  border-radius: 50%; animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
.welcome {
  display: flex; flex-direction: column; align-items: center;
  justify-content: center; padding: 4rem; color: #4a4a65; gap: 0.5rem;
}
.welcome h2 { color: #707088; font-size: 1.3rem; }
.welcome p { color: #4a4a65; font-size: 0.9rem; }
</style>
