<template>
  <div class="login">
    <div class="login-card">
      <h1><span class="brand">Qobuz</span> Desktop</h1>
      <p class="subtitle">Sign in to your account</p>
      <form @submit.prevent="doLogin">
        <label>Email</label>
        <input v-model="email" type="email" placeholder="your@email.com" autofocus />
        <label>Password</label>
        <input v-model="password" type="password" placeholder="Password" />
        <button type="submit" :disabled="loading">
          {{ loading ? 'Connecting...' : 'Sign In' }}
        </button>
        <p class="error" v-if="error">{{ error }}</p>
      </form>
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const emit = defineEmits(['login-success'])
const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

async function doLogin() {
  loading.value = true
  error.value = ''
  try {
    await invoke('login', { email: email.value, password: password.value })
    emit('login-success')
  } catch (e) {
    error.value = String(e)
  }
  loading.value = false
}
</script>

<style scoped>
.login {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: #1a1a2e;
}
.login-card {
  background: #262642;
  border: 1px solid #353555;
  border-radius: 12px;
  padding: 2.5rem;
  width: 380px;
  text-align: center;
}
h1 { font-size: 1.8rem; margin-bottom: 0.3rem; }
.brand { color: #6495ed; }
.subtitle { color: #9090a8; margin-bottom: 1.5rem; }
form { display: flex; flex-direction: column; gap: 0.5rem; text-align: left; }
label { color: #9090a8; font-size: 0.85rem; }
input {
  padding: 0.7rem;
  background: #202038;
  border: 1px solid #353555;
  border-radius: 6px;
  color: #eaeaf0;
  font-size: 0.95rem;
  outline: none;
  margin-bottom: 0.5rem;
}
input:focus { border-color: #6495ed; }
button {
  padding: 0.8rem;
  background: #6495ed;
  color: #1a1a2e;
  border: none;
  border-radius: 6px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  margin-top: 0.5rem;
}
button:hover { background: #7ba5f7; }
button:disabled { opacity: 0.6; cursor: wait; }
.error { color: #f05555; font-size: 0.85rem; text-align: center; margin-top: 0.5rem; }
</style>
