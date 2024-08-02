/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

// Example function:
// User goes to auth_url to complete flow, and when completed, authenticate_await_completion() returns the credentials
// export async function authenticate() {
//   const auth_url = await authenticate_begin_flow()
//   console.log(auth_url)
//   await authenticate_await_completion()
// }

/// Authenticate a user with Hydra - part 1
/// This begins the authentication flow quasi-synchronously
/// This returns a DeviceLoginSuccess object, with two relevant fields:
///  - verification_uri: the URL to go to to complete the flow
///  - user_code: the code to enter on the verification_uri page
export async function login() {
  return await invoke('auth_login')
}

/// Retrieves the default user
/// user is UUID
export async function get_default_user() {
  return await invoke('plugin:auth|auth_get_default_user')
}

/// Updates the default user
/// user is UUID
export async function set_default_user(user) {
  return await invoke('plugin:auth|auth_set_default_user', { user })
}

/// Remove a user account from the database
/// user is UUID
export async function remove_user(user) {
  return await invoke('plugin:auth|auth_remove_user', { user })
}

/// Returns a list of users
/// Returns an Array of Credentials
export async function users() {
  return await invoke('plugin:auth|auth_users')
}

// Refreshes a credential by UUID
export async function refresh(user) {
  return await invoke('plugin:auth|auth_refresh', { user })
}
