<template>
  <div
    v-if="
      loaderFilters.length > 1 || gameVersionFilters.length > 1 || versionTypeFilters.length > 1
    "
    class="card search-controls"
  >
    <Multiselect
      v-if="loaderFilters.length > 1"
      v-model="selectedLoaders"
      :options="loaderFilters"
      :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
      :multiple="true"
      :searchable="false"
      :show-no-results="false"
      :close-on-select="true"
      :clear-search-on-select="false"
      :show-labels="false"
      :allow-empty="true"
      placeholder="Filter loader..."
      @update:model-value="updateQuery"
    />
    <Multiselect
      v-if="gameVersionFilters.length > 1"
      v-model="selectedGameVersions"
      :options="
        includeSnapshots
          ? gameVersionFilters.map((x) => x.version)
          : gameVersionFilters.filter((it) => it.version_type === 'release').map((x) => x.version)
      "
      :multiple="true"
      :searchable="true"
      :show-no-results="false"
      :close-on-select="false"
      :show-labels="false"
      :hide-selected="true"
      :selectable="() => selectedGameVersions.length <= 6"
      placeholder="Filter versions..."
      @update:model-value="updateQuery"
    />
    <Multiselect
      v-if="versionTypeFilters.length > 1"
      v-model="selectedVersionTypes"
      :options="versionTypeFilters"
      :custom-label="(x) => $capitalizeString(x)"
      :multiple="true"
      :searchable="false"
      :show-no-results="false"
      :close-on-select="true"
      :clear-search-on-select="false"
      :show-labels="false"
      :allow-empty="true"
      placeholder="Filter channels..."
      @update:model-value="updateQuery"
    />
    <Checkbox
      v-if="
        gameVersionFilters.length > 1 &&
        gameVersionFilters.some((v) => v.version_type !== 'release')
      "
      v-model="includeSnapshots"
      label="Show all versions"
      description="Show all versions"
      :border="false"
      @update:model-value="updateQuery"
    />
    <button
      title="Clear filters"
      :disabled="selectedLoaders.length === 0 && selectedGameVersions.length === 0"
      class="iconified-button"
      @click="
        () => {
          selectedLoaders = [];
          selectedGameVersions = [];
          selectedVersionTypes = [];
          updateQuery();
        }
      "
    >
      <ClearIcon />
      Clear filters
    </button>
  </div>
</template>

<script setup>
import { Multiselect } from "vue-multiselect";
import { Checkbox } from "@modrinth/ui";
import ClearIcon from "~/assets/images/utils/clear.svg?component";

const props = defineProps({
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
});
const emit = defineEmits(["switch-page"]);

const router = useNativeRouter();
const route = useNativeRoute();

const tags = useTags();

const tempLoaders = new Set();
let tempVersions = new Set();
const tempReleaseChannels = new Set();

for (const version of props.versions) {
  for (const loader of version.loaders) {
    tempLoaders.add(loader);
  }
  for (const gameVersion of version.game_versions) {
    tempVersions.add(gameVersion);
  }
  tempReleaseChannels.add(version.version_type);
}

tempVersions = Array.from(tempVersions);

const loaderFilters = shallowRef(Array.from(tempLoaders));
const gameVersionFilters = shallowRef(
  tags.value.gameVersions.filter((gameVer) => tempVersions.includes(gameVer.version)),
);
const versionTypeFilters = shallowRef(Array.from(tempReleaseChannels));
const includeSnapshots = ref(route.query.s === "true");

const selectedGameVersions = shallowRef(getArrayOrString(route.query.g) ?? []);
const selectedLoaders = shallowRef(getArrayOrString(route.query.l) ?? []);
const selectedVersionTypes = shallowRef(getArrayOrString(route.query.c) ?? []);

async function updateQuery() {
  await router.replace({
    query: {
      ...route.query,
      l: selectedLoaders.value.length === 0 ? undefined : selectedLoaders.value,
      g: selectedGameVersions.value.length === 0 ? undefined : selectedGameVersions.value,
      c: selectedVersionTypes.value.length === 0 ? undefined : selectedVersionTypes.value,
      s: includeSnapshots.value ? true : undefined,
    },
  });
  emit("switch-page", 1);
}
</script>

<style lang="scss" scoped>
.search-controls {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-card-md);
  align-items: center;
  flex-wrap: wrap;
  .multiselect {
    flex: 1;
  }
  .checkbox-outer {
    min-width: fit-content;
  }
}
</style>
