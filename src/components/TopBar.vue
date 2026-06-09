<template>
  <header class="h-12 bg-daw-header border-b border-daw-border flex items-center justify-between px-4">
    <nav class="flex gap-1">
      <button 
        v-for="menu in menus" 
        :key="menu"
        class="px-3 py-1.5 text-sm text-gray-300 hover:text-daw-blue hover:bg-daw-blue/10 rounded transition-colors"
        @click="handleMenuClick(menu)"
      >
        {{ menu }}
      </button>
    </nav>

    <div class="text-daw-green font-semibold tracking-wider text-sm">
      Open Daw
    </div>

    <div class="flex items-center gap-2">
      <div class="flex items-center gap-1 mr-4 pr-4 border-r border-daw-border">
        <button 
          @click="handlePlay" 
          class="w-8 h-8 rounded flex items-center justify-center bg-daw-green text-daw-bg hover:scale-105 transition-transform"
          title="播放"
        >
          <span class="text-lg">▶</span>
        </button>
        <button 
          @click="handlePause" 
          class="w-8 h-8 rounded flex items-center justify-center bg-daw-panel border border-daw-border text-gray-300 hover:border-daw-blue hover:text-daw-blue transition-colors"
          title="暂停"
        >
          <span class="text-lg">⏸</span>
        </button>
        <button 
          @click="handleStop" 
          class="w-8 h-8 rounded flex items-center justify-center bg-daw-panel border border-daw-border text-gray-300 hover:border-daw-blue hover:text-daw-blue transition-colors"
          title="停止"
        >
          <span class="text-lg">■</span>
        </button>
        <button 
          class="w-8 h-8 rounded flex items-center justify-center bg-daw-red text-white hover:scale-105 transition-transform"
          title="录制"
        >
          <span class="text-lg">◼</span>
        </button>
      </div>

      <div class="font-mono text-daw-blue text-sm mr-4 min-w-[80px] text-right">
        {{ formattedTime }}
      </div>

      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-500">BPM</span>
        <input 
          type="number" 
          :value="bpm"
          @change="handleBpmChange"
          min="20" 
          max="300"
          class="w-16 bg-daw-panel border border-daw-border rounded px-2 py-1 text-center text-daw-blue text-sm focus:outline-none focus:border-daw-blue"
        >
      </div>

      <div class="flex items-center gap-2 ml-4 pl-4 border-l border-daw-border">
        <span class="text-xs text-gray-500">缩放</span>
        <input 
          type="range" 
          :value="projectStore.zoom"
          @input="handleZoomChange"
          min="25" 
          max="400"
          class="w-24 accent-daw-blue"
        >
        <span class="text-xs text-daw-blue min-w-[40px]">{{ projectStore.zoom }}%</span>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useProjectStore } from '@/stores/project'

const projectStore = useProjectStore()
const bpm = ref(120)

const menus = ['文件', '编辑', '工具', '帮助']

const formattedTime = computed(() => {
  return projectStore.currentTime.toFixed(4) + 's'
})

const handleMenuClick = async (menu: string) => {
  if (menu === '文件') {
    const result = await projectStore.createMidi('New Track')
    console.log(result)
  }
}

const handlePlay = async () => {
  await projectStore.startPlay()
}

const handlePause = async () => {
  await projectStore.pausePlay()
}

const handleStop = async () => {
  await projectStore.stopPlay()
}

const handleBpmChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const value = parseInt(target.value, 10)
  if (!isNaN(value)) {
    bpm.value = value
  }
}

const handleZoomChange = (e: Event) => {
  const target = e.target as HTMLInputElement
  const value = parseInt(target.value, 10)
  if (!isNaN(value)) {
    projectStore.setZoom(value)
  }
}

onMounted(() => {
  projectStore.loadState()
})
</script>
