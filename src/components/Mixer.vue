<template>
  <div class="h-48 bg-daw-header border-t border-daw-border flex">
    <div class="w-32 min-w-[128px] bg-daw-panel border-r border-daw-border flex flex-col">
      <div class="h-8 border-b border-daw-border flex items-center justify-center text-xs text-gray-500">
        混音台
      </div>
      <div class="flex-1 flex items-center justify-center">
        <button 
          class="text-xs text-daw-blue hover:text-daw-green transition-colors"
          @click="toggleMixer"
        >
          {{ isExpanded ? '收起' : '展开' }}
        </button>
      </div>
    </div>

    <div class="flex-1 flex overflow-x-auto">
      <div 
        v-for="track in projectStore.tracks" 
        :key="track.index"
        class="w-20 min-w-[80px] border-r border-daw-border/50 flex flex-col"
      >
        <div class="h-8 border-b border-daw-border flex items-center justify-center text-[10px] text-gray-400 truncate px-1">
          {{ track.name }}
        </div>

        <div class="flex-1 flex flex-col items-center justify-center gap-2 py-2">
          <div class="flex flex-col items-center gap-1">
            <span class="text-[8px] text-gray-500">PAN</span>
            <div 
              class="w-8 h-8 rounded-full bg-daw-track border border-daw-border relative cursor-pointer"
              :style="{ transform: `rotate(${track.pan * 1.8}deg)` }"
              @mousedown="startPanDrag($event, track.index)"
            >
              <div class="absolute top-1 left-1/2 w-0.5 h-2 bg-daw-blue rounded -translate-x-1/2" />
            </div>
            <span class="text-[9px] text-daw-blue">{{ Math.round(track.pan) }}</span>
          </div>

          <div class="flex flex-col items-center gap-1">
            <span class="text-[8px] text-gray-500">VOL</span>
            <div class="relative h-16 w-4 bg-daw-track rounded border border-daw-border">
              <div class="absolute top-1/2 left-0 right-0 h-px bg-daw-border" />
              <div 
                class="absolute left-0 right-0 h-3 bg-daw-blue/80 rounded-sm cursor-pointer hover:bg-daw-blue transition-colors"
                :style="{ top: `${100 - track.gain * 100}%`, height: '12px' }"
                @mousedown="startVolumeDrag($event, track.index)"
              />
            </div>
            <span class="text-[9px] text-daw-blue">{{ Math.round(track.gain * 100) }}</span>
          </div>
        </div>

        <div class="h-8 border-t border-daw-border flex items-center justify-center gap-1">
          <button 
            @click="toggleMute(track.index)"
            class="w-5 h-5 rounded-sm text-[8px] font-bold transition-all"
            :class="track.mute ? 'bg-daw-red text-white' : 'bg-daw-track border border-daw-border text-gray-500 hover:border-daw-blue'"
          >
            M
          </button>
          <button 
            @click="toggleSolo(track.index)"
            class="w-5 h-5 rounded-sm text-[8px] font-bold transition-all"
            :class="track.solo ? 'bg-daw-green text-daw-bg' : 'bg-daw-track border border-daw-border text-gray-500 hover:border-daw-green'"
          >
            S
          </button>
        </div>
      </div>

      <div class="w-24 min-w-[96px] border-r border-daw-border/50 bg-daw-panel/50 flex flex-col">
        <div class="h-8 border-b border-daw-border flex items-center justify-center text-[10px] text-daw-green font-medium">
          主输出
        </div>
        <div class="flex-1 flex flex-col items-center justify-center gap-2 py-2">
          <div class="flex flex-col items-center gap-1">
            <span class="text-[8px] text-gray-500">MASTER</span>
            <div class="relative h-16 w-4 bg-daw-track rounded border border-daw-green/50">
              <div class="absolute top-1/2 left-0 right-0 h-px bg-daw-border" />
              <div 
                class="absolute left-0 right-0 h-3 bg-daw-green/80 rounded-sm"
                style="top: 20%; height: 12px"
              />
            </div>
            <span class="text-[9px] text-daw-green">85</span>
          </div>
        </div>
      </div>
    </div>

    <div class="w-48 bg-daw-panel border-l border-daw-border p-3">
      <div class="text-[10px] text-daw-green mb-2 font-medium">Tone4 (Bass)</div>
      <select class="w-full bg-daw-track border border-daw-border rounded px-2 py-1 text-xs text-gray-300 mb-3 focus:outline-none focus:border-daw-blue">
        <option>Bass - Warm</option>
        <option>Bass - Punchy</option>
        <option>Bass - Sub</option>
        <option>Bass - Slap</option>
      </select>
      <div class="flex gap-2">
        <button class="flex-1 py-1 text-[10px] bg-daw-green/20 text-daw-green rounded hover:bg-daw-green/30 transition-colors">
          Reverb
        </button>
        <button class="flex-1 py-1 text-[10px] bg-daw-track border border-daw-border text-gray-500 rounded hover:border-daw-blue hover:text-daw-blue transition-colors">
          Delay
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useProjectStore } from '@/stores/project'

const projectStore = useProjectStore()
const isExpanded = ref(true)

const toggleMixer = () => {
  isExpanded.value = !isExpanded.value
}

const toggleMute = async (trackIndex: number) => {
  const track = projectStore.tracks[trackIndex]
  if (track) {
    await projectStore.updateTrack(trackIndex, { mute: !track.mute })
  }
}

const toggleSolo = async (trackIndex: number) => {
  const track = projectStore.tracks[trackIndex]
  if (track) {
    await projectStore.updateTrack(trackIndex, { solo: !track.solo })
  }
}

const startVolumeDrag = (e: MouseEvent, trackIndex: number) => {
  e.preventDefault()

  const track = projectStore.tracks[trackIndex]
  if (!track) return

  const container = (e.target as HTMLElement).parentElement
  if (!container) return

  const startY = e.clientY
  const startGain = track.gain

  const handleMouseMove = (moveEvent: MouseEvent) => {
    const deltaY = startY - moveEvent.clientY
    const newGain = Math.max(0, Math.min(1, startGain + deltaY * 0.01))
    projectStore.updateTrack(trackIndex, { gain: newGain })
  }

  const handleMouseUp = () => {
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

const startPanDrag = (e: MouseEvent, trackIndex: number) => {
  e.preventDefault()

  const track = projectStore.tracks[trackIndex]
  if (!track) return

  const startY = e.clientY
  const startPan = track.pan

  const handleMouseMove = (moveEvent: MouseEvent) => {
    const deltaY = startY - moveEvent.clientY
    const newPan = Math.max(-1, Math.min(1, startPan + deltaY * 0.01))
    projectStore.updateTrack(trackIndex, { pan: newPan })
  }

  const handleMouseUp = () => {
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}
</script>
