<template>
  <div class="album-grid" v-if="albums.length > 0">
    <div class="album-card" v-for="album in albums" :key="album.id" @click="emit('open', album.id)">
      <img v-if="album.image?.small" :src="album.image.small" :alt="album.title" class="cover" />
      <div class="placeholder-cover" v-else>♫</div>
      <div class="info">
        <div class="album-title">{{ album.title }}</div>
        <div class="album-artist">{{ album.artist?.name || 'Unknown' }}</div>
        <div class="album-meta">
          <span v-if="album.tracks_count">{{ album.tracks_count }} tracks</span>
          <span v-if="album.release_date_original"> · {{ album.release_date_original?.slice(0, 4) }}</span>
        </div>
      </div>
    </div>
  </div>
  <p class="empty" v-else>No albums.</p>
</template>

<script setup>
defineProps({ albums: { type: Array, default: () => [] } })
const emit = defineEmits(['open'])
</script>

<style scoped>
.album-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 1rem;
}
.album-card {
  background: #1e1e2e;
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  transition: transform 0.15s, box-shadow 0.15s;
}
.album-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
}
.cover { width: 100%; aspect-ratio: 1; object-fit: cover; display: block; }
.placeholder-cover {
  width: 100%;
  aspect-ratio: 1;
  background: #2a2a3a;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 2rem;
  color: #46465a;
}
.info { padding: 0.7rem; }
.album-title { font-size: 0.9rem; font-weight: 600; color: #e6e6f0; margin-bottom: 0.2rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.album-artist { font-size: 0.8rem; color: #9696a8; }
.album-meta { font-size: 0.75rem; color: #5a5a6e; margin-top: 0.2rem; }
.empty { color: #46465a; text-align: center; padding: 2rem; }
</style>
