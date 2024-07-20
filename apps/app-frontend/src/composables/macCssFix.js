import { invoke } from '@tauri-apps/api/tauri'
import cssContent from '@/assets/stylesheets/macFix.css?inline'

export async function useCheckDisableMouseover() {
  try {
    // Fetch the CSS content from the Rust backend
    let should_disable_mouseover = await invoke('plugin:utils|should_disable_mouseover')

    if (should_disable_mouseover) {
      // Create a style element and set its content
      const styleElement = document.createElement('style')
      styleElement.innerHTML = cssContent

      // Append the style element to the document's head
      document.head.appendChild(styleElement)
    }
  } catch (error) {
    console.error('Error checking OS version from Rust backend', error)
  }
}
