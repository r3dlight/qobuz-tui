<template>
  <div class="app" v-if="!isLoggedIn">
    <LoginView @login-success="onLoginSuccess" />
  </div>
  <div class="app" v-else>
    <nav class="tabs">
      <div class="brand">Qobuz</div>
      <button :class="{ active: tab === 'search' }" @click="tab = 'search'">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15.5 14h-.79l-.28-.27A6.47 6.47 0 0 0 16 9.5 6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/></svg>
        Search
      </button>
      <button :class="{ active: tab === 'albums' }" @click="tab = 'albums'; loadFavorites()">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/></svg>
        Albums
      </button>
      <button :class="{ active: tab === 'playlists' }" @click="tab = 'playlists'; loadPlaylists()">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"/></svg>
        Playlists
      </button>
    </nav>

    <main class="content">
      <SearchView v-if="tab === 'search'" @open-album="openAlbum" @play-tracks="playTracksFrom" />
      <AlbumListView v-else-if="tab === 'albums'" :albums="favorites" :loading="loadingFavorites"
        @open-album="openAlbum" @remove-favorite="removeFavorite" />
      <PlaylistListView v-else-if="tab === 'playlists'" :playlists="playlists" :loading="loadingPlaylists"
        @open-playlist="openPlaylist" />
      <AlbumView v-else-if="tab === 'album'" :album="currentAlbum" @back="tab = previousTab"
        @play-tracks="playTracksFrom" @open-artist="openArtist" @add-favorite="addFavorite" />
      <ArtistView v-else-if="tab === 'artist'" :artist="currentArtist" @back="tab = previousTab"
        @open-album="openAlbum" />
      <PlaylistDetailView v-else-if="tab === 'playlist'" :playlist="currentPlaylist" @back="tab = 'playlists'"
        @play-tracks="playTracksFrom" />
    </main>

    <PlayerBar />
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import LoginView from './components/LoginView.vue'
import SearchView from './components/SearchView.vue'
import AlbumListView from './components/AlbumListView.vue'
import PlaylistListView from './components/PlaylistListView.vue'
import AlbumView from './components/AlbumView.vue'
import ArtistView from './components/ArtistView.vue'
import PlaylistDetailView from './components/PlaylistDetailView.vue'
import PlayerBar from './components/PlayerBar.vue'

const isLoggedIn = ref(false)
const tab = ref('search')
const previousTab = ref('search')

const favorites = ref([])
const loadingFavorites = ref(false)
const playlists = ref([])
const loadingPlaylists = ref(false)
const currentAlbum = ref(null)
const currentArtist = ref(null)
const currentPlaylist = ref(null)

onMounted(async () => {
  isLoggedIn.value = await invoke('check_auth')
})

function onLoginSuccess() {
  isLoggedIn.value = true
}

async function loadFavorites() {
  if (favorites.value.length > 0) return
  loadingFavorites.value = true
  try {
    favorites.value = await invoke('get_favorites')
  } catch (e) { console.error(e) }
  loadingFavorites.value = false
}

async function loadPlaylists() {
  if (playlists.value.length > 0) return
  loadingPlaylists.value = true
  try {
    playlists.value = await invoke('get_playlists')
  } catch (e) { console.error(e) }
  loadingPlaylists.value = false
}

async function openAlbum(albumId) {
  previousTab.value = tab.value
  tab.value = 'album'
  currentAlbum.value = null
  try {
    currentAlbum.value = await invoke('get_album', { albumId })
  } catch (e) { console.error(e) }
}

async function openArtist(artistId) {
  previousTab.value = tab.value
  tab.value = 'artist'
  currentArtist.value = null
  try {
    currentArtist.value = await invoke('get_artist', { artistId })
  } catch (e) { console.error(e) }
}

async function openPlaylist(playlistId) {
  previousTab.value = 'playlists'
  tab.value = 'playlist'
  currentPlaylist.value = null
  try {
    currentPlaylist.value = await invoke('get_playlist', { playlistId })
  } catch (e) { console.error(e) }
}

async function playTracksFrom(tracks, index, coverUrl) {
  try {
    await invoke('play_queue_from', {
      tracksJson: JSON.stringify(tracks),
      index,
      coverUrl: coverUrl || null,
    })
  } catch (e) { console.error(e) }
}

async function addFavorite(albumId) {
  try {
    await invoke('add_favorite', { albumId })
  } catch (e) { console.error(e) }
}

async function removeFavorite(albumId) {
  try {
    await invoke('remove_favorite', { albumId })
    favorites.value = favorites.value.filter(a => a.id !== albumId)
  } catch (e) { console.error(e) }
}
</script>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  background: #12121a;
  color: #e6e6f0;
  overflow: hidden;
  height: 100vh;
}
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
}
.tabs {
  display: flex;
  align-items: center;
  gap: 0;
  background: #191924;
  border-bottom: 1px solid #252535;
  padding: 0 1rem;
  -webkit-app-region: drag;
}
.brand {
  font-size: 1.1rem;
  font-weight: 700;
  color: #6495ed;
  padding: 0 1rem 0 0.5rem;
  margin-right: 0.5rem;
  border-right: 1px solid #252535;
  -webkit-app-region: no-drag;
}
.tabs button {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.8rem 1.2rem;
  background: none;
  border: none;
  color: #5a5a6e;
  cursor: pointer;
  font-size: 0.85rem;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
  -webkit-app-region: no-drag;
}
.tabs button svg { width: 16px; height: 16px; }
.tabs button:hover { color: #bec0cd; }
.tabs button.active {
  color: #e6e6f0;
  border-bottom-color: #6495ed;
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}
</style>
