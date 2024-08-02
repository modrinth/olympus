import { invoke } from '@tauri-apps/api/tauri'
import { tauri } from '@tauri-apps/api'
import { SkinViewer } from 'skinview3d'
import { handleError } from '@/store/state.js'
import { ref } from 'vue'

// Returns SkinCache for user from cache
export async function get_user_skin_data(id) {
  return await invoke('plugin:skin|skin_get_user_skin_data', { id })
}

// Sets users skin
export async function set_skin(skin, arms, creds) {
  return await invoke('plugin:skin|skin_set_skin', { skin, arms, creds })
}

// Sets users cape
export async function set_cape(capeid, token) {
  return await invoke('plugin:skin|skin_set_cape', { capeid, token })
}

// Gets skin filter options
export async function get_filters() {
  Filters.value = await invoke('plugin:skin|skin_get_filters')
}

// Saves skin filter options
export async function save_filters() {
  return await invoke('plugin:skin|skin_save_filters', { filters: Filters.value })
}

// Returns true if image is 64x64
export async function check_image(path) {
  return await invoke('plugin:skin|skin_check_image', { path })
}

// Returns true if skin is in library
export async function check_skin(skin, id) {
  return await invoke('plugin:skin|skin_check_skin', { skin, id })
}

// Returns cape info
export async function get_cape_data(cape, key) {
  return await invoke('plugin:skin|skin_get_cape_data', { cape, key })
}

// Stores all users skin data to cache
export async function cache_users_skins() {
  return await invoke('plugin:skin|skin_cache_users_skins')
}

// Caches new user on login
export async function cache_new_user_skin(creds) {
  return await invoke('plugin:skin|skin_cache_new_user_skin', { creds })
}

// Saves SkinCache to the manager
export async function save_skin(user, data, name, model, skinid) {
  return await invoke('plugin:skin|skin_save_skin', { user, data, name, model, skinid })
}

// Deletes the skin save
export async function delete_skin(id) {
  return await invoke('plugin:skin|skin_delete_skin', { id })
}

// Gets custom skin order
export async function get_order(user) {
  return await invoke('plugin:skin|skin_get_order', { user })
}

// Saves custom skin order
export async function save_order(order, user) {
  return await invoke('plugin:skin|skin_save_order', { order, user })
}

// Gets all saved Skins
export async function get_skins() {
  return await invoke('plugin:skin|skin_get_skins')
}

export async function get_heads() {
  account_heads.value = await invoke('plugin:skin|skin_get_heads')
}

export async function get_launcher_names(path, installer) {
  return await invoke('plugin:skin|skin_get_launcher_names', { path, installer })
}

export async function import_skin(id, path, installer) {
  return await invoke('plugin:skin|skin_import_skin', { id, path, installer })
}

export async function get_render(skinData) {
  let arms = skinData.arms
  if (arms == 'classic') arms = 'default'
  const skinViewer = new SkinViewer({
    width: 144,
    height: 144,
    preserveDrawingBuffer: true,
  })
  skinViewer.camera.rotation.x = -0.62
  skinViewer.camera.rotation.y = 0.534
  skinViewer.camera.rotation.z = 0.348
  skinViewer.camera.position.x = -20
  skinViewer.camera.position.y = 10
  skinViewer.camera.position.z = 38

  let skin = skinData.skin
  if (!skin.startsWith('data:image/png;base64,')) skin = tauri.convertFileSrc(skinData.skin)
  await skinViewer.loadSkin(skin, { model: arms })
  const cape = await get_cape_data(skinData.cape, 'url').catch(handleError)
  if (cape !== 'no cape') {
    await skinViewer.loadCape(cape)
  }

  skinViewer.render()
  // delay is required, if not, sometimes the skin isnt loaded before the canvas is saved to img
  await sleep(50)
  const image = skinViewer.canvas.toDataURL()
  skinViewer.dispose()
  return image
}

export const account_heads = ref({})

export const loaded_skins = ref(false)

export const Filters = ref({
  sort: '',
  filter: ''
})

const sleep = (ms = 0) => new Promise((resolve) => setTimeout(resolve, ms))

export const selectedAccount = ref({})
