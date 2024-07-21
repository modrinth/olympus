<script setup>
import { ref, watch } from 'vue'
import { LogOutIcon, LogInIcon, BoxIcon, FolderSearchIcon, UpdatedIcon } from '@modrinth/assets'
import { Card, Slider, DropdownSelect, Toggle, Modal, Button } from '@modrinth/ui'
import { handleError, useTheming } from '@/store/state'
import { is_dir_writeable, change_config_dir, get, set } from '@/helpers/settings'
import { get as getCreds, logout } from '@/helpers/mr_auth.js'
import JavaSelector from '@/components/ui/JavaSelector.vue'
import ModrinthLoginScreen from '@/components/ui/tutorial/ModrinthLoginScreen.vue'
import { mixpanel_opt_out_tracking, mixpanel_opt_in_tracking } from '@/helpers/mixpanel'
import { open } from '@tauri-apps/api/dialog'
import { getOS } from '@/helpers/utils.js'
import { getVersion } from '@tauri-apps/api/app'
import useMemorySlider from '@/composables/useMemorySlider.js'

const pageOptions = ['Home', 'Library']

const themeStore = useTheming()

const version = await getVersion()

const accessSettings = async () => {
  const settings = await get()

  settings.javaArgs = settings.custom_java_args.join(' ')
  settings.envArgs = settings.custom_env_args.map((x) => x.join('=')).join(' ')

  return settings
}

const fetchSettings = await accessSettings().catch(handleError)

const settings = ref(fetchSettings)
const settingsDir = ref(settings.value.loaded_config_dir)
const { maxMemory, snapPoints } = await useMemorySlider()

watch(
  settings,
  async (oldSettings, newSettings) => {
    if (oldSettings.loaded_config_dir !== newSettings.loaded_config_dir) {
      return
    }

    const setSettings = JSON.parse(JSON.stringify(newSettings))

    if (setSettings.opt_out_analytics) {
      mixpanel_opt_out_tracking()
    } else {
      mixpanel_opt_in_tracking()
    }

    for (const [key, value] of Object.entries(setSettings.java_globals)) {
      if (value?.path === '') {
        value.path = undefined
      }

      if (value?.path) {
        value.path = value.path.replace('java.exe', 'javaw.exe')
      }

      console.log(`${key}: ${value}`)
    }

    setSettings.custom_java_args = setSettings.javaArgs.trim().split(/\s+/).filter(Boolean)
    setSettings.custom_env_args = setSettings.envArgs
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map((x) => x.split('=').filter(Boolean))

    if (!setSettings.hooks.pre_launch) {
      setSettings.hooks.pre_launch = null
    }
    if (!setSettings.hooks.wrapper) {
      setSettings.hooks.wrapper = null
    }
    if (!setSettings.hooks.post_exit) {
      setSettings.hooks.post_exit = null
    }

    await set(setSettings)
  },
  { deep: true },
)

const credentials = ref(await getCreds().catch(handleError))
const loginScreenModal = ref()

async function logOut() {
  await logout().catch(handleError)
  credentials.value = await getCreds().catch(handleError)
}

async function signInAfter() {
  loginScreenModal.value.hide()
  credentials.value = await getCreds().catch(handleError)
}

async function findLauncherDir() {
  const newDir = await open({
    multiple: false,
    directory: true,
    title: 'Select a new app directory',
  })

  const writeable = await is_dir_writeable(newDir)

  if (!writeable) {
    handleError('The selected directory does not have proper permissions for write access.')
    return
  }

  if (newDir) {
    settingsDir.value = newDir
    await refreshDir()
  }
}

async function refreshDir() {
  await change_config_dir(settingsDir.value).catch(handleError)
  settings.value = await accessSettings().catch(handleError)
  settingsDir.value = settings.value.loaded_config_dir
}
</script>

<template>
  <div class="settings-page">
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">General settings</span>
        </h3>
      </div>
      <Modal
        ref="loginScreenModal"
        class="login-screen-modal"
        :noblur="!themeStore.advancedRendering"
      >
        <ModrinthLoginScreen :modal="true" :prev-page="signInAfter" :next-page="signInAfter" />
      </Modal>
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title">Manage account</span>
          <span v-if="credentials" class="label__description">
            You are currently logged in as {{ credentials.user.username }}.
          </span>
          <span v-else> Sign in to your Modrinth account. </span>
        </label>
        <button v-if="credentials" class="btn" @click="logOut">
          <LogOutIcon />
          Sign out
        </button>
        <button v-else class="btn" @click="$refs.loginScreenModal.show()">
          <LogInIcon />
          Sign in
        </button>
      </div>
      <label for="theme">
        <span class="label__title">App directory</span>
        <span class="label__description">
          The directory where the launcher stores all of its files.
        </span>
      </label>
      <div class="app-directory">
        <div class="iconified-input">
          <BoxIcon />
          <input id="appDir" v-model="settingsDir" type="text" class="input" />
          <Button class="r-btn" @click="findLauncherDir">
            <FolderSearchIcon />
          </Button>
        </div>
        <Button large @click="refreshDir">
          <UpdatedIcon />
          Refresh
        </Button>
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Display</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="theme">
          <span class="label__title">Color theme</span>
          <span class="label__description">Change the global launcher color theme.</span>
        </label>
        <DropdownSelect
          id="theme"
          name="Theme dropdown"
          :options="themeStore.themeOptions"
          :default-value="settings.theme"
          :model-value="settings.theme"
          class="theme-dropdown"
          @change="
            (e) => {
              themeStore.setThemeState(e.option.toLowerCase())
              settings.theme = themeStore.selectedTheme
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="advanced-rendering">
          <span class="label__title">Advanced rendering</span>
          <span class="label__description">
            Enables advanced rendering such as blur effects that may cause performance issues
            without hardware-accelerated rendering.
          </span>
        </label>
        <Toggle
          id="advanced-rendering"
          :model-value="themeStore.advancedRendering"
          :checked="themeStore.advancedRendering"
          @update:model-value="
            (e) => {
              themeStore.advancedRendering = e
              settings.advanced_rendering = themeStore.advancedRendering
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="minimize-launcher">
          <span class="label__title">Minimize launcher</span>
          <span class="label__description"
            >Minimize the launcher when a Minecraft process starts.</span
          >
        </label>
        <Toggle
          id="minimize-launcher"
          :model-value="settings.hide_on_process"
          :checked="settings.hide_on_process"
          @update:model-value="
            (e) => {
              settings.hide_on_process = e
            }
          "
        />
      </div>
      <div v-if="getOS() != 'MacOS'" class="adjacent-input">
        <label for="native-decorations">
          <span class="label__title">Native decorations</span>
          <span class="label__description">Use system window frame (app restart required).</span>
        </label>
        <Toggle
          id="native-decorations"
          :model-value="settings.native_decorations"
          :checked="settings.native_decorations"
          @update:model-value="
            (e) => {
              settings.native_decorations = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="opening-page">
          <span class="label__title">Default landing page</span>
          <span class="label__description">Change the page to which the launcher opens on.</span>
        </label>
        <DropdownSelect
          id="opening-page"
          name="Opening page dropdown"
          :options="pageOptions"
          :default-value="settings.default_page"
          :model-value="settings.default_page"
          class="opening-page"
          @change="
            (e) => {
              settings.default_page = e.option
            }
          "
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Resource management</span>
        </h3>
      </div>

      <div class="adjacent-input">
        <label for="max-downloads">
          <span class="label__title">Maximum concurrent downloads</span>
          <span class="label__description"
            >The maximum amount of files the launcher can download at the same time. Set this to a
            lower value if you have a poor internet connection.</span
          >
        </label>
        <Slider
          id="max-downloads"
          v-model="settings.max_concurrent_downloads"
          :min="1"
          :max="10"
          :step="1"
        />
      </div>

      <div class="adjacent-input">
        <label for="max-writes">
          <span class="label__title">Maximum concurrent writes</span>
          <span class="label__description"
            >The maximum amount of files the launcher can write to the disk at once. Set this to a
            lower value if you are frequently getting I/O errors.</span
          >
        </label>
        <Slider
          id="max-writes"
          v-model="settings.max_concurrent_writes"
          :min="1"
          :max="50"
          :step="1"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Privacy</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="opt-out-analytics">
          <span class="label__title">Disable analytics</span>
          <span class="label__description">
            Modrinth collects anonymized analytics and usage data to improve our user experience and
            customize your experience. By enabling this option, you opt out and your data will no
            longer be collected.
          </span>
        </label>
        <Toggle
          id="opt-out-analytics"
          :model-value="settings.opt_out_analytics"
          :checked="settings.opt_out_analytics"
          @update:model-value="
            (e) => {
              settings.opt_out_analytics = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="disable-discord-rpc">
          <span class="label__title">Disable Discord RPC</span>
          <span class="label__description">
            Disables the Discord Rich Presence integration. 'Modrinth' will no longer show up as a
            game or app you are using on your Discord profile. This does not disable any
            instance-specific Discord Rich Presence integrations, such as those added by mods.
          </span>
        </label>
        <Toggle
          id="disable-discord-rpc"
          v-model="settings.disable_discord_rpc"
          :checked="settings.disable_discord_rpc"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Java settings</span>
        </h3>
      </div>
      <label for="java-21">
        <span class="label__title">Java 21 location</span>
      </label>
      <JavaSelector id="java-17" v-model="settings.java_globals.JAVA_21" :version="21" />
      <label for="java-17">
        <span class="label__title">Java 17 location</span>
      </label>
      <JavaSelector id="java-17" v-model="settings.java_globals.JAVA_17" :version="17" />
      <label for="java-8">
        <span class="label__title">Java 8 location</span>
      </label>
      <JavaSelector id="java-8" v-model="settings.java_globals.JAVA_8" :version="8" />
      <hr class="card-divider" />
      <label for="java-args">
        <span class="label__title">Java arguments</span>
      </label>
      <input
        id="java-args"
        v-model="settings.javaArgs"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter java arguments..."
      />
      <label for="env-vars">
        <span class="label__title">Environmental variables</span>
      </label>
      <input
        id="env-vars"
        v-model="settings.envArgs"
        autocomplete="off"
        type="text"
        class="installation-input"
        placeholder="Enter environmental variables..."
      />
      <hr class="card-divider" />
      <div class="adjacent-input">
        <label for="max-memory">
          <span class="label__title">Java memory</span>
          <span class="label__description">
            The memory allocated to each instance when it is ran.
          </span>
        </label>
        <Slider
          id="max-memory"
          v-model="settings.memory.maximum"
          :min="8"
          :max="maxMemory"
          :step="64"
          unit="mb"
          :snap-points="snapPoints"
          :snap-range="512"
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Hooks</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="pre-launch">
          <span class="label__title">Pre launch</span>
          <span class="label__description"> Ran before the instance is launched. </span>
        </label>
        <input
          id="pre-launch"
          v-model="settings.hooks.pre_launch"
          autocomplete="off"
          type="text"
          placeholder="Enter pre-launch command..."
        />
      </div>
      <div class="adjacent-input">
        <label for="wrapper">
          <span class="label__title">Wrapper</span>
          <span class="label__description"> Wrapper command for launching Minecraft. </span>
        </label>
        <input
          id="wrapper"
          v-model="settings.hooks.wrapper"
          autocomplete="off"
          type="text"
          placeholder="Enter wrapper command..."
        />
      </div>
      <div class="adjacent-input">
        <label for="post-exit">
          <span class="label__title">Post exit</span>
          <span class="label__description"> Ran after the game closes. </span>
        </label>
        <input
          id="post-exit"
          v-model="settings.hooks.post_exit"
          autocomplete="off"
          type="text"
          placeholder="Enter post-exit command..."
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Window size</span>
        </h3>
      </div>
      <div class="adjacent-input">
        <label for="fullscreen">
          <span class="label__title">Fullscreen</span>
          <span class="label__description">
            Overwrites the options.txt file to start in full screen when launched.
          </span>
        </label>
        <Toggle
          id="fullscreen"
          :model-value="settings.force_fullscreen"
          :checked="settings.force_fullscreen"
          @update:model-value="
            (e) => {
              settings.force_fullscreen = e
            }
          "
        />
      </div>
      <div class="adjacent-input">
        <label for="width">
          <span class="label__title">Width</span>
          <span class="label__description"> The width of the game window when launched. </span>
        </label>
        <input
          id="width"
          v-model="settings.game_resolution[0]"
          :disabled="settings.force_fullscreen"
          autocomplete="off"
          type="number"
          placeholder="Enter width..."
        />
      </div>
      <div class="adjacent-input">
        <label for="height">
          <span class="label__title">Height</span>
          <span class="label__description"> The height of the game window when launched. </span>
        </label>
        <input
          id="height"
          v-model="settings.game_resolution[1]"
          :disabled="settings.force_fullscreen"
          autocomplete="off"
          type="number"
          class="input"
          placeholder="Enter height..."
        />
      </div>
    </Card>
    <Card>
      <div class="label">
        <h3>
          <span class="label__title size-card-header">About</span>
        </h3>
      </div>
      <div>
        <label>
          <span class="label__title">App version</span>
          <span class="label__description">Theseus v{{ version }} </span>
        </label>
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.settings-page {
  margin: 1rem;
}

.installation-input {
  width: 100% !important;
  flex-grow: 1;
}

.theme-dropdown {
  text-transform: capitalize;
}

.card-divider {
  margin: 1rem 0;
}

:deep(.login-screen-modal) {
  .modal-container .modal-body {
    width: auto;

    .content {
      background: none;
    }
  }
}

.app-directory {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-sm);

  .iconified-input {
    flex-grow: 1;

    input {
      flex-basis: auto;
    }
  }
}
</style>
