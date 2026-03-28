<template>
  <div class="app" :class="{ light: lightTheme }" v-if="!isLoggedIn">
    <LoginView @login-success="onLoginSuccess" />
  </div>
  <div class="app" :class="{ light: lightTheme }" v-else>
    <nav class="tabs">
      <div class="brand">Qobuz</div>
      <button :class="{ active: tab === 'home' }" @click="tab = 'home'">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
        Home
      </button>
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
      <div class="tab-spacer"></div>
      <button class="icon-btn" @click="showQueue = !showQueue" title="Queue (Ctrl+Q)">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"/></svg>
      </button>
      <button class="icon-btn" @click="lightTheme = !lightTheme" title="Toggle theme (Ctrl+T)">
        <svg v-if="lightTheme" viewBox="0 0 24 24" fill="currentColor"><path d="M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z"/></svg>
        <svg v-else viewBox="0 0 24 24" fill="currentColor"><path d="M6.76 4.84l-1.8-1.79-1.41 1.41 1.79 1.79 1.42-1.41zM4 10.5H1v2h3v-2zm9-9.95h-2V3.5h2V.55zm7.45 3.91l-1.41-1.41-1.79 1.79 1.41 1.41 1.79-1.79zm-3.21 13.7l1.79 1.8 1.41-1.41-1.8-1.79-1.4 1.4zM20 10.5v2h3v-2h-3zm-8-5a6 6 0 1 0 0 12 6 6 0 0 0 0-12zm-1 16.95h2V19.5h-2v2.95zm-7.45-3.91l1.41 1.41 1.79-1.8-1.41-1.41-1.79 1.8z"/></svg>
      </button>
    </nav>

    <main class="content">
      <Transition name="fade" mode="out-in">
        <HomeView v-if="tab === 'home'" :key="'home'" @open-album="openAlbum" />
        <SearchView v-else-if="tab === 'search'" :key="'search'" ref="searchView"
          @open-album="openAlbum" @play-tracks="playTracksFrom" />
        <AlbumListView v-else-if="tab === 'albums'" :key="'albums'" :albums="favorites"
          :loading="loadingFavorites" @open-album="openAlbum" @remove-favorite="removeFavorite" />
        <PlaylistListView v-else-if="tab === 'playlists'" :key="'playlists'" :playlists="playlists"
          :loading="loadingPlaylists" @open-playlist="openPlaylist" />
        <AlbumView v-else-if="tab === 'album'" :key="'album-'+currentAlbum?.id" :album="currentAlbum"
          @back="tab = previousTab" @play-tracks="playTracksFrom" @open-artist="openArtist"
          @add-favorite="addFavorite" />
        <ArtistView v-else-if="tab === 'artist'" :key="'artist'" :artist="currentArtist"
          @back="tab = previousTab" @open-album="openAlbum" />
        <PlaylistDetailView v-else-if="tab === 'playlist'" :key="'playlist'" :playlist="currentPlaylist"
          @back="tab = 'playlists'" @play-tracks="playTracksFrom" />
      </Transition>
    </main>

    <PlayerBar />
    <QueuePanel :visible="showQueue" @close="showQueue = false" />
    <ContextMenu ref="contextMenu" />
  </div>
</template>

<script setup>
import { ref, onMounted, provide } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useKeyboard } from './composables/useKeyboard.js'
import { useTrackNotifications } from './composables/useNotifications.js'
import LoginView from './components/LoginView.vue'
import HomeView from './components/HomeView.vue'
import SearchView from './components/SearchView.vue'
import AlbumListView from './components/AlbumListView.vue'
import PlaylistListView from './components/PlaylistListView.vue'
import AlbumView from './components/AlbumView.vue'
import ArtistView from './components/ArtistView.vue'
import PlaylistDetailView from './components/PlaylistDetailView.vue'
import PlayerBar from './components/PlayerBar.vue'
import QueuePanel from './components/QueuePanel.vue'
import ContextMenu from './components/ContextMenu.vue'

const isLoggedIn = ref(false)
const tab = ref('home')
const previousTab = ref('home')
const lightTheme = ref(false)
const showQueue = ref(false)
const searchView = ref(null)
const contextMenu = ref(null)

const favorites = ref([])
const loadingFavorites = ref(false)
const playlists = ref([])
const loadingPlaylists = ref(false)
const currentAlbum = ref(null)
const currentArtist = ref(null)
const currentPlaylist = ref(null)

// Player state for notifications
const playerState = ref({})
let pollInterval = null
onMounted(async () => {
  isLoggedIn.value = await invoke('check_auth')
  pollInterval = setInterval(async () => {
    try { playerState.value = await invoke('get_player_state') } catch (_) {}
  }, 1000)
})

// Keyboard shortcuts
useKeyboard({
  onSearch: () => { tab.value = 'search'; setTimeout(() => searchView.value?.focus(), 100) },
  onToggleQueue: () => { showQueue.value = !showQueue.value },
  onToggleTheme: () => { lightTheme.value = !lightTheme.value },
})

// Track change notifications
useTrackNotifications(playerState)

// Provide context menu to children
provide('contextMenu', contextMenu)

function onLoginSuccess() { isLoggedIn.value = true }

async function loadFavorites() {
  if (favorites.value.length > 0) return
  loadingFavorites.value = true
  try { favorites.value = await invoke('get_favorites') } catch (e) { console.error(e) }
  loadingFavorites.value = false
}

async function loadPlaylists() {
  if (playlists.value.length > 0) return
  loadingPlaylists.value = true
  try { playlists.value = await invoke('get_playlists') } catch (e) { console.error(e) }
  loadingPlaylists.value = false
}

async function openAlbum(albumId) {
  previousTab.value = tab.value
  tab.value = 'album'
  currentAlbum.value = null
  try { currentAlbum.value = await invoke('get_album', { albumId }) } catch (e) { console.error(e) }
}

async function openArtist(artistId) {
  previousTab.value = tab.value
  tab.value = 'artist'
  currentArtist.value = null
  try { currentArtist.value = await invoke('get_artist', { artistId }) } catch (e) { console.error(e) }
}

async function openPlaylist(playlistId) {
  previousTab.value = 'playlists'
  tab.value = 'playlist'
  currentPlaylist.value = null
  try { currentPlaylist.value = await invoke('get_playlist', { playlistId }) } catch (e) { console.error(e) }
}

async function playTracksFrom(tracks, index, coverUrl) {
  try {
    await invoke('play_queue_from', { tracksJson: JSON.stringify(tracks), index, coverUrl: coverUrl || null })
  } catch (e) { console.error(e) }
}

async function addFavorite(albumId) {
  try { await invoke('add_favorite', { albumId }) } catch (e) { console.error(e) }
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

/* Dark theme (default) */
body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  background: var(--bg, #1a1a2e);
  color: var(--text, #eaeaf0);
  overflow: hidden;
  height: 100vh;
}
::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border, #3a3a52); border-radius: 4px; }

.app {
  --bg: #1a1a2e;
  --surface: #20203a;
  --card: #262642;
  --border: #30304a;
  --text: #eaeaf0;
  --text2: #a8a8be;
  --text3: #707088;
  --accent: #7aa5f7;
  display: flex;
  flex-direction: column;
  height: 100vh;
}

/* Light theme */
.app.light {
  --bg: #f5f5f8;
  --surface: #ffffff;
  --card: #f0f0f4;
  --border: #dddde5;
  --text: #1a1a2e;
  --text2: #555568;
  --text3: #8888a0;
  --accent: #4a7ee0;
}
.app.light body, .app.light { background: var(--bg); color: var(--text); }

.tabs {
  display: flex;
  align-items: center;
  gap: 0;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  padding: 0 0.8rem;
  -webkit-app-region: drag;
}
.brand {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--accent);
  padding: 0 1rem 0 0.5rem;
  margin-right: 0.3rem;
  border-right: 1px solid var(--border);
  -webkit-app-region: no-drag;
}
.tabs button {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.8rem 1rem;
  background: none;
  border: none;
  color: var(--text3);
  cursor: pointer;
  font-size: 0.85rem;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
  -webkit-app-region: no-drag;
}
.tabs button svg { width: 16px; height: 16px; }
.tabs button:hover { color: var(--text2); }
.tabs button.active { color: var(--text); border-bottom-color: var(--accent); }
.tab-spacer { flex: 1; }
.icon-btn {
  padding: 0.5rem !important;
  border-bottom: none !important;
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 1.2rem;
  background: var(--bg);
}

/* Transitions */
.fade-enter-active { transition: opacity 0.15s ease; }
.fade-leave-active { transition: opacity 0.1s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
