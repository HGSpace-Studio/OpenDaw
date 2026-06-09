<template>
  <div class="flex-1 flex flex-col overflow-hidden bg-daw-bg">
    <div class="h-7 bg-daw-header border-b border-daw-border flex items-end relative">
      <div 
        class="absolute top-0 left-0 h-full"
        :style="{ width: `${totalWidth}px` }"
      >
        <div 
          v-for="second in timeMarkers" 
          :key="second"
          class="absolute top-0 h-full flex flex-col justify-end pb-1"
          :style="{ left: `${second * projectStore.pixelsPerSecond}px` }"
        >
          <span class="text-[10px] text-gray-500 ml-1">{{ second }}s</span>
          <div class="w-px h-3 bg-daw-border" />
        </div>
      </div>
      <div 
        v-if="projectStore.isPlaying"
        class="absolute top-0 w-px h-full bg-daw-green/50 z-10"
        :style="{ left: `${projectStore.playheadPosition}px` }"
      />
    </div>

    <div 
      ref="timelineRef"
      class="flex-1 overflow-auto relative"
      @scroll="handleScroll"
    >
      <div 
        class="relative"
        :style="{ width: `${totalWidth}px`, minHeight: '100%' }"
      >
        <div class="absolute inset-0 pointer-events-none">
          <div 
            v-for="second in timeMarkers" 
            :key="`grid-${second}`"
            class="absolute top-0 bottom-0 w-px bg-daw-border/30"
            :style="{ left: `${second * projectStore.pixelsPerSecond}px` }"
          />
        </div>

        <div 
          v-for="track in projectStore.tracks" 
          :key="`row-${track.index}`"
          class="h-track border-b border-daw-border/50 relative"
          :class="{ 'bg-daw-track/30': projectStore.activeTrackId === track.index }"
          @click="handleRowClick(track.index)"
        >
          <div 
            v-if="track.duration_seconds > 0"
            class="absolute top-1 h-[38px] rounded cursor-pointer transition-all hover:brightness-125"
            :class="track.kind === 'midi' ? 'bg-daw-blue/30 border border-daw-blue/60' : 'bg-daw-red/30 border border-daw-red/60'"
            :style="{
              left: `${track.offset_seconds * projectStore.pixelsPerSecond}px`,
              width: `${track.duration_seconds * projectStore.pixelsPerSecond}px`
            }"
            @dblclick="handleClipDoubleClick(track.index)"
            @mousedown="startDrag($event, track.index)"
          >
            <div class="px-2 py-1 text-[10px] font-medium truncate">
              {{ track.path.includes('/') ? track.path.split('/').pop() : track.name }}
            </div>
            <div v-if="track.kind === 'midi' && track.notes && track.notes.length > 0" class="absolute inset-x-1 bottom-1 flex gap-[1px] flex-wrap">
              <div 
                v-for="(_, i) in track.notes.slice(0, 20)" 
                :key="i"
                class="w-1 bg-daw-blue/60 rounded-sm"
                :style="{ height: '3px' }"
              />
            </div>
          </div>
        </div>

        <div 
          class="absolute top-0 bottom-0 w-0.5 bg-daw-red z-20 pointer-events-none"
          :style="{ left: `${projectStore.playheadPosition}px` }"
        >
          <div class="w-3 h-3 bg-daw-red rounded-full absolute -top-1 -left-[4px]" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProjectStore } from '@/stores/project'

const emit = defineEmits<{
  (e: 'scroll', scrollTop: number): void
  (e: 'openPianoRoll', trackIndex: number): void
}>()

const projectStore = useProjectStore()
const timelineRef = ref<HTMLElement | null>(null)

const totalWidth = computed(() => Math.max(10 * projectStore.pixelsPerSecond, projectStore.projectDuration * projectStore.pixelsPerSecond + 100))
const timeMarkers = computed(() => {
  const markers = []
  const maxSeconds = Math.ceil(projectStore.projectDuration) + 10
  for (let i = 0; i <= maxSeconds; i++) {
    markers.push(i)
  }
  return markers
})

const handleScroll = (e: Event) => {
  const target = e.target as HTMLElement
  emit('scroll', target.scrollTop)
}

const handleRowClick = (trackIndex: number) => {
  projectStore.setActiveTrack(trackIndex)
}

const handleClipDoubleClick = (trackIndex: number) => {
  const track = projectStore.tracks[trackIndex]
  if (track && track.kind === 'midi') {
    emit('openPianoRoll', trackIndex)
  }
}

const startDrag = (e: MouseEvent, trackIndex: number) => {
  e.preventDefault()
  const startX = e.clientX
  const track = projectStore.tracks[trackIndex]
  if (!track) return

  const originalOffset = track.offset_seconds

  const handleMouseMove = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - startX
    const deltaSeconds = deltaX / projectStore.pixelsPerSecond
    projectStore.updateTrack(trackIndex, { offset_seconds: Math.max(0, originalOffset + deltaSeconds) })
  }

  const handleMouseUp = () => {
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}
</script>
