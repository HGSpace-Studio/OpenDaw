<template>
  <div class="h-screen flex flex-col bg-daw-bg">
    <TopBar />

    <div class="flex-1 flex overflow-hidden">
      <TrackList ref="trackListRef" />
      <TimelineView 
        @scroll="handleTimelineScroll"
        @openPianoRoll="openPianoRoll"
      />
    </div>

    <Mixer />

    <PianoRoll 
      :isOpen="pianoRollOpen"
      :trackIndex="pianoRollTrackIndex"
      @close="closePianoRoll"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import TopBar from './components/TopBar.vue'
import TrackList from './components/TrackList.vue'
import TimelineView from './components/TimelineView.vue'
import PianoRoll from './components/PianoRoll.vue'
import Mixer from './components/Mixer.vue'
import { useProjectStore } from './stores/project'

const projectStore = useProjectStore()

const trackListRef = ref<InstanceType<typeof TrackList> | null>(null)

const pianoRollOpen = ref(false)
const pianoRollTrackIndex = ref<number | null>(null)

const handleTimelineScroll = (scrollTop: number) => {
  trackListRef.value?.scrollTop(scrollTop)
}

const openPianoRoll = (trackIndex: number) => {
  pianoRollTrackIndex.value = trackIndex
  pianoRollOpen.value = true
}

const closePianoRoll = () => {
  pianoRollOpen.value = false
  pianoRollTrackIndex.value = null
}

let animationFrame: number | null = null

const updatePlayhead = () => {
  if (projectStore.isPlaying) {
    projectStore.loadState()
  }
  animationFrame = requestAnimationFrame(updatePlayhead)
}

onMounted(() => {
  projectStore.loadState()
  animationFrame = requestAnimationFrame(updatePlayhead)
})

onUnmounted(() => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame)
  }
})

window.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'OPENDAW_UI_PARAM_CHANGED') {
    const { pluginId, paramKey, value } = event.data
    console.log('Plugin param changed:', { pluginId, paramKey, value })
  }
})
</script>
