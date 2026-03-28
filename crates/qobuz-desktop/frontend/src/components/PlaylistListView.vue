<template>
  <div>
    <p class="loading" v-if="loading">Loading playlists...</p>
    <div class="playlist-list" v-else-if="playlists.length > 0">
      <div class="playlist-row" v-for="pl in playlists" :key="pl.id" @click="emit('open-playlist', pl.id)">
        <div class="pl-icon">♫</div>
        <div class="pl-info">
          <div class="pl-name">{{ pl.name }}</div>
          <div class="pl-meta">
            <span v-if="pl.owner">{{ pl.owner.name }}</span>
            <span v-if="pl.tracks_count"> · {{ pl.tracks_count }} tracks</span>
          </div>
        </div>
      </div>
    </div>
    <p class="empty" v-else>No playlists.</p>
  </div>
</template>

<script setup>
defineProps({
  playlists: { type: Array, default: () => [] },
  loading: Boolean
})
const emit = defineEmits(['open-playlist'])
</script>

<style scoped>
.playlist-list { display: flex; flex-direction: column; gap: 0.3rem; }
.playlist-row {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.8rem 1rem;
  background: #1e1e2e;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}
.playlist-row:hover { background: #252535; }
.pl-icon { font-size: 1.5rem; color: #aa78ff; width: 40px; text-align: center; }
.pl-name { font-weight: 600; color: #e6e6f0; }
.pl-meta { font-size: 0.8rem; color: #78788c; }
.loading { color: #ffc832; text-align: center; padding: 2rem; }
.empty { color: #46465a; text-align: center; padding: 2rem; }
</style>
