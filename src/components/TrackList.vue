<template>
  <div 
    ref="trackListRef"
    class="w-[140px] min-w-[140px] bg-daw-panel border-r border-daw-border overflow-y-auto overflow-x-hidden"
  >
    <div class="h-[28px]" />
    <div 
      v-for="track in projectStore.tracks" 
      :key="track.index"
      class="h-track flex items-center px-3 gap-2 border-b border-daw-border/50 cursor-pointer transition-colors"
      :class="{ 
        'bg-daw-track': projectStore.activeTrackId === track.index,
        'hover:bg-daw-track/50': projectStore.activeTrackId !== track.index
      }"
      @click="handleTrackClick(track.index)"
    >
      <button 
        @click.stop="toggleMute(track.index, track.mute)"
        class="w-4 h-4 rounded-sm border transition-all flex-shrink-0"
        :class="track.mute ? 'bg-daw-red border-daw-red' : 'bg-daw-track border-daw-border hover:border-daw-blue'"
        :title="track.mute ? '取消静音' : '静音'"
      />
      <button 
        @click.stop="toggleSolo(track.index, track.solo)"
        class="w-4 h-4 rounded-sm border transition-all flex-shrink-0"
        :class="track.solo ? 'bg-daw-green border-daw-green' : 'bg-daw-track border-daw-border hover:border-daw-green'"
        :title="track.solo ? '取消独奏' : '独奏'"
      >
        <span v-if="track.solo" class="text-[8px] text-daw-bg font-bold">S</span>
      </button>
      
      <span 
        class="text-xs truncate flex-1"
        :class="{ 'text-daw-green font-medium': track.name === 'Main' }"
      >
        {{ track.name }}
      </span>

      <span 
        class="text-[8px] px-1 rounded flex-shrink-0"
        :class="track.kind === 'midi' ? 'bg-daw-blue/20 text-daw-blue' : 'bg-daw-red/20 text-daw-red'"
      >
        {{ track.kind.toUpperCase() }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useProjectStore } from '@/stores/project'

const projectStore = useProjectStore()
const trackListRef = ref<HTMLElement | null>(null)

defineExpose({
  scrollTop
})

function scrollTop(value: number) {
  if (trackListRef.value) {
    trackListRef.value.scrollTop = value
  }
}

const handleTrackClick = (trackIndex: number) => {
  projectStore.setActiveTrack(trackIndex)
}

const toggleMute = async (trackIndex: number, currentMuted: boolean) => {
  await projectStore.updateTrack(trackIndex, { mute: !currentMuted })
}

const toggleSolo = async (trackIndex: number, currentSolo: boolean) => {
  await projectStore.updateTrack(trackIndex, { solo: !currentSolo })
}
</script>
