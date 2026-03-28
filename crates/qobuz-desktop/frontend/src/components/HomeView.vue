<template>
  <div class="home">
    <!-- Hero banner -->
    <div class="hero">
      <div class="hero-content">
        <h1>Welcome back</h1>
        <p>Discover new music in Hi-Res quality</p>
      </div>
      <div class="hero-quick" v-if="newReleases.length > 0">
        <div class="hero-card" v-for="album in newReleases.slice(0, 5)" :key="album.id"
          @click="emit('open-album', album.id)">
          <img v-if="album.image?.small" :src="album.image.small" class="hero-cover" />
          <div class="hero-card-info">
            <div class="hero-card-title">{{ album.title }}</div>
            <div class="hero-card-artist">{{ album.artist?.name || '' }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Recently Played -->
    <section v-if="recentTracks.length > 0">
      <div class="section-header">
        <h2><span class="section-icon" style="color: #e74c3c">♪</span> Recently Played</h2>
      </div>
      <div class="album-row">
        <div class="album-card" v-for="(track, i) in recentTracks" :key="i"
          @click="track.album_id && emit('open-album', track.album_id)">
          <div class="cover-wrapper">
            <img v-if="track.cover_url" :src="track.cover_url" class="cover" />
            <div class="placeholder" v-else>♫</div>
          </div>
          <div class="card-title">{{ track.title }}</div>
          <div class="card-artist">{{ track.artist }}</div>
        </div>
      </div>
    </section>

    <!-- Sections -->
    <section v-for="section in sections" :key="section.key" v-show="section.items.length > 0">
      <div class="section-header">
        <h2>
          <span class="section-icon" :style="{ color: section.color }">{{ section.icon }}</span>
          {{ section.title }}
        </h2>
        <span class="section-count">{{ section.items.length }} albums</span>
      </div>
      <div class="album-row">
        <div class="album-card" v-for="album in section.items" :key="album.id"
          @click="emit('open-album', album.id)">
          <div class="cover-wrapper">
            <img v-if="album.image?.small" :src="album.image.small" class="cover" :alt="album.title" />
            <div class="placeholder" v-else>♫</div>
            <div class="cover-overlay">
              <button class="play-overlay-btn">▶</button>
            </div>
          </div>
          <div class="card-title">{{ album.title }}</div>
          <div class="card-artist">{{ album.artist?.name || '' }}</div>
          <div class="card-year" v-if="album.release_date_original">
            {{ album.release_date_original?.slice(0, 4) }}
          </div>
        </div>
      </div>
    </section>

    <!-- Loading -->
    <div class="loading-screen" v-if="loading">
      <div class="loading-grid">
        <div class="skeleton-card" v-for="i in 12" :key="i">
          <div class="skeleton-cover"></div>
          <div class="skeleton-line w80"></div>
          <div class="skeleton-line w50"></div>
        </div>
      </div>
    </div>

    <!-- Error -->
    <div class="error-box" v-if="error">
      <p>{{ error }}</p>
      <button @click="retry">Retry</button>
    </div>

    <!-- Empty state (loaded but nothing) -->
    <div class="empty-home" v-if="!loading && !error && sections.every(s => s.items.length === 0)">
      <p>No recommendations available. Try searching for music.</p>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['open-album'])
const newReleases = ref([])
const editorPicks = ref([])
const bestSellers = ref([])
const mostStreamed = ref([])
const genreSections = ref([])
const recentTracks = ref([])
const loading = ref(true)
const error = ref('')

const sections = computed(() => [
  { key: 'new', title: 'New Releases', icon: '✦', color: '#6495ed', items: newReleases.value },
  { key: 'editor', title: "Editor's Picks", icon: '★', color: '#ffc832', items: editorPicks.value },
  { key: 'best', title: 'Best Sellers', icon: '♛', color: '#ff9f43', items: bestSellers.value },
  { key: 'stream', title: 'Most Streamed', icon: '♫', color: '#00d2d3', items: mostStreamed.value },
  ...genreSections.value,
])

const genreColors = ['#e74c3c', '#9b59b6', '#1abc9c', '#e67e22', '#2ecc71', '#3498db', '#f39c12', '#e91e63']

onMounted(() => loadHome())

async function loadHome() {
  loading.value = true
  error.value = ''

  // Load recent history
  try { recentTracks.value = await invoke('get_recent') } catch (_) {}

  try {
    const [nr, ep, bs, ms] = await Promise.allSettled([
      invoke('get_featured', { featuredType: 'new-releases', limit: 20 }),
      invoke('get_featured', { featuredType: 'editor-picks', limit: 20 }),
      invoke('get_featured', { featuredType: 'best-sellers', limit: 20 }),
      invoke('get_featured', { featuredType: 'most-streamed', limit: 20 }),
    ])
    if (nr.status === 'fulfilled') newReleases.value = nr.value
    else if (nr.status === 'rejected') error.value = String(nr.reason)

    if (ep.status === 'fulfilled') editorPicks.value = ep.value
    if (bs.status === 'fulfilled') bestSellers.value = bs.value
    if (ms.status === 'fulfilled') mostStreamed.value = ms.value
  } catch (e) {
    error.value = String(e)
  }
  loading.value = false

  // Genre sections (non-blocking, loaded after main content)
  try {
    const genres = await invoke('get_genres')
    const selected = genres.slice(0, 6)
    const results = await Promise.allSettled(
      selected.map(g => invoke('get_featured_by_genre', {
        featuredType: 'new-releases', genreId: g.id, limit: 15
      }))
    )
    genreSections.value = selected
      .map((g, i) => ({
        key: `genre-${g.id}`,
        title: `New in ${g.name}`,
        icon: '●',
        color: genreColors[i % genreColors.length],
        items: results[i].status === 'fulfilled' ? results[i].value : [],
      }))
      .filter(s => s.items.length > 0)
  } catch (_) {}
}

function retry() { loadHome() }
</script>

<style scoped>
.home { display: flex; flex-direction: column; gap: 1.8rem; padding-bottom: 1rem; }

/* Hero */
.hero {
  background: linear-gradient(135deg, #252545 0%, #1e3060 50%, #1a4080 100%);
  border-radius: 12px;
  padding: 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 2rem;
}
.hero-content h1 {
  font-size: 1.8rem;
  font-weight: 800;
  background: linear-gradient(90deg, #6495ed, #00d2d3);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 0.3rem;
}
.hero-content p { color: #a8a8be; font-size: 0.95rem; }

.hero-quick { display: flex; gap: 0.6rem; }
.hero-card {
  display: flex; align-items: center; gap: 0.6rem;
  background: rgba(255,255,255,0.05);
  border-radius: 8px; padding: 0.5rem 0.8rem 0.5rem 0.5rem;
  cursor: pointer; transition: background 0.15s;
  min-width: 180px;
}
.hero-card:hover { background: rgba(255,255,255,0.1); }
.hero-cover { width: 44px; height: 44px; border-radius: 4px; object-fit: cover; }
.hero-card-title { font-size: 0.8rem; font-weight: 600; color: #eaeaf0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 110px; }
.hero-card-artist { font-size: 0.7rem; color: #a8a8be; }

/* Section headers */
.section-header { display: flex; justify-content: space-between; align-items: baseline; padding: 0 0.2rem; }
.section-header h2 { font-size: 1.05rem; font-weight: 700; color: #eaeaf0; display: flex; align-items: center; gap: 0.5rem; }
.section-icon { font-size: 1rem; }
.section-count { font-size: 0.75rem; color: #707088; }

/* Album row */
.album-row {
  display: flex;
  gap: 1rem;
  overflow-x: auto;
  padding-bottom: 0.5rem;
  scrollbar-width: thin;
  scrollbar-color: #353555 transparent;
}
.album-row::-webkit-scrollbar { height: 6px; }
.album-row::-webkit-scrollbar-thumb { background: #353555; border-radius: 3px; }

.album-card {
  flex: 0 0 160px;
  cursor: pointer;
  transition: transform 0.15s;
}
.album-card:hover { transform: translateY(-3px); }

.cover-wrapper { position: relative; border-radius: 8px; overflow: hidden; }
.cover {
  width: 160px; height: 160px;
  object-fit: cover; display: block;
}
.placeholder {
  width: 160px; height: 160px;
  background: linear-gradient(135deg, #262642, #303050);
  display: flex; align-items: center; justify-content: center;
  font-size: 2.5rem; color: #4a4a65;
}
.cover-overlay {
  position: absolute; inset: 0;
  background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center;
  opacity: 0; transition: opacity 0.2s;
}
.album-card:hover .cover-overlay { opacity: 1; }
.play-overlay-btn {
  width: 44px; height: 44px;
  background: rgba(255,255,255,0.9);
  border: none; border-radius: 50%;
  color: #1a1a2e; font-size: 1.2rem;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
  box-shadow: 0 2px 12px rgba(0,0,0,0.3);
  transition: transform 0.15s;
}
.play-overlay-btn:hover { transform: scale(1.1); }

.card-title {
  margin-top: 0.5rem;
  font-size: 0.82rem; font-weight: 600; color: #eaeaf0;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  max-width: 160px;
}
.card-artist {
  font-size: 0.75rem; color: #a8a8be;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
  max-width: 160px;
}
.card-year { font-size: 0.7rem; color: #707088; }

/* Loading skeleton */
.loading-screen { padding: 1rem 0; }
.loading-grid {
  display: flex; gap: 1rem; overflow: hidden;
}
.skeleton-card { flex: 0 0 160px; }
.skeleton-cover {
  width: 160px; height: 160px;
  background: linear-gradient(90deg, #262642 25%, #303050 50%, #262642 75%);
  background-size: 200% 100%;
  border-radius: 8px;
  animation: shimmer 1.5s infinite;
}
.skeleton-line {
  height: 10px; border-radius: 4px; margin-top: 0.5rem;
  background: linear-gradient(90deg, #262642 25%, #303050 50%, #262642 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}
.w80 { width: 80%; }
.w50 { width: 50%; }
@keyframes shimmer { to { background-position: -200% 0; } }

.error-box {
  background: rgba(240,85,85,0.1);
  border: 1px solid #f05555;
  border-radius: 8px;
  padding: 1.5rem;
  text-align: center;
  color: #f05555;
}
.error-box button {
  margin-top: 0.8rem;
  padding: 0.5rem 1.5rem;
  background: #f05555;
  color: #fff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
}
.error-box button:hover { background: #e04444; }

.empty-home {
  text-align: center;
  padding: 3rem;
  color: #707088;
  font-size: 0.95rem;
}
</style>
