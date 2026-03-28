<template>
  <div v-if="artist">
    <div class="artist-header">
      <div>
        <h2>{{ artist.name }}</h2>
        <p class="meta">{{ artist.albums_count || 0 }} albums</p>
        <p class="bio" v-if="artist.biography?.summary">{{ artist.biography.summary }}</p>
        <button class="btn" @click="emit('back')">← Back</button>
      </div>
    </div>
    <h3 class="section-title">Discography</h3>
    <AlbumGrid :albums="albums" @open="id => emit('open-album', id)" />
  </div>
  <p class="loading" v-else>Loading artist...</p>
</template>

<script setup>
import { computed } from 'vue'
import AlbumGrid from './AlbumGrid.vue'

const props = defineProps({ artist: Object })
const emit = defineEmits(['back', 'open-album'])

const albums = computed(() => props.artist?.albums?.items || [])
</script>

<style scoped>
.artist-header { margin-bottom: 1.5rem; }
h2 { font-size: 1.8rem; margin-bottom: 0.3rem; }
.meta { color: #9090a8; margin-bottom: 0.5rem; }
.bio { color: #a8a8be; font-size: 0.9rem; line-height: 1.5; margin-bottom: 1rem; max-width: 600px; }
.section-title { color: #6495ed; font-size: 1rem; margin-bottom: 1rem; }
.btn {
  padding: 0.5rem 1rem;
  background: #353555;
  border: none;
  border-radius: 6px;
  color: #eaeaf0;
  cursor: pointer;
  font-size: 0.85rem;
  margin-bottom: 1rem;
}
.btn:hover { background: #3a3a4a; }
.loading { color: #ffc832; text-align: center; padding: 2rem; }
</style>
