<template>
  <div class="home">
    <section v-if="newReleases.length > 0">
      <h2>New Releases</h2>
      <div class="album-row">
        <div class="album-card" v-for="album in newReleases" :key="album.id" @click="emit('open-album', album.id)">
          <img v-if="album.image?.small" :src="album.image.small" class="cover" :alt="album.title" />
          <div class="placeholder" v-else>♫</div>
          <div class="card-title">{{ album.title }}</div>
          <div class="card-artist">{{ album.artist?.name || '' }}</div>
        </div>
      </div>
    </section>

    <section v-if="editorPicks.length > 0">
      <h2>Editor's Picks</h2>
      <div class="album-row">
        <div class="album-card" v-for="album in editorPicks" :key="album.id" @click="emit('open-album', album.id)">
          <img v-if="album.image?.small" :src="album.image.small" class="cover" :alt="album.title" />
          <div class="placeholder" v-else>♫</div>
          <div class="card-title">{{ album.title }}</div>
          <div class="card-artist">{{ album.artist?.name || '' }}</div>
        </div>
      </div>
    </section>

    <section v-if="bestSellers.length > 0">
      <h2>Best Sellers</h2>
      <div class="album-row">
        <div class="album-card" v-for="album in bestSellers" :key="album.id" @click="emit('open-album', album.id)">
          <img v-if="album.image?.small" :src="album.image.small" class="cover" :alt="album.title" />
          <div class="placeholder" v-else>♫</div>
          <div class="card-title">{{ album.title }}</div>
          <div class="card-artist">{{ album.artist?.name || '' }}</div>
        </div>
      </div>
    </section>

    <section v-if="mostStreamed.length > 0">
      <h2>Most Streamed</h2>
      <div class="album-row">
        <div class="album-card" v-for="album in mostStreamed" :key="album.id" @click="emit('open-album', album.id)">
          <img v-if="album.image?.small" :src="album.image.small" class="cover" :alt="album.title" />
          <div class="placeholder" v-else>♫</div>
          <div class="card-title">{{ album.title }}</div>
          <div class="card-artist">{{ album.artist?.name || '' }}</div>
        </div>
      </div>
    </section>

    <div class="loading" v-if="loading">
      <div class="spinner"></div>
      Loading recommendations...
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['open-album'])
const newReleases = ref([])
const editorPicks = ref([])
const bestSellers = ref([])
const mostStreamed = ref([])
const loading = ref(true)

onMounted(async () => {
  try {
    const [nr, ep, bs, ms] = await Promise.allSettled([
      invoke('get_featured', { type_: 'new-releases', limit: 12 }),
      invoke('get_featured', { type_: 'editor-picks', limit: 12 }),
      invoke('get_featured', { type_: 'best-sellers', limit: 12 }),
      invoke('get_featured', { type_: 'most-streamed', limit: 12 }),
    ])
    if (nr.status === 'fulfilled') newReleases.value = nr.value
    if (ep.status === 'fulfilled') editorPicks.value = ep.value
    if (bs.status === 'fulfilled') bestSellers.value = bs.value
    if (ms.status === 'fulfilled') mostStreamed.value = ms.value
  } catch (_) {}
  loading.value = false
})
</script>

<style scoped>
.home { display: flex; flex-direction: column; gap: 2rem; }

section h2 {
  font-size: 1.1rem;
  font-weight: 700;
  color: #e6e6f0;
  margin-bottom: 0.8rem;
  padding-left: 0.2rem;
}

.album-row {
  display: flex;
  gap: 0.8rem;
  overflow-x: auto;
  padding-bottom: 0.5rem;
  scrollbar-width: thin;
  scrollbar-color: #2a2a3a transparent;
}
.album-row::-webkit-scrollbar { height: 6px; }
.album-row::-webkit-scrollbar-thumb { background: #2a2a3a; border-radius: 3px; }

.album-card {
  flex: 0 0 150px;
  cursor: pointer;
  transition: transform 0.15s;
}
.album-card:hover { transform: translateY(-4px); }

.cover {
  width: 150px; height: 150px;
  border-radius: 6px;
  object-fit: cover;
  display: block;
  box-shadow: 0 2px 12px rgba(0,0,0,0.3);
}
.placeholder {
  width: 150px; height: 150px;
  background: #252535; border-radius: 6px;
  display: flex; align-items: center; justify-content: center;
  font-size: 2rem; color: #3a3a4e;
}
.card-title {
  margin-top: 0.5rem;
  font-size: 0.82rem;
  font-weight: 600;
  color: #e6e6f0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  max-width: 150px;
}
.card-artist {
  font-size: 0.75rem;
  color: #78788c;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  max-width: 150px;
}

.loading {
  display: flex; align-items: center; justify-content: center;
  gap: 0.7rem; color: #5a5a6e; padding: 3rem;
}
.spinner {
  width: 16px; height: 16px;
  border: 2px solid #5a5a6e; border-top-color: transparent;
  border-radius: 50%; animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
