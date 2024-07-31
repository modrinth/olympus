import { get_full_path, get_mod_full_path } from '@/helpers/profile'
import { invoke } from '@tauri-apps/api/tauri'

export async function isDev() {
  return await invoke('is_dev')
}

/// Show Main Window - called once after first frontend rendered
export async function showMainWindow() {
  return await invoke('show_window')
}

// One of 'Windows', 'Linux', 'MacOS'
export async function getOS() {
  return await invoke('plugin:utils|get_os')
}

export async function showInFolder(path) {
  return await invoke('plugin:utils|show_in_folder', { path })
}

export async function showLauncherLogsFolder() {
  return await invoke('plugin:utils|show_launcher_logs_folder', {})
}

// Opens a profile's folder in the OS file explorer
export async function showProfileInFolder(path) {
  const fullPath = await get_full_path(path)
  return await showInFolder(fullPath)
}

export async function highlightModInProfile(profilePath, projectPath) {
  const fullPath = await get_mod_full_path(profilePath, projectPath)
  return await showInFolder(fullPath)
}

export const releaseColor = (releaseType) => {
  switch (releaseType) {
    case 'release':
      return 'green'
    case 'beta':
      return 'orange'
    case 'alpha':
      return 'red'
    default:
      return ''
  }
}

export function debounce(fn, wait) {
  let timer
  return function (...args) {
    if (timer) {
      clearTimeout(timer) // clear any pre-existing timer
    }
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const context = this // get the current context
    timer = setTimeout(() => {
      fn.apply(context, args) // call the function if time expires
    }, wait)
  }
}
