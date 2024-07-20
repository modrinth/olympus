<template>
  <Card class="filter-header">
    <div class="manage">
      <multiselect
        v-model="filterLoader"
        :options="possibleLoaders"
        :multiple="true"
        :searchable="true"
        :show-no-results="false"
        :close-on-select="false"
        :clear-search-on-select="false"
        :show-labels="false"
        :selectable="() => versions.length <= 6"
        placeholder="Filter loader..."
        :custom-label="(option) => option.charAt(0).toUpperCase() + option.slice(1)"
      />
      <multiselect
        v-model="filterGameVersions"
        :options="possibleGameVersions"
        :multiple="true"
        :searchable="true"
        :show-no-results="false"
        :close-on-select="false"
        :clear-search-on-select="false"
        :show-labels="false"
        :selectable="() => versions.length <= 6"
        placeholder="Filter versions..."
        :custom-label="(option) => option.charAt(0).toUpperCase() + option.slice(1)"
      />
      <multiselect
        v-model="filterVersions"
        :options="possibleVersions"
        :multiple="true"
        :searchable="true"
        :show-no-results="false"
        :close-on-select="false"
        :clear-search-on-select="false"
        :show-labels="false"
        :selectable="() => versions.length <= 6"
        placeholder="Filter release channel..."
        :custom-label="(option) => option.charAt(0).toUpperCase() + option.slice(1)"
      />
    </div>
    <Button
      class="no-wrap clear-filters"
      :disabled="
        filterVersions.length === 0 && filterLoader.length === 0 && filterGameVersions.length === 0
      "
      :action="clearFilters"
    >
      <ClearIcon />
      Clear filters
    </Button>
  </Card>
  <Pagination
    :page="currentPage"
    :count="Math.ceil(filteredVersions.length / 20)"
    class="pagination-before"
    :link-function="(page) => `?page=${page}`"
    @switch-page="switchPage"
  />
  <Card class="mod-card">
    <div class="table">
      <div class="table-row table-head">
        <div class="table-cell table-text download-cell" />
        <div class="name-cell table-cell table-text">Name</div>
        <div class="table-cell table-text">Supports</div>
        <div class="table-cell table-text">Stats</div>
      </div>
      <div
        v-for="version in filteredVersions.slice((currentPage - 1) * 20, currentPage * 20)"
        :key="version.id"
        class="table-row selectable"
        @click="$router.push(`/project/${$route.params.id}/version/${version.id}`)"
      >
        <div class="table-cell table-text">
          <Button
            :color="installed && version.id === installedVersion ? '' : 'primary'"
            icon-only
            :disabled="installing || (installed && version.id === installedVersion)"
            @click.stop="() => install(version.id)"
          >
            <DownloadIcon v-if="!installed" />
            <SwapIcon v-else-if="installed && version.id !== installedVersion" />
            <CheckIcon v-else />
          </Button>
        </div>
        <div class="name-cell table-cell table-text">
          <div class="version-link">
            {{ version.name.charAt(0).toUpperCase() + version.name.slice(1) }}
            <div class="version-badge">
              <div class="channel-indicator">
                <Badge
                  :color="releaseColor(version.version_type)"
                  :type="
                    version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)
                  "
                />
              </div>
              <div>
                {{ version.version_number }}
              </div>
            </div>
          </div>
        </div>
        <div class="table-cell table-text stacked-text">
          <span>
            {{
              version.loaders.map((str) => str.charAt(0).toUpperCase() + str.slice(1)).join(', ')
            }}
          </span>
          <span>
            {{ version.game_versions.join(', ') }}
          </span>
        </div>
        <div class="table-cell table-text stacked-text">
          <div>
            <span> Published on </span>
            <strong>
              {{
                new Date(version.date_published).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'short',
                  day: 'numeric',
                })
              }}
            </strong>
          </div>
          <div>
            <strong>
              {{ formatNumber(version.downloads) }}
            </strong>
            <span> Downloads </span>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, Pagination, Badge } from '@modrinth/ui'
import { CheckIcon, ClearIcon, DownloadIcon } from '@modrinth/assets'
import { formatNumber } from '@modrinth/utils'
import Multiselect from 'vue-multiselect'
import { releaseColor } from '@/helpers/utils'
import { computed, ref, watch } from 'vue'
import { SwapIcon } from '@/assets/icons/index.js'

const props = defineProps({
  versions: {
    type: Array,
    required: true,
  },
  install: {
    type: Function,
    required: true,
  },
  installed: {
    type: Boolean,
    default: null,
  },
  installing: {
    type: Boolean,
    default: false,
  },
  instance: {
    type: Object,
    default: null,
  },
  installedVersion: {
    type: String,
    default: null,
  },
})

const possibleVersions = computed(() => {
  return props.versions
    .flatMap((value) => value.version_type)
    .filter((value, index, self) => self.indexOf(value) === index)
})
const possibleLoaders = computed(() => {
  return props.versions
    .flatMap((value) => value.loaders)
    .filter((value, index, self) => self.indexOf(value) === index)
})
const possibleGameVersions = computed(() => {
  return props.versions
    .flatMap((value) => value.game_versions)
    .filter((value, index, self) => self.indexOf(value) === index)
})

const filterVersions = ref([])
const filterLoader = ref([])
const filterGameVersions = ref([])

if (props.instance) {
  let loader = props.instance.metadata?.loader
  if (possibleLoaders.value.includes(loader)) {
    filterLoader.value.push(loader)
  }

  let gameVersion = props.instance.metadata?.game_version
  if (possibleGameVersions.value.includes(gameVersion)) {
    filterGameVersions.value.push(gameVersion)
  }
}

const currentPage = ref(1)

const clearFilters = () => {
  filterVersions.value = []
  filterLoader.value = []
  filterGameVersions.value = []
}

const filteredVersions = computed(() => {
  return props.versions.filter(
    (projectVersion) =>
      (filterGameVersions.value.length === 0 ||
        filterGameVersions.value.some((gameVersion) =>
          projectVersion.game_versions.includes(gameVersion),
        )) &&
      (filterLoader.value.length === 0 ||
        filterLoader.value.some((loader) => projectVersion.loaders.includes(loader))) &&
      (filterVersions.value.length === 0 ||
        filterVersions.value.includes(projectVersion.version_type)),
  )
})

function switchPage(page) {
  currentPage.value = page
}

//watch all the filters and if a value changes, reset to page 1
watch([filterVersions, filterLoader, filterGameVersions], () => {
  currentPage.value = 1
})
</script>

<style scoped lang="scss">
.filter-header {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.table-row {
  grid-template-columns: min-content 1fr 1fr 1.5fr;
}

.manage {
  display: flex;
  gap: 0.5rem;
  flex-grow: 1;

  .multiselect {
    flex-grow: 1;
  }
}

.card-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: var(--color-raised-bg);
}

.mod-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  overflow: hidden;
  margin-top: 0.5rem;
}

.text-combo {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.select {
  width: 100% !important;
  max-width: 20rem;
}

.version-link {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  text-wrap: wrap;

  .version-badge {
    display: flex;
    flex-wrap: wrap;

    .channel-indicator {
      margin-right: 0.5rem;
    }
  }
}

.stacked-text {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  align-items: flex-start;
}

.download-cell {
  width: 4rem;
  padding: 1rem;
}

.filter-checkbox {
  :deep(.checkbox) {
    border: none;
  }
}
</style>
