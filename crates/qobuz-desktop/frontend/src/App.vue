<template>
  <div class="app">
    <header>
      <h1>Qobuz Desktop</h1>
      <p class="subtitle">Work in progress — Vue.js frontend for qobuz-lib</p>
    </header>
    <main>
      <p>The Tauri backend is ready with {{ commandCount }} commands available.</p>
      <p>Start building the UI by editing <code>crates/qobuz-desktop/frontend/src/App.vue</code></p>
    </main>
  </div>
</template>

<script setup>
const commandCount = 18;
</script>

<style>
body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #12121a;
  color: #e6e6f0;
}
.app {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  text-align: center;
}
h1 {
  color: #6495ed;
}
.subtitle {
  color: #787890;
}
code {
  background: #1e1e28;
  padding: 0.2em 0.5em;
  border-radius: 4px;
  color: #00d2d3;
}
</style>
