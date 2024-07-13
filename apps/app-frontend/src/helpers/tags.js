/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Gets tag bundle of all tags
export async function get_tag_bundle() {
  return await invoke('plugin:tags|tags_get_tag_bundle')
}

// Gets cached category tags
export async function get_categories() {
  return await invoke('plugin:tags|tags_get_categories')
}

// Gets cached loaders tags
export async function get_loaders() {
  return await invoke('plugin:tags|tags_get_loaders')
}

// Gets cached game_versions tags
export async function get_game_versions() {
  return await invoke('plugin:tags|tags_get_game_versions')
}

// Gets cached donation_platforms tags
export async function get_donation_platforms() {
  return await invoke('plugin:tags|tags_get_donation_platforms')
}

// Gets cached licenses tags
export async function get_report_types() {
  return await invoke('plugin:tags|tags_get_report_types')
}

// Sorts alphabetically, but correctly identifies 8x, 128x, 256x, etc
// identifier[0], then if it ties, identifier[1], etc
export function sortByNameOrNumber(sortable, identifiers) {
  sortable.sort((a, b) => {
    for (let identifier of identifiers) {
      let aNum = parseFloat(a[identifier])
      let bNum = parseFloat(b[identifier])
      if (isNaN(aNum) && isNaN(bNum)) {
        // Both are strings, sort alphabetically
        let stringComp = a[identifier].localeCompare(b[identifier])
        if (stringComp != 0) return stringComp
      } else if (!isNaN(aNum) && !isNaN(bNum)) {
        // Both are numbers, sort numerically
        let numComp = aNum - bNum
        if (numComp != 0) return numComp
      } else {
        // One is a number and one is a string, numbers go first
        let numStringComp = isNaN(aNum) ? 1 : -1
        if (numStringComp != 0) return numStringComp
      }
    }
    return 0
  })
  return sortable
}
