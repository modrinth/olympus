<script setup>
import { onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { DownloadIcon, StopCircleIcon } from '@modrinth/assets'
import { Card, Avatar, AnimatedLogo } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import { install as pack_install } from '@/helpers/pack'
import { list, run } from '@/helpers/profile'
import {
  get_all_running_profile_paths,
  get_uuids_by_profile_path,
  kill_by_uuid,
} from '@/helpers/process'
import { process_listener } from '@/helpers/events'
import { useFetch } from '@/helpers/fetch.js'
import { handleError } from '@/store/state.js'
import { showProfileInFolder } from '@/helpers/utils.js'
import ModInstallModal from '@/components/ui/ModInstallModal.vue'
import { mixpanel_track } from '@/helpers/mixpanel'
import { useInstanceIcon } from '@/composable/instance/icon.js'
import { handleSevereError } from '@/store/error.js'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const confirmModal = ref(null)
const modInstallModal = ref(null)
const playing = ref(false)

const uuid = ref(null)
const modLoading = ref(
  props.instance.install_stage ? props.instance.install_stage !== 'installed' : false,
)

watch(
  () => props.instance,
  () => {
    modLoading.value = props.instance.install_stage
      ? props.instance.install_stage !== 'installed'
      : false
  },
)

const router = useRouter()

const seeInstance = async () => {
  const instancePath = props.instance.metadata
    ? `/instance/${encodeURIComponent(props.instance.path)}/`
    : `/project/${encodeURIComponent(props.instance.project_id)}/`

  await router.push(instancePath)
}

const checkProcess = async () => {
  const runningPaths = await get_all_running_profile_paths().catch(handleError)

  if (runningPaths.includes(props.instance.path)) {
    playing.value = true
    return
  }

  playing.value = false
  uuid.value = null
}

const install = async (e) => {
  e?.stopPropagation()
  modLoading.value = true
  const versions = await useFetch(
    `https://api.modrinth.com/v2/project/${props.instance.project_id}/version`,
    'project versions',
  )

  if (props.instance.project_type === 'modpack') {
    const packs = Object.values(await list(true).catch(handleError))

    if (
      packs.length === 0 ||
      !packs
        .map((value) => value.metadata)
        .find((pack) => pack.linked_data?.project_id === props.instance.project_id)
    ) {
      modLoading.value = true
      await pack_install(
        props.instance.project_id,
        versions[0].id,
        props.instance.title,
        props.instance.icon_url,
      ).catch(handleError)
      modLoading.value = false

      mixpanel_track('PackInstall', {
        id: props.instance.project_id,
        version_id: versions[0].id,
        title: props.instance.title,
        source: 'InstanceCard',
      })
    } else
      confirmModal.value.show(
        props.instance.project_id,
        versions[0].id,
        props.instance.title,
        props.instance.icon_url,
      )
  } else {
    modInstallModal.value.show(
      props.instance.project_id,
      versions,
      props.instance.title,
      props.instance.project_type,
    )
  }

  modLoading.value = false
}

const play = async (e, context) => {
  e?.stopPropagation()
  modLoading.value = true
  uuid.value = await run(props.instance.path).catch(handleSevereError)
  modLoading.value = false
  playing.value = true

  mixpanel_track('InstancePlay', {
    loader: props.instance.metadata.loader,
    game_version: props.instance.metadata.game_version,
    source: context,
  })
}

const stop = async (e, context) => {
  e?.stopPropagation()
  playing.value = false

  // If we lost the uuid for some reason, such as a user navigating
  // from-then-back to this page, we will get all uuids by the instance path.
  // For-each uuid, kill the process.
  if (!uuid.value) {
    const uuids = await get_uuids_by_profile_path(props.instance.path).catch(handleError)
    uuid.value = uuids[0]
    uuids.forEach(async (u) => await kill_by_uuid(u).catch(handleError))
  } else await kill_by_uuid(uuid.value).catch(handleError) // If we still have the uuid, just kill it

  mixpanel_track('InstanceStop', {
    loader: props.instance.metadata.loader,
    game_version: props.instance.metadata.game_version,
    source: context,
  })

  uuid.value = null
}

const openFolder = async () => {
  await showProfileInFolder(props.instance.path)
}

const addContent = async () => {
  await router.push({
    path: `/browse/${props.instance.metadata.loader === 'vanilla' ? 'datapack' : 'mod'}`,
    query: { i: props.instance.path },
  })
}

const avatarSrc = useInstanceIcon(props.instance)

defineExpose({
  install,
  playing,
  play,
  stop,
  seeInstance,
  openFolder,
  addContent,
  instance: props.instance,
})

const unlisten = await process_listener((e) => {
  if (e.event === 'finished' && e.uuid === uuid.value) playing.value = false
})

onUnmounted(() => unlisten())
</script>

<template>
  <div class="instance">
    <Card class="instance-card-item button-base" @click="seeInstance" @mouseenter="checkProcess">
      <Avatar size="lg" :src="avatarSrc" alt="Mod card" class="mod-image" />
      <div class="project-info">
        <p class="title">{{ props.instance.metadata?.name || props.instance.title }}</p>
        <p class="description">
          {{ props.instance.metadata?.loader }}
          {{ props.instance.metadata?.game_version || props.instance.latest_version }}
        </p>
      </div>
    </Card>
    <div
      v-if="props.instance.metadata && playing === false && modLoading === false"
      class="install cta button-base"
      @click="(e) => play(e, 'InstanceCard')"
    >
      <PlayIcon />
    </div>
    <div v-else-if="modLoading === true && playing === false" class="cta loading-cta">
      <AnimatedLogo class="loading-indicator" />
    </div>
    <div
      v-else-if="playing === true"
      class="stop cta button-base"
      @click="(e) => stop(e, 'InstanceCard')"
      @mousehover="checkProcess"
    >
      <StopCircleIcon />
    </div>
    <div v-else class="install cta button-base" @click="install">
      <DownloadIcon />
    </div>
    <InstallConfirmModal ref="confirmModal" />
    <ModInstallModal ref="modInstallModal" />
  </div>
</template>

<style lang="scss">
.loading-indicator {
  width: 2.5rem !important;
  height: 2.5rem !important;

  svg {
    width: 2.5rem !important;
    height: 2.5rem !important;
  }
}
</style>

<style lang="scss" scoped>
.instance {
  position: relative;

  &:hover {
    .cta {
      opacity: 1;
      bottom: calc(var(--gap-md) + 4.25rem);
    }
  }
}

.cta {
  position: absolute;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  z-index: 1;
  width: 3rem;
  height: 3rem;
  right: calc(var(--gap-md) * 2);
  bottom: 3.25rem;
  opacity: 0;
  transition:
    0.2s ease-in-out bottom,
    0.2s ease-in-out opacity,
    0.1s ease-in-out filter !important;
  cursor: pointer;
  box-shadow: var(--shadow-floating);

  svg {
    color: var(--color-accent-contrast);
    width: 1.5rem !important;
    height: 1.5rem !important;
  }

  &.install {
    background: var(--color-brand);
    display: flex;
  }

  &.stop {
    background: var(--color-red);
    display: flex;
  }

  &.loading-cta {
    background: hsl(220, 11%, 10%) !important;
    display: flex;
    justify-content: center;
    align-items: center;
  }
}

.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .mod-image {
    --size: 100%;

    width: 100% !important;
    height: auto !important;
    max-width: unset !important;
    max-height: unset !important;
    aspect-ratio: 1 / 1 !important;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      width: 100%;
      margin: 0;
      font-weight: 600;
      font-size: 1rem;
      line-height: 110%;
      display: inline-block;
    }

    .description {
      color: var(--color-base);
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      font-weight: 500;
      font-size: 0.775rem;
      line-height: 125%;
      margin: 0.25rem 0 0;
      text-transform: capitalize;
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
}
</style>
