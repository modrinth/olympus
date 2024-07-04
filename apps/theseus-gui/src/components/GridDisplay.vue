<script setup>
import Instance from '@/components/ui/Instance.vue'
import { computed, ref } from 'vue'
import {
  ClipboardCopyIcon,
  FolderOpenIcon,
  PlayIcon,
  PlusIcon,
  TrashIcon,
  StopCircleIcon,
  EyeIcon,
  Card,
  DropdownSelect,
  SearchIcon,
  XIcon,
  Button,
  formatCategoryHeader,
  ConfirmModal,
} from '@modrinth/omorphia'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import dayjs from 'dayjs'
import { useTheming } from '@/store/theme.js'
import { duplicate, remove } from '@/helpers/profile.js'
import { handleError } from '@/store/notifications.js'

const props = defineProps({
  instances: {
    type: Array,
    default() {
      return []
    },
  },
  label: {
    type: String,
    default: '',
  },
})
const instanceOptions = ref(null)
const instanceComponents = ref(null)

const themeStore = useTheming()
const currentDeleteInstance = ref(null)
const confirmModal = ref(null)

async function deleteProfile() {
  if (currentDeleteInstance.value) {
    instanceComponents.value = instanceComponents.value.filter(
      (x) => x.instance.path !== currentDeleteInstance.value,
    )
    await remove(currentDeleteInstance.value).catch(handleError)
  }
}

async function duplicateProfile(p) {
  await duplicate(p).catch(handleError)
}

const handleRightClick = (event, profilePathId) => {
  const item = instanceComponents.value.find((x) => x.instance.path === profilePathId)
  const baseOptions = [
    { name: 'add_content' },
    { type: 'divider' },
    { name: 'edit' },
    { name: 'duplicate' },
    { name: 'open' },
    { name: 'copy' },
    { type: 'divider' },
    {
      name: 'delete',
      color: 'danger',
    },
  ]

  instanceOptions.value.showMenu(
    event,
    item,
    item.playing
      ? [
          {
            name: 'stop',
            color: 'danger',
          },
          ...baseOptions,
        ]
      : [
          {
            name: 'play',
            color: 'primary',
          },
          ...baseOptions,
        ],
  )
}

const handleOptionsClick = async (args) => {
  switch (args.option) {
    case 'play':
      args.item.play(null, 'InstanceGridContextMenu')
      break
    case 'stop':
      args.item.stop(null, 'InstanceGridContextMenu')
      break
    case 'add_content':
      await args.item.addContent()
      break
    case 'edit':
      await args.item.seeInstance()
      break
    case 'duplicate':
      if (args.item.instance.install_stage == 'installed')
        await duplicateProfile(args.item.instance.path)
      break
    case 'open':
      await args.item.openFolder()
      break
    case 'copy':
      await navigator.clipboard.writeText(args.item.instance.path)
      break
    case 'delete':
      currentDeleteInstance.value = args.item.instance.path
      confirmModal.value.show()
      break
  }
}

const search = ref('')
const group = ref('Category')
const filters = ref('All profiles')
const sortBy = ref('Name')

const filteredResults = computed(() => {
  let instances = props.instances.filter((instance) => {
    return instance.metadata.name.toLowerCase().includes(search.value.toLowerCase())
  })

  if (sortBy.value === 'Name') {
    instances.sort((a, b) => {
      return a.metadata.name.localeCompare(b.metadata.name)
    })
  }

  if (sortBy.value === 'Game version') {
    instances.sort((a, b) => {
      return a.metadata.game_version.localeCompare(
        b.metadata.game_version,
        undefined,
        { numeric: true }
      )
    })
  }

  if (sortBy.value === 'Last played') {
    instances.sort((a, b) => {
      return dayjs(b.metadata.last_played ?? 0).diff(dayjs(a.metadata.last_played ?? 0))
    })
  }

  if (sortBy.value === 'Date created') {
    instances.sort((a, b) => {
      return dayjs(b.metadata.date_created).diff(dayjs(a.metadata.date_created))
    })
  }

  if (sortBy.value === 'Date modified') {
    instances.sort((a, b) => {
      return dayjs(b.metadata.date_modified).diff(dayjs(a.metadata.date_modified))
    })
  }

  if (filters.value === 'Custom instances') {
    instances = instances.filter((instance) => {
      return !instance.metadata?.linked_data
    })
  } else if (filters.value === 'Downloaded modpacks') {
    instances = instances.filter((instance) => {
      return instance.metadata?.linked_data
    })
  }

  const instanceMap = new Map()

  if (group.value === 'Loader') {
    instances.forEach((instance) => {
      const loader = formatCategoryHeader(instance.metadata.loader)
      if (!instanceMap.has(loader)) {
        instanceMap.set(loader, [])
      }

      instanceMap.get(loader).push(instance)
    })
  } else if (group.value === 'Game version') {
    instances.forEach((instance) => {
      if (!instanceMap.has(instance.metadata.game_version)) {
        instanceMap.set(instance.metadata.game_version, [])
      }

      instanceMap.get(instance.metadata.game_version).push(instance)
    })
  } else if (group.value === 'Category') {
    instances.forEach((instance) => {
      if (instance.metadata.groups.length === 0) {
        instance.metadata.groups.push('None')
      }

      for (const category of instance.metadata.groups) {
        if (!instanceMap.has(category)) {
          instanceMap.set(category, [])
        }

        instanceMap.get(category).push(instance)
      }
    })
  } else {
    return instanceMap.set('None', instances)
  }

  // For 'name', we intuitively expect the sorting to apply to the name of the group first, not just the name of the instance
  // ie: Category A should come before B, even if the first instance in B comes before the first instance in A
  if (sortBy.value === 'Name') {
    const sortedEntries = [...instanceMap.entries()].sort((a, b) => {
      // None should always be first
      if (a[0] === 'None' && b[0] !== 'None') {
        return -1
      }
      if (a[0] !== 'None' && b[0] === 'None') {
        return 1
      }
      return a[0].localeCompare(b[0])
    })
    instanceMap.clear()
    sortedEntries.forEach((entry) => {
      instanceMap.set(entry[0], entry[1])
    })
  }
  // default sorting would do 1.20.4 < 1.8.9 because 2 < 8
  // localeCompare with numeric=true puts 1.8.9 < 1.20.4 because 8 < 20
  if (group.value === 'Game version') {
    const sortedEntries = [...instanceMap.entries()].sort((a, b) => {
      return a[0].localeCompare(b[0], undefined, { numeric: true })
    })
    instanceMap.clear()
    sortedEntries.forEach((entry) => {
      instanceMap.set(entry[0], entry[1])
    })

  }

  return instanceMap
})
</script>
<template>
  <ConfirmModal
    ref="confirmModal"
    title="Are you sure you want to delete this instance?"
    description="If you proceed, all data for your instance will be removed. You will not be able to recover it."
    :has-to-type="false"
    proceed-label="Delete"
    :noblur="!themeStore.advancedRendering"
    @proceed="deleteProfile"
  />
  <Card class="header">
    <div class="iconified-input">
      <SearchIcon />
      <input v-model="search" type="text" placeholder="Search" class="search-input" />
      <Button @click="() => (search = '')">
        <XIcon />
      </Button>
    </div>
    <div class="labeled_button">
      <span>Sort by</span>
      <DropdownSelect
        v-model="sortBy"
        class="sort-dropdown"
        name="Sort Dropdown"
        :options="['Name', 'Last played', 'Date created', 'Date modified', 'Game version']"
        placeholder="Select..."
      />
    </div>
    <div class="labeled_button">
      <span>Filter by</span>
      <DropdownSelect
        v-model="filters"
        class="filter-dropdown"
        name="Filter Dropdown"
        :options="['All profiles', 'Custom instances', 'Downloaded modpacks']"
        placeholder="Select..."
      />
    </div>
    <div class="labeled_button">
      <span>Group by</span>
      <DropdownSelect
        v-model="group"
        class="group-dropdown"
        name="Group Dropdown"
        :options="['Category', 'Loader', 'Game version', 'None']"
        placeholder="Select..."
      />
    </div>
  </Card>
  <div
    v-for="instanceSection in Array.from(filteredResults, ([key, value]) => ({
      key,
      value,
    }))"
    :key="instanceSection.key"
    class="row"
  >
    <div v-if="instanceSection.key !== 'None'" class="divider">
      <p>{{ instanceSection.key }}</p>
      <hr aria-hidden="true" />
    </div>
    <section class="instances">
      <Instance
        v-for="instance in instanceSection.value"
        ref="instanceComponents"
        :key="instance.path + instance.install_stage"
        :instance="instance"
        @contextmenu.prevent.stop="(event) => handleRightClick(event, instance.path)"
      />
    </section>
  </div>
  <ContextMenu ref="instanceOptions" @option-clicked="handleOptionsClick">
    <template #play> <PlayIcon /> Play </template>
    <template #stop> <StopCircleIcon /> Stop </template>
    <template #add_content> <PlusIcon /> Add content </template>
    <template #edit> <EyeIcon /> View instance </template>
    <template #duplicate> <ClipboardCopyIcon /> Duplicate instance</template>
    <template #delete> <TrashIcon /> Delete </template>
    <template #open> <FolderOpenIcon /> Open folder </template>
    <template #copy> <ClipboardCopyIcon /> Copy path </template>
  </ContextMenu>
</template>
<style lang="scss" scoped>
.row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  width: 100%;
  padding: 1rem;

  .divider {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    gap: 1rem;
    margin-bottom: 1rem;

    p {
      margin: 0;
      font-size: 1rem;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    hr {
      background-color: var(--color-gray);
      height: 1px;
      width: 100%;
      border: none;
    }
  }
}

.header {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  align-items: inherit;
  margin: 1rem 1rem 0 !important;
  padding: 1rem;
  width: calc(100% - 2rem);

  .iconified-input {
    flex-grow: 1;

    input {
      min-width: 100%;
    }
  }

  .sort-dropdown {
    width: 10rem;
  }

  .filter-dropdown {
    width: 15rem;
  }

  .group-dropdown {
    width: 10rem;
  }

  .labeled_button {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
  }
}

.instances {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
  width: 100%;
  gap: 1rem;
  margin-right: auto;
  scroll-behavior: smooth;
  overflow-y: auto;
}
</style>
