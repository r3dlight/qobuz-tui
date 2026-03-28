<template>
  <div v-if="album">
    <div class="album-header">
      <img v-if="album.image?.large" :src="album.image.large" class="album-art" />
      <div class="placeholder-art" v-else>♫</div>
      <div class="album-info">
        <h2>{{ album.title }}</h2>
        <p class="artist-link" @click="onArtistClick">{{ album.artist?.name || 'Unknown' }}</p>
        <p class="meta">
          {{ album.tracks_count || 0 }} tracks
          <span v-if="album.release_date_original"> · {{ album.release_date_original.slice(0, 4) }}</span>
        </p>
        <div class="actions">
          <button class="btn" @click="emit('back')">← Back</button>
          <button class="btn primary" @click="playAll">▶ Play all</button>
          <button class="btn" @click="emit('add-favorite', album.id)">♥ Favorite</button>
        </div>
      </div>
    </div>
    <TrackList :tracks="tracks" @play="onPlay" />
  </div>
  <p class="loading" v-else>Loading album...</p>
</template>

<script setup>
import { computed } from 'vue'
import TrackList from './TrackList.vue'

const props = defineProps({ album: Object })
const emit = defineEmits(['back', 'play-tracks', 'open-artist', 'add-favorite'])

const tracks = computed(() => props.album?.tracks?.items || [])

const coverUrl = computed(() => props.album?.image?.large || props.album?.image?.small || null)

function onPlay(index) {
  emit('play-tracks', tracks.value, index, coverUrl.value)
}
function playAll() {
  emit('play-tracks', tracks.value, 0, coverUrl.value)
}
function onArtistClick() {
  const id = props.album?.artist?.id
  if (id) emit('open-artist', id)
}
</script>

<style scoped>
.album-header {
  display: flex;
  gap: 1.5rem;
  margin-bottom: 1.5rem;
  align-items: flex-start;
}
.album-art {
  width: 200px;
  height: 200px;
  border-radius: 8px;
  object-fit: cover;
}
.placeholder-art {
  width: 200px;
  height: 200px;
  background: #2a2a3a;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 3rem;
  color: #46465a;
}
.album-info { flex: 1; }
h2 { font-size: 1.5rem; margin-bottom: 0.3rem; }
.artist-link { color: #6495ed; cursor: pointer; margin-bottom: 0.3rem; }
.artist-link:hover { text-decoration: underline; }
.meta { color: #78788c; font-size: 0.9rem; margin-bottom: 1rem; }
.actions { display: flex; gap: 0.5rem; }
.btn {
  padding: 0.5rem 1rem;
  background: #2a2a3a;
  border: none;
  border-radius: 6px;
  color: #e6e6f0;
  cursor: pointer;
  font-size: 0.85rem;
}
.btn:hover { background: #3a3a4a; }
.btn.primary { background: #6495ed; color: #12121a; font-weight: 600; }
.btn.primary:hover { background: #7ba5f7; }
.loading { color: #ffc832; text-align: center; padding: 2rem; }
</style>
