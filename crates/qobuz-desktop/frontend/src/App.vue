<template>
  <div class="app" v-if="!isLoggedIn">
    <LoginView @login-success="onLoginSuccess" />
  </div>
  <div class="app" v-else>
    <nav class="tabs">
      <button :class="{ active: tab === 'search' }" @click="tab = 'search'">Search</button>
      <button :class="{ active: tab === 'albums' }" @click="tab = 'albums'; loadFavorites()">Albums</button>
      <button :class="{ active: tab === 'playlists' }" @click="tab = 'playlists'; loadPlaylists()">Playlists</button>
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
  gap: 0;
  background: #191924;
  border-bottom: 1px solid #2a2a3a;
  padding: 0 1rem;
}
.tabs button {
  padding: 0.8rem 1.5rem;
  background: none;
  border: none;
  color: #78788c;
  cursor: pointer;
  font-size: 0.9rem;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}
.tabs button:hover { color: #bec0cd; }
.tabs button.active {
  color: #6495ed;
  border-bottom-color: #6495ed;
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 1rem;
}
</style>
