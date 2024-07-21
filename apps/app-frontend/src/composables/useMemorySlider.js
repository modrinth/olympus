import { ref, computed } from 'vue'
import { get_max_memory } from '@/helpers/jre.js'
import { handleError } from '@/store/notifications.js'

export default async function () {
  const maxMemory = ref(Math.floor((await get_max_memory().catch(handleError)) / 1024))

  const snapPoints = computed(() => {
    let points = []
    let memory = 2048

    while (memory <= maxMemory.value) {
      points.push(memory)
      memory *= 2
    }

    return points
  })

  return { maxMemory, snapPoints }
}
