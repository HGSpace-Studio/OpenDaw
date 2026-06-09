<template>
  <Teleport to="body">
    <div v-if="isOpen" class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center" @click.self="close">
      <div class="bg-daw-panel rounded-lg shadow-2xl border border-daw-border w-[900px] h-[600px] flex flex-col overflow-hidden">
        <div class="h-10 bg-daw-header border-b border-daw-border flex items-center justify-between px-4">
          <span class="text-daw-green font-medium text-sm">钢琴卷帘 - {{ trackName }}</span>
          <button 
            @click="close"
            class="w-6 h-6 rounded flex items-center justify-center text-gray-500 hover:text-white hover:bg-daw-border transition-colors"
          >
            ✕
          </button>
        </div>

        <div class="flex-1 flex overflow-hidden">
          <div class="w-16 bg-daw-header border-r border-daw-border overflow-y-auto overflow-x-hidden">
            <div class="h-[22px] border-b border-daw-border" />
            <div 
              v-for="pitch in pitches" 
              :key="pitch"
              class="h-[22px] flex items-center justify-end pr-2 text-[10px] text-gray-500 border-b border-daw-border/30"
              :class="{ 'text-daw-blue font-medium': pitch.startsWith('C') }"
            >
              {{ pitch }}
            </div>
          </div>

          <div class="flex-1 overflow-auto relative" ref="gridRef" @scroll="handleGridScroll">
            <div 
              class="relative"
              :style="{ width: `${gridWidth}px`, height: `${pitches.length * 22}px` }"
            >
              <div class="absolute inset-0 pointer-events-none">
                <div 
                  v-for="(_, i) in pitches" 
                  :key="`h-${i}`"
                  class="absolute left-0 right-0 h-px bg-daw-border/30"
                  :style="{ top: `${i * 22}px` }"
                />
                <div 
                  v-for="beat in beatMarkers" 
                  :key="`v-${beat}`"
                  class="absolute top-0 bottom-0 h-full w-px"
                  :class="beat % 4 === 0 ? 'bg-daw-border/60' : 'bg-daw-border/20'"
                  :style="{ left: `${beat * beatWidth}px` }"
                />
              </div>

              <div 
                v-for="(note, noteIndex) in currentNotes" 
                :key="noteIndex"
                class="absolute bg-daw-blue rounded-sm cursor-pointer hover:brightness-125 shadow-md"
                :style="getNoteStyle(note)"
                @mousedown="startNoteDrag($event, noteIndex)"
              />

              <div 
                class="absolute inset-0 cursor-crosshair"
                @dblclick="handleGridDoubleClick"
              />
            </div>

            <div 
              class="absolute top-0 bottom-0 w-0.5 bg-daw-red z-10 pointer-events-none"
              :style="{ left: `${currentTrackOffset * pixelsPerBeat + 16}px` }"
            />
          </div>
        </div>

        <div class="h-10 bg-daw-header border-t border-daw-border flex items-center gap-4 px-4">
          <span class="text-xs text-gray-500">片段位置: {{ currentTrackOffset.toFixed(2) }}s</span>
          <span class="text-xs text-gray-500">|</span>
          <span class="text-xs text-gray-500">音符数: {{ currentNotes.length }}</span>
          <div class="flex-1" />
          <button 
            @click="deleteSelectedNote"
            class="px-3 py-1 text-xs bg-daw-red/20 text-daw-red rounded hover:bg-daw-red/30 transition-colors"
          >
            删除选中
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useProjectStore } from '@/stores/project'
import type { MidiNote } from '@/api/backend'

const props = defineProps<{
  isOpen: boolean
  trackIndex: number | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const projectStore = useProjectStore()
const gridRef = ref<HTMLElement | null>(null)

const pitches = [
  'D9', 'C#9', 'C9', 'B8', 'A#8', 'A8', 'G#8', 'G8', 'F#8', 'F8',
  'E8', 'D#8', 'D8', 'C#8', 'C8', 'B7', 'A#7', 'A7', 'G#7', 'G7',
  'F#7', 'F7', 'E7', 'D#7', 'D7', 'C#7', 'C7', 'B6', 'A#6', 'A6',
  'G#6', 'G6', 'F#6', 'F6', 'E6', 'D#6', 'D6', 'C#6', 'C6', 'B5',
  'A#5', 'A5', 'G#5', 'G5', 'F#5', 'F5', 'E5', 'D#5', 'D5', 'C#5',
  'C5', 'B4', 'A#4', 'A4', 'G#4', 'G4', 'F#4', 'F4', 'E4', 'D#4',
  'D4', 'C#4', 'C4', 'B3', 'A#3', 'A3', 'G#3', 'G3', 'F#3', 'F3',
  'E3', 'D#3', 'D3', 'C#3', 'C3', 'B2', 'A#2', 'A2', 'G#2', 'G2',
  'F#2', 'F2', 'E2', 'D#2', 'D2', 'C#2', 'C2', 'B1', 'A#1', 'A1',
  'G#1', 'G1', 'F#1', 'F1', 'E1', 'D#1', 'D1', 'C#1', 'C1', 'D0', 'C#0', 'C0'
]

const beatWidth = 40
const pixelsPerBeat = 40
const rowHeight = 22

const currentTrack = computed(() => {
  if (props.trackIndex === null) return null
  return projectStore.tracks[props.trackIndex]
})

const trackName = computed(() => currentTrack.value?.name || '')
const currentNotes = computed(() => currentTrack.value?.notes || [])
const currentTrackOffset = computed(() => currentTrack.value?.offset_seconds || 0)
const gridWidth = computed(() => (currentTrack.value?.duration_seconds || 4) * 4 * beatWidth)
const beatMarkers = computed(() => Array.from({ length: Math.ceil((currentTrack.value?.duration_seconds || 4) * 4) + 1 }, (_, i) => i))

const getNoteStyle = (note: MidiNote) => {
  const noteNumber = note.note
  const pitchIndex = 127 - noteNumber
  const clampedIndex = Math.max(0, Math.min(pitches.length - 1, pitchIndex))
  
  return {
    left: `${(note.start_seconds) * pixelsPerBeat}px`,
    top: `${clampedIndex * rowHeight + 2}px`,
    width: `${(note.end_seconds - note.start_seconds) * pixelsPerBeat - 2}px`,
    height: `${rowHeight - 4}px`
  }
}

const handleGridScroll = () => {}

const handleGridDoubleClick = (e: MouseEvent) => {
  if (props.trackIndex === null) return
  
  const gridRect = gridRef.value?.getBoundingClientRect()
  if (!gridRect) return

  const x = e.clientX - gridRect.left + (gridRef.value?.scrollLeft || 0) - 16
  const y = e.clientY - gridRect.top + (gridRef.value?.scrollTop || 0)

  const startTime = Math.max(0, x / pixelsPerBeat)
  const pitchIndex = Math.floor(y / rowHeight)
  const pitchNumber = 127 - Math.max(0, Math.min(pitches.length - 1, pitchIndex))

  const newNotes = [...(currentNotes.value || []), {
    note: pitchNumber,
    velocity: 100,
    channel: 0,
    start_seconds: startTime,
    end_seconds: startTime + 0.5
  }]

  projectStore.updateTrackNotes(props.trackIndex, newNotes, currentTrack.value?.duration_seconds || 2)
}

const startNoteDrag = (e: MouseEvent, noteIndex: number) => {
  e.preventDefault()
  e.stopPropagation()

  const startX = e.clientX
  const startY = e.clientY
  const note = currentNotes.value[noteIndex]
  if (!note) return

  const originalStart = note.start_seconds
  const originalNote = note.note

  const handleMouseMove = (moveEvent: MouseEvent) => {
    const deltaX = moveEvent.clientX - startX
    const deltaY = moveEvent.clientY - startY

    const deltaTime = deltaX / pixelsPerBeat
    const deltaNote = -Math.round(deltaY / rowHeight)
    const newNote = Math.max(0, Math.min(127, originalNote + deltaNote))

    const updatedNotes = [...(currentNotes.value || [])]
    updatedNotes[noteIndex] = {
      ...note,
      start_seconds: Math.max(0, originalStart + deltaTime),
      note: newNote
    }

    projectStore.updateTrackNotes(props.trackIndex!, updatedNotes, currentTrack.value?.duration_seconds || 2)
  }

  const handleMouseUp = () => {
    document.removeEventListener('mousemove', handleMouseMove)
    document.removeEventListener('mouseup', handleMouseUp)
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

const deleteSelectedNote = () => {}

const close = () => {
  emit('close')
}

watch(() => props.isOpen, (newVal) => {
  if (newVal && gridRef.value) {
    gridRef.value.scrollTop = 0
  }
})
</script>
