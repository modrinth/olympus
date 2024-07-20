import { createApp } from 'vue'
import router from '@/routes'
import App from '@/App.vue'
import { createPinia } from 'pinia'
import '@modrinth/assets/omorphia.scss'
import '@/assets/stylesheets/global.scss'
import FloatingVue from 'floating-vue'
import 'floating-vue/dist/style.css'
import { get_opening_command, initialize_state } from '@/helpers/state'
import { get } from '@/helpers/settings'
import { invoke } from '@tauri-apps/api'
import { isDev } from './helpers/utils.js'
import { createPlugin } from '@vintl/vintl/plugin'

const VIntlPlugin = createPlugin({
  controllerOpts: {
    defaultLocale: 'en-US',
    locale: 'en-US',
    locales: [
      {
        tag: 'en-US',
        meta: {
          displayName: 'American English',
        },
      },
    ],
  },
  globalMixin: true,
  injectInto: [],
})

const pinia = createPinia()

let app = createApp(App)
app.use(router)
app.use(pinia)
app.use(FloatingVue)
app.use(VIntlPlugin)

const mountedApp = app.mount('#app')

const raw_invoke = async (plugin, fn, args) => {
  if (plugin === '') {
    await invoke(fn, args)
  } else {
    await invoke('plugin:' + plugin + '|' + fn, args)
  }
}
isDev()
  .then((dev) => {
    if (dev) {
      window.raw_invoke = raw_invoke
    }
  })
  .catch((err) => {
    console.error(err)
  })

initialize_state()
  .then(() => {
    // First, redirect to other landing page if we have that setting
    get()
      .then((fetchSettings) => {
        if (fetchSettings?.default_page && fetchSettings?.default_page !== 'Home') {
          router.push({ name: fetchSettings?.default_page })
        }
      })
      .catch((err) => {
        console.error(err)
      })
      .finally(() => {
        mountedApp.initialize()
        get_opening_command().then((command) => {
          console.log(JSON.stringify(command)) // change me to use whatever FE command handler is made
        })
      })
  })
  .catch((err) => {
    console.error('Failed to initialize app', err)
    mountedApp.failure(err)
  })
