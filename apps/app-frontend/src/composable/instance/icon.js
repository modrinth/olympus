import { toValue, computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/tauri'

export function useInstanceIcon(instance) {
  return computed(() => {
    let instanceValue = toValue(instance)

    if (instanceValue.metadata.icon) {
      if (instanceValue.metadata.icon.startsWith('http')) {
        return instanceValue.metadata.icon
      } else {
        return convertFileSrc(instanceValue.metadata?.icon)
      }
    }

    if (instanceValue.metadata.icon_url) {
      return instanceValue.metadata.icon_url
    }
    return null
  })
}
