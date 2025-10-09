<template>
  <div class="user-avatar-wrapper">
    <button class="user-avatar" @click="toggleDropdown">
      <img 
        v-if="avatar" 
        :src="avatar" 
        :alt="name"
        class="avatar-image"
      />
      <div v-else class="avatar-placeholder">
        {{ initials }}
      </div>
    </button>

    <Transition name="dropdown">
      <div v-if="showDropdown" class="dropdown-menu">
        <div class="user-info">
          <div class="user-name">{{ name }}</div>
          <div class="user-email">{{ email }}</div>
        </div>
        
        <div class="dropdown-divider"></div>
        
        <button class="dropdown-item" @click="goToProfile">
          <span class="item-icon">👤</span>
          个人中心
        </button>
        
        <button class="dropdown-item" @click="goToSettings">
          <span class="item-icon">⚙️</span>
          设置
        </button>
        
        <div class="dropdown-divider"></div>
        
        <button class="dropdown-item logout" @click="handleLogout">
          <span class="item-icon">🚪</span>
          退出登录
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const props = defineProps<{
  name: string
  email?: string
  avatar?: string
}>()

const router = useRouter()
const authStore = useAuthStore()

const showDropdown = ref(false)

const initials = computed(() => {
  if (!props.name) return '?'
  const words = props.name.split(' ')
  if (words.length >= 2) {
    return (words[0][0] + words[1][0]).toUpperCase()
  }
  return props.name.slice(0, 2).toUpperCase()
})

function toggleDropdown() {
  showDropdown.value = !showDropdown.value
}

function closeDropdown() {
  showDropdown.value = false
}

function goToProfile() {
  closeDropdown()
  router.push('/profile')
}

function goToSettings() {
  closeDropdown()
  router.push('/settings')
}

async function handleLogout() {
  closeDropdown()
  await authStore.logout()
  router.push('/login')
}

// Close dropdown when clicking outside
function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.user-avatar-wrapper')) {
    closeDropdown()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
.user-avatar-wrapper {
  position: relative;
}

.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: 2px solid #e0e0e0;
  background: none;
  padding: 0;
  cursor: pointer;
  transition: all 0.3s;
}

.user-avatar:hover {
  border-color: #667eea;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
}

.avatar-image {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
}

.avatar-placeholder {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 16px;
}

.dropdown-menu {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 220px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  padding: 8px 0;
  z-index: 1000;
}

.user-info {
  padding: 12px 16px;
}

.user-name {
  font-weight: 600;
  color: #1a1a1a;
  margin-bottom: 4px;
}

.user-email {
  font-size: 13px;
  color: #666;
}

.dropdown-divider {
  height: 1px;
  background-color: #e0e0e0;
  margin: 8px 0;
}

.dropdown-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  background: none;
  border: none;
  text-align: left;
  font-size: 14px;
  color: #444;
  cursor: pointer;
  transition: background-color 0.2s;
}

.dropdown-item:hover {
  background-color: #f5f5f5;
}

.dropdown-item.logout {
  color: #c33;
}

.dropdown-item.logout:hover {
  background-color: #fee;
}

.item-icon {
  font-size: 18px;
}

/* Dropdown transition */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>

