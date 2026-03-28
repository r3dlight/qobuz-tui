<template>
  <div>
    <p class="loading" v-if="loading">Loading favorites...</p>
    <AlbumGrid v-else :albums="albums" @open="id => emit('open-album', id)" />
  </div>
</template>

<script setup>
import AlbumGrid from './AlbumGrid.vue'
defineProps({
  albums: { type: Array, default: () => [] },
  loading: Boolean
})
const emit = defineEmits(['open-album', 'remove-favorite'])
</script>

<style scoped>
.loading { color: #ffc832; text-align: center; padding: 2rem; }
</style>
