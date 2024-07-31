<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { RouterView, RouterLink, useRouter, useRoute } from 'vue-router'
import {
  HomeIcon,
  SearchIcon,
  LibraryIcon,
  PlusIcon,
  SettingsIcon,
  FileIcon,
  XIcon,
} from '@modrinth/assets'
import { Button, Notifications, Card } from '@modrinth/ui'
import { useLoading, useTheming } from '@/store/state'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { get } from '@/helpers/settings'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import ModrinthLoadingIndicator from '@/components/modrinth-loading-indicator'
import { handleError, useNotifications } from '@/store/notifications.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { MinimizeIcon, MaximizeIcon, ChatIcon } from '@/assets/icons'
import { type } from '@tauri-apps/api/os'
import { appWindow } from '@tauri-apps/api/window'
import { isDev, getOS, showLauncherLogsFolder, showMainWindow } from '@/helpers/utils.js'
import {
  mixpanel_track,
  mixpanel_init,
  mixpanel_opt_out_tracking,
  mixpanel_is_loaded,
} from '@/helpers/mixpanel'
import { saveWindowState, StateFlags } from 'tauri-plugin-window-state-api'
import { getVersion } from '@tauri-apps/api/app'
import { window as TauriWindow } from '@tauri-apps/api'
import { TauriEvent } from '@tauri-apps/api/event'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import OnboardingScreen from '@/components/ui/tutorial/OnboardingScreen.vue'
import { install_from_file } from './helpers/pack'
import { useError } from '@/store/error.js'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import { useInstall } from '@/store/install.js'

const themeStore = useTheming()
const urlModal = ref(null)
const isLoading = ref(true)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const showOnboarding = ref(false)
const nativeDecorations = ref(false)

const onboardingVideo = ref()

const failureText = ref(null)
const os = ref('')

defineExpose({
  initialize: async () => {
    const {
      native_decorations,
      theme,
      telemetry,
      collapsed_navigation,
      advanced_rendering,
      onboarded,
    } = await get()
    // video should play if the user is not on linux, and has not onboarded
    os.value = await getOS()
    const dev = await isDev()
    const version = await getVersion()
    showOnboarding.value = !onboarded

    nativeDecorations.value = native_decorations
    if (os.value !== 'MacOS') appWindow.setDecorations(native_decorations)

    themeStore.setThemeState(theme)
    themeStore.collapsedNavigation = collapsed_navigation
    themeStore.advancedRendering = advanced_rendering

    mixpanel_init('014c7d6a336d0efaefe3aca91063748d', { debug: dev, persistence: 'localStorage' })
    if (telemetry) {
      mixpanel_opt_out_tracking()
    }
    mixpanel_track('Launched', { version, dev, onboarded })

    if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

    if ((await type()) === 'Darwin') {
      document.getElementsByTagName('html')[0].classList.add('mac')
    } else {
      document.getElementsByTagName('html')[0].classList.add('windows')
    }

    await warning_listener((e) =>
      notificationsWrapper.value.addNotification({
        title: 'Warning',
        text: e.message,
        type: 'warn',
      }),
    )

    if (showOnboarding.value) {
      onboardingVideo.value.play()
    }

    isLoading.value = false
  },
  failure: async (e) => {
    isLoading.value = false
    failureText.value = e
    os.value = await getOS()
  },
})

onMounted(async () => {
  await showMainWindow()
})

const handleClose = async () => {
  await TauriWindow.getCurrent().close()
}

TauriWindow.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
  await handleClose()
})

const router = useRouter()
router.afterEach((to, from, failure) => {
  if (mixpanel_is_loaded()) {
    mixpanel_track('PageView', { path: to.path, fromPath: from.path, failed: failure })
  }
})
const route = useRoute()
const isOnBrowse = computed(() => route.path.startsWith('/browse'))

const loading = useLoading()

const notifications = useNotifications()
const notificationsWrapper = ref()

const error = useError()
const errorModal = ref()

const install = useInstall()
const modInstallModal = ref()
const installConfirmModal = ref()
const incompatibilityWarningModal = ref()

onMounted(() => {
  notifications.setNotifs(notificationsWrapper.value)

  error.setErrorModal(errorModal.value)

  install.setIncompatibilityWarningModal(incompatibilityWarningModal)
  install.setInstallConfirmModal(installConfirmModal)
  install.setModInstallModal(modInstallModal)
})

document.querySelector('body').addEventListener('click', function (e) {
  let target = e.target
  while (target != null) {
    if (target.matches('a')) {
      if (
        target.href &&
        ['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
        !target.classList.contains('router-link-active') &&
        !target.href.startsWith('http://localhost') &&
        !target.href.startsWith('https://tauri.localhost')
      ) {
        window.__TAURI_INVOKE__('tauri', {
          __tauriModule: 'Shell',
          message: {
            cmd: 'open',
            path: target.href,
          },
        })
      }
      e.preventDefault()
      break
    }
    target = target.parentElement
  }
})

document.querySelector('body').addEventListener('auxclick', function (e) {
  // disables middle click -> new tab
  if (e.button === 1) {
    e.preventDefault()
    // instead do a left click
    const event = new MouseEvent('click', {
      view: window,
      bubbles: true,
      cancelable: true,
    })
    e.target.dispatchEvent(event)
  }
})

const accounts = ref(null)

command_listener(async (e) => {
  if (e.event === 'RunMRPack') {
    // RunMRPack should directly install a local mrpack given a path
    if (e.path.endsWith('.mrpack')) {
      await install_from_file(e.path).catch(handleError)
      mixpanel_track('InstanceCreate', {
        source: 'CreationModalFileDrop',
      })
    }
  } else {
    // Other commands are URL-based (deep linking)
    urlModal.value.show(e)
  }
})
</script>

<template>
  <div v-if="failureText" class="failure dark-mode">
    <div class="appbar-failure dark-mode">
      <Button v-if="os != 'MacOS'" icon-only @click="TauriWindow.getCurrent().close()">
        <XIcon />
      </Button>
    </div>
    <div class="error-view dark-mode">
      <Card class="error-text">
        <div class="label">
          <h3>
            <span class="label__title size-card-header">Failed to initialize</span>
          </h3>
        </div>
        <div class="error-div">
          Modrinth App failed to load correctly. This may be because of a corrupted file, or because
          the app is missing crucial files.
        </div>
        <div class="error-div">You may be able to fix it one of the following ways:</div>
        <ul class="error-div">
          <li>Ennsuring you are connected to the internet, then try restarting the app.</li>
          <li>Redownloading the app.</li>
        </ul>
        <div class="error-div">
          If it still does not work, you can seek support using the link below. You should provide
          the following error, as well as any recent launcher logs in the folder below.
        </div>
        <div class="error-div">The following error was provided:</div>

        <Card class="error-message">
          {{ failureText.message }}
        </Card>

        <div class="button-row push-right">
          <Button @click="showLauncherLogsFolder"><FileIcon />Open launcher logs</Button>

          <a class="btn" href="https://support.modrinth.com"> <ChatIcon /> Get support </a>
        </div>
      </Card>
    </div>
  </div>
  <SplashScreen v-else-if="isLoading" app-loading />
  <OnboardingScreen v-else-if="showOnboarding" :finish="() => (showOnboarding = false)" />
  <div v-else class="container">
    <div class="nav-container">
      <div class="nav-section">
        <suspense>
          <AccountsCard ref="accounts" mode="small" />
        </suspense>
        <div class="pages-list">
          <RouterLink v-tooltip="'Home'" to="/" class="btn icon-only collapsed-button">
            <HomeIcon />
          </RouterLink>
          <RouterLink
            v-tooltip="'Browse'"
            to="/browse/modpack"
            class="btn icon-only collapsed-button"
            :class="{
              'router-link-active': isOnBrowse,
            }"
          >
            <SearchIcon />
          </RouterLink>
          <RouterLink v-tooltip="'Library'" to="/library" class="btn icon-only collapsed-button">
            <LibraryIcon />
          </RouterLink>
          <Suspense>
            <InstanceCreationModal ref="installationModal" />
          </Suspense>
        </div>
      </div>
      <div class="settings pages-list">
        <Button
          v-tooltip="'Create profile'"
          class="sleek-primary collapsed-button"
          icon-only
          :disabled="offline"
          @click="() => $refs.installationModal.show()"
        >
          <PlusIcon />
        </Button>
        <RouterLink v-tooltip="'Settings'" to="/settings" class="btn icon-only collapsed-button">
          <SettingsIcon />
        </RouterLink>
      </div>
    </div>
    <div class="view">
      <div class="appbar-row">
        <div data-tauri-drag-region class="appbar">
          <section class="navigation-controls">
            <Breadcrumbs data-tauri-drag-region />
          </section>
          <section class="mod-stats">
            <Suspense>
              <RunningAppBar />
            </Suspense>
          </section>
        </div>
        <section v-if="!nativeDecorations" class="window-controls">
          <Button class="titlebar-button" icon-only @click="() => appWindow.minimize()">
            <MinimizeIcon />
          </Button>
          <Button class="titlebar-button" icon-only @click="() => appWindow.toggleMaximize()">
            <MaximizeIcon />
          </Button>
          <Button
            class="titlebar-button close"
            icon-only
            @click="
              () => {
                saveWindowState(StateFlags.ALL)
                handleClose()
              }
            "
          >
            <XIcon />
          </Button>
        </section>
      </div>
      <div class="router-view">
        <ModrinthLoadingIndicator
          offset-height="var(--appbar-height)"
          offset-width="var(--sidebar-width)"
        />
        <RouterView v-slot="{ Component }">
          <template v-if="Component">
            <Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
              <component :is="Component"></component>
            </Suspense>
          </template>
        </RouterView>
      </div>
    </div>
  </div>
  <URLConfirmModal ref="urlModal" />
  <Notifications ref="notificationsWrapper" />
  <ErrorModal ref="errorModal" />
  <ModInstallModal ref="modInstallModal" />
  <IncompatibilityWarningModal ref="incompatibilityWarningModal" />
  <InstallConfirmModal ref="installConfirmModal" />
</template>

<style lang="scss" scoped>
.sleek-primary {
  background-color: var(--color-brand-highlight);
  transition: all ease-in-out 0.1s;
}

.navigation-controls {
  flex-grow: 1;
  width: min-content;
}

.appbar-row {
  display: flex;
  flex-direction: row;
}

.window-controls {
  z-index: 20;
  display: none;
  flex-direction: row;
  align-items: center;

  .titlebar-button {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all ease-in-out 0.1s;
    background-color: var(--color-raised-bg);
    color: var(--color-base);
    border-radius: 0;
    height: 3.25rem;

    &.close {
      &:hover,
      &:active {
        background-color: var(--color-red);
        color: var(--color-accent-contrast);
      }
    }

    &:hover,
    &:active {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
    }
  }
}

.container {
  --appbar-height: 3.25rem;
  --sidebar-width: 4.5rem;

  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;

  .view {
    width: calc(100% - var(--sidebar-width));
    background-color: var(--color-raised-bg);

    .appbar {
      display: flex;
      align-items: center;
      flex-grow: 1;
      background: var(--color-raised-bg);
      text-align: center;
      padding: var(--gap-md);
      height: 3.25rem;
      gap: var(--gap-sm);
      //no select
      user-select: none;
      -webkit-user-select: none;
    }

    .router-view {
      width: 100%;
      height: calc(100% - 3.125rem);
      overflow: auto;
      overflow-x: hidden;
      background-color: var(--color-bg);
      border-top-left-radius: var(--radius-xl);
    }
  }
}

.failure {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--color-bg);

  .appbar-failure {
    display: flex; /* Change to flex to align items horizontally */
    justify-content: flex-end; /* Align items to the right */
    height: 3.25rem;
    //no select
    user-select: none;
    -webkit-user-select: none;
  }

  .error-view {
    display: flex; /* Change to flex to align items horizontally */
    justify-content: center;
    width: 100%;
    background-color: var(--color-bg);

    color: var(--color-base);

    .card {
      background-color: var(--color-raised-bg);
    }

    .error-text {
      display: flex;
      max-width: 60%;
      gap: 0.25rem;
      flex-direction: column;

      .error-div {
        // spaced out
        margin: 0.5rem;
      }

      .error-message {
        margin: 0.5rem;
        background-color: var(--color-button-bg);
      }
    }
  }
}

.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
  padding: var(--gap-md);
}

.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
  gap: 0.5rem;

  a {
    display: flex;
    align-items: center;
    word-spacing: 3px;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-base);
    box-shadow: none;

    &.router-link-active {
      color: var(--color-contrast);
      background: var(--color-button-bg);
      box-shadow: var(--shadow-floating);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }
  }

  &.primary {
    color: var(--color-accent-contrast);
    background-color: var(--color-brand);
  }
}

.collapsed-button {
  height: 3rem !important;
  width: 3rem !important;
  padding: 0.75rem;
  border-radius: var(--radius-md);
  box-shadow: none;

  svg {
    width: 1.5rem !important;
    height: 1.5rem !important;
    max-width: 1.5rem !important;
    max-height: 1.5rem !important;
  }
}

.nav-section {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 100%;
  gap: 1rem;
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-md);

  .transparent {
    padding: var(--gap-sm) 0;
  }
}
</style>
