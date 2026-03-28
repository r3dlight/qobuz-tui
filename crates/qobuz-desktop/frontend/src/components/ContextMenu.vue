<template>
  <Teleport to="body">
    <div class="ctx-overlay" v-if="visible" @click="close" @contextmenu.prevent="close">
      <div class="ctx-menu" :style="{ left: x + 'px', top: y + 'px' }">
        <button v-for="item in items" :key="item.label" @click="item.action(); close()"
          :class="{ danger: item.danger }">
          <span class="ctx-icon" v-if="item.icon">{{ item.icon }}</span>
          {{ item.label }}
        </button>
      </div>
    </div>
  </Teleport>
</template>

<script setup>
import { ref } from 'vue'

const visible = ref(false)
const x = ref(0)
const y = ref(0)
const items = ref([])

function show(event, menuItems) {
  x.value = Math.min(event.clientX, window.innerWidth - 200)
  y.value = Math.min(event.clientY, window.innerHeight - menuItems.length * 36)
  items.value = menuItems
  visible.value = true
}

function close() { visible.value = false }

defineExpose({ show, close })
</script>

<style scoped>
.ctx-overlay { position: fixed; inset: 0; z-index: 200; }
.ctx-menu {
  position: absolute;
  background: #2a2a48;
  border: 1px solid #3a3a58;
  border-radius: 8px;
  padding: 0.3rem;
  min-width: 180px;
  box-shadow: 0 8px 24px rgba(0,0,0,0.5);
}
.ctx-menu button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.8rem;
  background: none;
  border: none;
  color: #eaeaf0;
  cursor: pointer;
  font-size: 0.85rem;
  border-radius: 5px;
  text-align: left;
}
.ctx-menu button:hover { background: #3a3a58; }
.ctx-menu button.danger { color: #f05555; }
.ctx-menu button.danger:hover { background: rgba(240,85,85,0.15); }
.ctx-icon { width: 20px; text-align: center; }
</style>
