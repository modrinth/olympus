<template>
  <Card class="header">
    <div class="iconified-input">
      <SearchIcon />
      <input v-model="search" type="text" placeholder="Search" class="search-input" />
      <Button class="r-btn" @click="() => (search = '')">
        <XIcon />
      </Button>
    </div>
    <div class="labeled_button">
      <span>Sort by</span>
      <DropdownSelect
        v-model="Filters.sort"
        class="sort-dropdown"
        name="Sort Dropdown"
        :options="['Name', 'Custom', 'Date created', 'Date modified']"
        placeholder="Select..."
      />
    </div>
    <div class="labeled_button">
      <span>Filter by</span>
      <DropdownSelect
        v-model="Filters.filter"
        class="filter-dropdown"
        name="Filter Dropdown"
        :options="['Current user', 'All users']"
        placeholder="Select..."
      />
    </div>
  </Card>
  <div class="content">
    <div class="instance">
      <Card class="instance-card-item">
        <div class="overlap">
          <canvas id="skin_container" class="render" />
          <AnimatedLogo v-if="!loaded_skins" />
        </div>
        <div class="card-row">
          <div class="project-info">
            <p class="title">Current Skin</p>
            <p class="description">{{ skinData.arms }}, {{ skinData.cape }}</p>
          </div>
          <Button v-if="!InLibrary" color="primary" @click="handleAdd"> Add to Library </Button>
        </div>
      </Card>
    </div>

    <div class="row">
      <div class="instance">
        <Card class="instance-card-item button-base" @click="handleModal">
          <PlusIcon size="lg" alt="Mod card" class="mod-image" />
          <div class="project-info">
            <p class="title">Add new skin</p>
            <p class="description">&nbsp;</p>
          </div>
        </Card>
      </div>
      <SkinSave
        v-for="skin in filteredResults"
        :key="skin"
        :data="skin"
        @contextmenu.prevent.stop="(event) => handleRightClick(event, skin)"
        @set-skin="(data) => clickCard(data)"
      />
      <ContextMenu ref="skinOptions" @option-clicked="handleOptionsClick">
        <template #use> <PlayIcon /> Use </template>
        <template v-if="Filters.sort === 'Custom'" #left> <ChevronLeftIcon /> Move Left </template>
        <template v-if="Filters.sort === 'Custom'" #right>
          <ChevronRightIcon /> Move Right
        </template>
        <template #edit> <EyeIcon /> Edit </template>
        <template #duplicate> <ClipboardCopyIcon /> Duplicate </template>
        <template #delete> <TrashIcon /> Delete </template>
      </ContextMenu>
    </div>
  </div>

  <Modal ref="skinModal" class="modal" header="Change skin" :noblur="!themeStore.advancedRendering">
    <div v-if="!editSkin" class="modal-header">
      <Chips
        v-model="changeSkinType"
        :items="['from file', 'import from launcher']"
        @click="handleModalType"
      />
    </div>
    <hr v-if="!editSkin" class="card-divider" />
    <div v-if="changeSkinType == 'from file'" class="modal-column">
      <div class="modal-row">
        <div class="modal-colum">
          <div class="image-upload">
            <Avatar :src="displaySkin" size="lg" />
          </div>
          <div class="image-input">
            <Button @click="openskin">
              <FolderOpenIcon />
              Select skin
            </Button>
          </div>
          <div class="input-row">
            <p class="input-label">Name</p>
            <input
              v-model="selectedSkin.name"
              autocomplete="off"
              class="text-input"
              type="text"
              maxlength="30"
              size="15"
            />
          </div>
          <div class="input-row">
            <p class="input-label">Arm style</p>
            <Chips v-model="selectedSkin.arms" :items="['classic', 'slim']" @click="handleArms" />
          </div>
        </div>
        <canvas id="new_render" class="render" width="0" height="0" />
      </div>
      <div class="input-row">
        <p class="input-label">Cape</p>
        <Chips v-model="selectedSkin.cape" :items="skinData.unlocked_capes" @click="handleCape" />
      </div>
      <div v-if="!editSkin" class="input-group push-right">
        <Button @click="skinModal.hide()">
          <XIcon />
          Cancel
        </Button>
        <Button :disabled="!validSkin || uploadingSkin" @click="handleSkin('upload')">
          <UploadIcon />
          {{ uploadingSkin ? 'Uploading...' : 'Use' }}
        </Button>
        <Button
          :disabled="!validSkin || uploadingSkin || selectedSkin.name.trim() == ''"
          @click="handleSkin('save')"
        >
          <SaveIcon />
          Save
        </Button>
        <Button
          color="primary"
          :disabled="!validSkin || uploadingSkin || selectedSkin.name.trim() == ''"
          @click="handleSkin('saveupload')"
        >
          <PlusIcon v-if="!uploadingSkin" />
          {{ uploadingSkin ? 'Uploading...' : 'Save & Use' }}
        </Button>
      </div>
      <div v-else class="input-group push-right">
        <Button @click="skinModal.hide()">
          <XIcon />
          Cancel
        </Button>
        <Button
          color="primary"
          :disabled="!validSkin || selectedSkin.name.trim() == ''"
          @click="edit_skin_end"
        >
          <SaveIcon />
          Update
        </Button>
      </div>
    </div>
    <div v-else class="modal-body">
      <Chips v-model="importType" :items="['Mojang', 'Curseforge']" @click="handleImportType" />
      <div class="path-selection">
        <h3>{{ importer.display }} path</h3>
        <div class="path-input">
          <div class="iconified-input">
            <FolderOpenIcon />
            <input v-model="importer.path" type="text" placeholder="Path to launcher" />
            <Button @click="() => (importer.path = '')">
              <XIcon />
            </Button>
          </div>
          <Button icon-only @click="selectLauncherPath">
            <FolderSearchIcon />
          </Button>
          <Button icon-only @click="reload">
            <UpdatedIcon />
          </Button>
        </div>
      </div>
      <div class="table">
        <div class="table-head table-row">
          <div class="toggle-all table-cell">
            <Checkbox
              class="select-checkbox"
              :model-value="importer.skinNames.every((child) => child.selected)"
              @update:model-value="
                (newValue) => importer.skinNames.forEach((child) => (child.selected = newValue))
              "
            />
          </div>
          <div class="name-cell table-cell">All skins</div>
        </div>
        <div v-if="importer.skinNames.length > 0" class="table-content">
          <div v-for="skin in importer.skinNames" :key="skin" class="table-row">
            <div class="checkbox-cell table-cell">
              <Checkbox v-model="skin.selected" class="select-checkbox" />
            </div>
            <div class="name-cell table-cell">
              {{ skin.name }}
            </div>
          </div>
        </div>
        <div v-else class="table-content empty">No skins found</div>
      </div>
      <div class="button-row">
        <Button
          :disabled="
            loading ||
            !Array.from(importer.skinNames)
              .flatMap((e) => e)
              .some((e) => e.selected)
          "
          color="primary"
          @click="next"
        >
          {{
            loading
              ? 'Importing...'
              : Array.from(importer.skinNames)
                    .flatMap((e) => e)
                    .some((e) => e.selected)
                ? `Import ${
                    Array.from(importer.skinNames)
                      .flatMap((e) => e)
                      .filter((e) => e.selected).length
                  } skins`
                : 'Select skins to import'
          }}
        </Button>
        <ProgressBar v-if="loading" :progress="(importedSkins / (totalSkins + 0.0001)) * 100" />
      </div>
    </div>
  </Modal>
  <ConfirmModal
    ref="deleteConfirmModal"
    title="Are you sure you want to delete this skin?"
    description="If you proceed, the skin will be removed for all users. You will not be able to recover it."
    :has-to-type="false"
    proceed-label="Delete"
    :noblur="!themeStore.advancedRendering"
    @proceed="deleteSkin"
  />
  <Notifications ref="notificationsWrapper" />
</template>

<script setup>
import {
  Avatar,
  Notifications,
  Card,
  Button,
  Modal,
  ConfirmModal,
  Chips,
  Checkbox,
  DropdownSelect,
  AnimatedLogo,
} from '@modrinth/ui'
import {
  PlusIcon,
  SaveIcon,
  SearchIcon,
  UploadIcon,
  UpdatedIcon,
  PlayIcon,
  FolderOpenIcon,
  FolderSearchIcon,
  ClipboardCopyIcon,
  XIcon,
  EyeIcon,
  TrashIcon,
  ChevronRightIcon,
  ChevronLeftIcon,
} from '@modrinth/assets'
import { ref, onMounted, watch, computed, onBeforeUnmount } from 'vue'
import ProgressBar from '@/components/ui/ProgressBar.vue'
import { handleError, useTheming } from '@/store/state.js'
import { useNotifications } from '@/store/notifications.js'
import { open } from '@tauri-apps/api/dialog'
import { tauri } from '@tauri-apps/api'
import dayjs from 'dayjs'
import SkinSave from '@/components/ui/SkinSave.vue'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { get_default_launcher_path } from '@/helpers/import.js'
import {
  check_image,
  check_skin,
  loaded_skins,
  get_user_skin_data,
  get_launcher_names,
  selectedAccount,
  set_skin,
  save_skin,
  import_skin,
  delete_skin,
  get_order,
  save_order,
  get_skins,
  get_render,
  get_cape_data,
  get_heads,
  set_cape,
  save_filters,
  Filters,
} from '@/helpers/skin_manager.js'
import { IdleAnimation, SkinViewer, WalkingAnimation } from 'skinview3d'

const themeStore = useTheming()
const notificationsWrapper = ref(null)

const skinModal = ref(null)
const deleteConfirmModal = ref(null)
const skinOptions = ref(null)
// Possibly dont need skinComponents?
const changeSkinType = ref('from file')

const skinData = ref({})
const selectedSkin = ref({
  arms: 'classic',
  cape: 'no cape',
  name: '',
  skin: '',
  unlocked_capes: [],
})

const displaySkin = ref(null)
const currentRender = ref(null)
const modalRender = ref(null)

const InLibrary = ref(true)
const validSkin = ref(false)
const uploadingSkin = ref(false)
const editSkin = ref(false)

const loading = ref(false)
const importedSkins = ref(0)
const totalSkins = ref(0)
const importer = ref({
  skinNames: [],
  path: '',
  display: '',
})
const importType = ref('Mojang')

const skinSaves = ref(await get_skins().catch(handleError))
const skinOrder = ref([])

const search = ref('')

watch(notificationsWrapper, () => {
  useNotifications().setNotifs(notificationsWrapper.value)
})

const filteredResults = computed(() => {
  let saves = skinSaves.value.filter((save) => {
    return save.name.toLowerCase().includes(search.value.toLowerCase())
  })

  if (Filters.value.filter === 'Current user') {
    saves = saves.filter((save) => {
      return save.user === selectedAccount.value.id
    })
  }

  if (Filters.value.sort === 'Name') {
    saves.sort((a, b) => {
      return a.name.localeCompare(b.name)
    })
  }

  if (Filters.value.sort === 'Custom') {
    saves.sort((a, b) => {
      return skinOrder.value.indexOf(a.id) - skinOrder.value.indexOf(b.id)
    })
  }

  if (Filters.value.sort === 'Date created') {
    saves.sort((a, b) => {
      return dayjs(b.created).diff(dayjs(a.created))
    })
  }

  if (Filters.value.sort === 'Date modified') {
    saves.sort((a, b) => {
      return dayjs(b.updated).diff(dayjs(a.updated))
    })
  }
  return saves
})

const moveCard = async (move, id) => {
  let order = Array.from(skinOrder.value)
  if (Filters.value.filter === 'Current user') {
    let saves = Array.from(skinSaves.value).sort((a, b) => {
      return skinOrder.value.indexOf(a.id) - skinOrder.value.indexOf(b.id)
    })
    order = order.filter((_, i) => {
      return saves[i].user === selectedAccount.value.id
    })
  }
  let index = order.indexOf(id)
  if (index == 0 && move == -1) return
  if (index == order.length - 1 && move == 1) return

  let targetIndex = skinOrder.value.indexOf(order[index + move])
  let currentIndex = skinOrder.value.indexOf(id)
  skinOrder.value.splice(currentIndex, 1)
  skinOrder.value.splice(targetIndex, 0, id)
  await save_order(skinOrder.value, selectedAccount.value.id)
}

const handleRightClick = (event, item) => {
  let baseOptions = [
    {
      name: 'use',
      color: 'primary',
    },
    { type: 'divider' },
    { name: 'edit' },
    { name: 'duplicate' },
    { type: 'divider' },
    {
      name: 'delete',
      color: 'danger',
    },
  ]

  if (Filters.value.sort === 'Custom') {
    baseOptions = [
      {
        name: 'use',
        color: 'primary',
      },
      { type: 'divider' },
      { name: 'edit' },
      { name: 'duplicate' },
      { type: 'divider' },
      { name: 'left' },
      { name: 'right' },
      { type: 'divider' },
      {
        name: 'delete',
        color: 'danger',
      },
    ]
  }

  skinOptions.value.showMenu(event, item, baseOptions)
}

const handleOptionsClick = async (args) => {
  switch (args.option) {
    case 'use':
      selectedSkin.value = {
        arms: args.item.arms,
        cape: args.item.cape,
        skin: args.item.skin,
      }
      await handleSkin('upload')
      break
    case 'left':
      if (Filters.value.sort === 'Custom') await moveCard(-1, args.item.id)
      break
    case 'right':
      if (Filters.value.sort === 'Custom') await moveCard(1, args.item.id)
      break
    case 'edit':
      await edit_skin(args.item)
      break
    case 'duplicate':
      await duplicate_skin(args.item)
      break
    case 'delete':
      selectedSkin.value = args.item
      deleteConfirmModal.value.show()
      break
  }
}

const deleteSkin = async () => {
  skinSaves.value.splice(skinSaves.value.indexOf(selectedSkin.value), 1)
  await delete_skin(selectedSkin.value.id).catch(handleError)
  InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError,
  )
}

const selectLauncherPath = async () => {
  importer.value.path = await open({ multiple: false, directory: true })

  if (importer.value.path) {
    await reload()
  }
}

const reload = async () => {
  importer.value.skinNames = get_launcher_names(importer.value.path, importType.value).catch(
    handleError,
  )
}

const next = async () => {
  importedSkins.value = 0
  totalSkins.value = Array.from(importer.value.skinNames)
    .flatMap((e) => e)
    .filter((e) => e.selected).length
  loading.value = true
  for (const skin of importer.value.skinNames.filter((skin) => skin.selected)) {
    const data = await import_skin(skin.id, importer.value.path, importType.value).catch(
      handleError,
    )
    const model = await get_render(data).catch(handleError)
    await save_skin(selectedAccount.value.id, data, skin.name, model, '').catch(handleError)
    skin.selected = false
    importedSkins.value++
  }
  skinSaves.value = await get_skins().catch(handleError)
  skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
  loading.value = false
  skinModal.value.hide()
}

const handleModal = async () => {
  changeSkinType.value = 'from file'
  editSkin.value = false
  validSkin.value = true
  displaySkin.value = skinData.value.skin
  selectedSkin.value.skin = skinData.value.skin
  await skinModal.value.show()
  selectedSkin.value.arms = skinData.value.arms
  selectedSkin.value.cape = skinData.value.cape
  await create_modal_render()
}

const handleAdd = async () => {
  const model = await get_render(skinData.value).catch(handleError)
  await save_skin(selectedAccount.value.id, skinData.value, 'untitled', model, '').catch(
    handleError,
  )
  skinSaves.value = await get_skins().catch(handleError)
  skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
  InLibrary.value = true
}

const skinClear = async () => {
  validSkin.value = false
  displaySkin.value = null
  selectedSkin.value = {
    arms: 'classic',
    cape: 'no cape',
    name: '',
    skin: '',
    unlocked_capes: [],
  }
  if (modalRender.value) {
    modalRender.value.resetSkin()
    modalRender.value.resetCape()
  }
}

const handleArms = async () => {
  if (validSkin.value) {
    modalRender.value.loadSkin(displaySkin.value, { model: convert_arms(selectedSkin.value.arms) })
  }
}

const handleCape = async () => {
  if (validSkin.value) {
    if (selectedSkin.value.cape == 'no cape') modalRender.value.resetCape()
    else
      modalRender.value.loadCape(
        await get_cape_data(selectedSkin.value.cape, 'url').catch(handleError),
      )
  }
}

const clickCard = (data) => {
  selectedSkin.value = data
  handleSkin('upload')
}

const handleSkin = async (state) => {
  if (state.includes('save')) {
    const model = await get_render(selectedSkin.value).catch(handleError)
    await save_skin(
      selectedAccount.value.id,
      selectedSkin.value,
      selectedSkin.value.name.trim(),
      model,
      '',
    ).catch(handleError)
    skinSaves.value = await get_skins().catch(handleError)
    skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
  }
  if (state.includes('upload')) {
    uploadingSkin.value = true
    const capeid = await get_cape_data(selectedSkin.value.cape, 'id').catch(handleError)
    const uploadedCape = await set_cape(capeid, selectedAccount.value.access_token).catch(
      handleError,
    )
    const uploadedSkin = await set_skin(
      selectedSkin.value.skin,
      selectedSkin.value.arms,
      selectedAccount.value,
    ).catch(handleError)

    if (uploadedSkin) {
      if (!selectedSkin.value.skin.startsWith('data:image/png;base64,')) {
        skinData.value.skin = tauri.convertFileSrc(selectedSkin.value.skin)
      } else {
        skinData.value.skin = selectedSkin.value.skin
      }
      skinData.value.arms = selectedSkin.value.arms
      skinData.value.cape = selectedSkin.value.cape
      InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
        handleError,
      )
      const renderArms = convert_arms(selectedSkin.value.arms)
      currentRender.value.loadSkin(skinData.value.skin, { model: renderArms })
    } else {
      notificationsWrapper.value.addNotification({
        title: 'Error Uploading Skin',
        text: 'Improper response from Mojang API. Please try again soon',
        type: 'error',
      })
    }
    if (uploadedCape) {
      if (capeid == 'no cape') currentRender.value.resetCape()
      else {
        const capeurl = await get_cape_data(selectedSkin.value.cape, 'url').catch(handleError)
        currentRender.value.loadCape(capeurl)
      }
    } else {
      notificationsWrapper.value.addNotification({
        title: 'Error Uploading Cape',
        text: 'Improper response from Mojang API. Please try again soon',
        type: 'error',
      })
    }
    uploadingSkin.value = false
    get_heads()
  }

  skinModal.value.hide()
}

const edit_skin = async (data) => {
  changeSkinType.value = 'from file'
  editSkin.value = true
  validSkin.value = true
  displaySkin.value = data.skin
  selectedSkin.value.skin = data.skin
  selectedSkin.value.name = data.name
  selectedSkin.value.id = data.id
  selectedSkin.value.user = data.user
  await skinModal.value.show()
  selectedSkin.value.arms = data.arms
  selectedSkin.value.cape = data.cape
  await create_modal_render()
}

const edit_skin_end = async () => {
  const data = {
    skin: selectedSkin.value.skin,
    cape: selectedSkin.value.cape,
    arms: selectedSkin.value.arms,
    unlocked_capes: [],
  }

  const model = await get_render(data).catch(handleError)
  await save_skin(
    selectedSkin.value.user,
    data,
    selectedSkin.value.name.trim(),
    model,
    selectedSkin.value.id,
  ).catch(handleError)
  skinSaves.value = await get_skins().catch(handleError)
  skinClear()
  editSkin.value = false

  skinModal.value.hide()
  InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError,
  )
}

const duplicate_skin = async (args) => {
  let data = {}
  data.skin = args.skin
  data.cape = args.cape
  data.arms = args.arms
  data.unlocked_capes = []
  await save_skin(selectedAccount.value.id, data, args.name, args.model, '').catch(handleError)
  skinSaves.value = await get_skins().catch(handleError)
  skinOrder.value = await get_order(selectedAccount.value.id).catch(handleError)
}

const openskin = async () => {
  selectedSkin.value.skin = await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png'],
      },
    ],
  })
  if (!selectedSkin.value.skin) {
    skinClear()
    return
  }
  validSkin.value = await check_image(selectedSkin.value.skin).catch(handleError)
  if (!validSkin.value) {
    skinClear()
    return
  }
  displaySkin.value = tauri.convertFileSrc(selectedSkin.value.skin)
  create_modal_render()
}

const create_modal_render = async () => {
  modalRender.value = new SkinViewer({
    canvas: document.getElementById('new_render'),
    width: 247.5,
    height: 330,
  })
  modalRender.value.animation = new IdleAnimation()
  modalRender.value.controls.enableZoom = false
  modalRender.value.loadSkin(displaySkin.value, { model: convert_arms(selectedSkin.value.arms) })
  if (selectedSkin.value.cape !== 'no cape')
    modalRender.value.loadCape(
      await get_cape_data(selectedSkin.value.cape, 'url').catch(handleError),
    )
}

const create_render = async () => {
  skinData.value = await get_user_skin_data(selectedAccount.value.id).catch(handleError)
  InLibrary.value = await check_skin(skinData.value.skin, selectedAccount.value.id).catch(
    handleError,
  )
  const arms = convert_arms(skinData.value.arms)
  const cape = await get_cape_data(skinData.value.cape, 'url').catch(handleError)
  currentRender.value = new SkinViewer({
    canvas: document.getElementById('skin_container'),
    width: 300,
    height: 400,
    skin: skinData.value.skin,
    model: arms,
  })
  if (cape !== 'no cape') currentRender.value.loadCape(cape)
  currentRender.value.animation = new WalkingAnimation()
  currentRender.value.animation.speed = 0.5
  currentRender.value.animation.headBobbing = false
  currentRender.value.controls.enableZoom = false
}

const update_render = async (account) => {
  if (currentRender.value == null) return
  skinData.value = await get_user_skin_data(account).catch(handleError)
  InLibrary.value = await check_skin(skinData.value.skin, account).catch(handleError)
  currentRender.value.loadSkin(skinData.value.skin, {
    model: convert_arms(skinData.value.arms),
  })
  if (skinData.value.cape == 'no cape') currentRender.value.resetCape()
  else
    currentRender.value.loadCape(await get_cape_data(skinData.value.cape, 'url').catch(handleError))
}

const handleModalType = async () => {
  if (changeSkinType.value == 'import from launcher') {
    handleImportType()
  }
}

const handleImportType = async () => {
  if (importType.value == 'Mojang') importer.value.display = '.minecraft'
  else importer.value.display = importType.value
  importer.value.path = await get_default_launcher_path(importType.value).catch(handleError)
  if (importer.value.path !== null) {
    importer.value.skinNames = await get_launcher_names(
      importer.value.path,
      importType.value,
    ).catch(handleError)
  } else importer.value.path = ''
}

watch(selectedAccount, async (newAccount) => {
  skinOrder.value = await get_order(newAccount.id).catch(handleError)
  await update_render(newAccount.id)
})

watch(
  loaded_skins,
  async (val) => {
    if (val) {
      await update_render(selectedAccount.value.id)
    }
  },
  { once: true },
)

function convert_arms(arms) {
  if (arms == 'classic') arms = 'default'
  return arms
}

onMounted(async () => {
  await create_render()
})

onBeforeUnmount(async () => {
  await save_filters()
})
</script>

<style scoped lang="scss">
.content {
  display: flex;
  flex-direction: row;
  width: 100%;
  padding: 1rem;
  gap: 1rem;

  -ms-overflow-style: none;
  scrollbar-width: none;

  &::-webkit-scrollbar {
    width: 0;
    background: transparent;
  }
}

.overlap {
  display: grid;
  justify-items: start;
  align-items: end;

  .loading {
    margin: 0;
    padding: 0;
    width: 1rem;
    height: 1rem;
  }
}
.overlap > * {
  grid-column-start: 1;
  grid-row-start: 1;
}

.modal {
  position: absolute;
}

.image-upload {
  display: flex;
  gap: 1rem;
}

.image-input {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  justify-content: center;
}

.input-label {
  font-size: 1rem;
  font-weight: bolder;
  color: var(--color-contrast);
  margin-bottom: 0.5rem;
}

.modal-header {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: var(--gap-lg);
  padding-bottom: 0;
}

.modal-column {
  display: flex;
  flex-direction: column;
  padding: var(--gap-lg);
  gap: var(--gap-sm);
}

.modal-row {
  display: flex;
  flex-direction: row;
  gap: var(--gap-sm);
}

.row {
  display: flex;
  flex-wrap: wrap;
  width: 100%;
  gap: 1rem;

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

  .labeled_button {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
  }
}

.render {
  cursor: pointer;
}

.instance-card-item {
  display: block;
  flex: none;
  align-items: center;
  justify-content: center;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .card-row {
    display: flex;
    flex-direction: row;
  }

  .mod-image {
    --size: 100%;
    width: 144px;
    height: 144px;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      width: 144px;
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

.modal-body {
  display: flex;
  flex-direction: column;
  padding: var(--gap-lg);
  gap: var(--gap-md);
}

.input-label {
  font-size: 1rem;
  font-weight: bolder;
  color: var(--color-contrast);
  margin-bottom: 0.5rem;
}

.text-input {
  width: 20rem;
}

.image-upload {
  display: flex;
  gap: 1rem;
}

.image-input {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  justify-content: center;
}

.warning {
  font-style: italic;
}

.modal-header {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: var(--gap-lg);
  padding-bottom: 0;
}

.path-selection {
  padding: var(--gap-xl);
  background-color: var(--color-bg);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  gap: var(--gap-md);

  h3 {
    margin: 0;
  }

  .path-input {
    display: flex;
    align-items: center;
    width: 100%;
    flex-direction: row;
    gap: var(--gap-sm);

    .iconified-input {
      flex-grow: 1;
      :deep(input) {
        width: 100%;
        flex-basis: auto;
      }
    }
  }
}

.table {
  border: 1px solid var(--color-bg);
}

.table-row {
  grid-template-columns: min-content auto;
}

.table-content {
  max-height: calc(5 * (18px + 2rem));
  height: calc(5 * (18px + 2rem));
  overflow-y: auto;
}

.select-checkbox {
  button.checkbox {
    border: none;
  }
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: var(--gap-md);

  .transparent {
    padding: var(--gap-sm) 0;
  }
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  font-weight: bolder;
  color: var(--color-contrast);
}

.card-divider {
  margin: var(--gap-md) var(--gap-lg) 0 var(--gap-lg);
}
</style>
