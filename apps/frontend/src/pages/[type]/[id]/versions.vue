<template>
  <div class="content">
    <div v-if="currentMember" class="card header-buttons">
      <FileInput
        :max-size="524288000"
        :accept="acceptFileFromProjectType(project.project_type)"
        prompt="Upload a version"
        class="iconified-button brand-button"
        :disabled="!isPermission(currentMember?.permissions, 1 << 0)"
        @change="handleFiles"
      >
        <UploadIcon />
      </FileInput>
      <span class="indicator">
        <InfoIcon /> Click to choose a file or drag one onto this page
      </span>
      <DropArea :accept="acceptFileFromProjectType(project.project_type)" @change="handleFiles" />
    </div>
    <VersionFilterControl :versions="props.versions" @switch-page="switchPage" />
    <Pagination
      :page="currentPage"
      :count="Math.ceil(filteredVersions.length / 20)"
      class="pagination-before"
      :link-function="(page) => `?page=${page}`"
      @switch-page="switchPage"
    />
    <div v-if="filteredVersions.length > 0" id="all-versions" class="universal-card all-versions">
      <div class="header">
        <div />
        <div>Version</div>
        <div>Supports</div>
        <div>Stats</div>
      </div>
      <div
        v-for="version in filteredVersions.slice((currentPage - 1) * 20, currentPage * 20)"
        :key="version.id"
        class="version-button button-transparent"
        @click="
          $router.push(
            `/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/version/${encodeURI(version.displayUrlEnding)}`,
          )
        "
      >
        <a
          v-tooltip="
            version.primaryFile.filename + ' (' + $formatBytes(version.primaryFile.size) + ')'
          "
          :href="version.primaryFile.url"
          class="download-button square-button brand-button"
          :class="version.version_type"
          :aria-label="`Download ${version.name}`"
          @click.stop="(event) => event.stopPropagation()"
        >
          <DownloadIcon aria-hidden="true" />
        </a>
        <nuxt-link
          :to="`/${project.project_type}/${
            project.slug ? project.slug : project.id
          }/version/${encodeURI(version.displayUrlEnding)}`"
          class="version__title"
        >
          {{ version.name }}
        </nuxt-link>
        <div class="version__metadata">
          <Badge v-if="version.version_type === 'release'" type="release" color="green" />
          <Badge v-else-if="version.version_type === 'beta'" type="beta" color="orange" />
          <Badge v-else-if="version.version_type === 'alpha'" type="alpha" color="red" />
          <span class="divider" />
          <span class="version_number">{{ version.version_number }}</span>
        </div>
        <div class="version__supports">
          <span>
            {{ version.loaders.map((x) => $formatCategory(x)).join(", ") }}
          </span>
          <span>{{ $formatVersion(version.game_versions) }}</span>
        </div>
        <div class="version__stats">
          <span>
            <strong>{{ $formatNumber(version.downloads) }}</strong>
            download<span v-if="version.downloads !== 1">s</span>
          </span>
          <span>
            Published on
            <strong>{{ $dayjs(version.date_published).format("MMM D, YYYY") }}</strong>
          </span>
        </div>
      </div>
    </div>
    <Pagination
      :page="currentPage"
      :count="Math.ceil(filteredVersions.length / 20)"
      class="pagination-before"
      :link-function="(page) => `?page=${page}`"
      @switch-page="switchPage"
    />
  </div>
</template>
<script setup>
import { Badge, FileInput, DropArea } from "@modrinth/ui";
import { acceptFileFromProjectType } from "~/helpers/fileUtils.js";
import DownloadIcon from "~/assets/images/utils/download.svg?component";
import UploadIcon from "~/assets/images/utils/upload.svg?component";
import InfoIcon from "~/assets/images/utils/info.svg?component";
import Pagination from "~/components/ui/Pagination.vue";
import VersionFilterControl from "~/components/ui/VersionFilterControl.vue";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
  members: {
    type: Array,
    default() {
      return [];
    },
  },
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
});

const data = useNuxtApp();

const title = `${props.project.title} - Versions`;
const description = `Download and browse ${props.versions.length} ${
  props.project.title
} versions. ${data.$formatNumber(props.project.downloads)} total downloads. Last updated ${data
  .$dayjs(props.project.updated)
  .format("MMM D, YYYY")}.`;

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});

const router = useNativeRouter();
const route = useNativeRoute();

const currentPage = ref(Number(route.query.p ?? 1));
const filteredVersions = computed(() => {
  const selectedGameVersions = getArrayOrString(route.query.g) ?? [];
  const selectedLoaders = getArrayOrString(route.query.l) ?? [];
  const selectedVersionTypes = getArrayOrString(route.query.c) ?? [];

  return props.versions.filter(
    (projectVersion) =>
      (selectedGameVersions.length === 0 ||
        selectedGameVersions.some((gameVersion) =>
          projectVersion.game_versions.includes(gameVersion),
        )) &&
      (selectedLoaders.length === 0 ||
        selectedLoaders.some((loader) => projectVersion.loaders.includes(loader))) &&
      (selectedVersionTypes.length === 0 ||
        selectedVersionTypes.includes(projectVersion.version_type)),
  );
});

function switchPage(page) {
  currentPage.value = page;

  router.replace({
    query: {
      ...route.query,
      p: currentPage.value !== 1 ? currentPage.value : undefined,
    },
  });
}

async function handleFiles(files) {
  await router.push({
    name: "type-id-version-version",
    params: {
      type: props.project.project_type,
      id: props.project.slug ? props.project.slug : props.project.id,
      version: "create",
    },
    state: {
      newPrimaryFile: files[0],
    },
  });
}
</script>

<style lang="scss" scoped>
.header-buttons {
  display: flex;
  align-items: center;
  gap: 1rem;

  .indicator {
    display: flex;
    gap: 0.5ch;
    align-items: center;
    color: var(--color-text-inactive);
  }
}

.all-versions {
  display: flex;
  flex-direction: column;

  .header {
    display: grid;
    grid-template: "download title supports stats";
    grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1.25fr 1fr 1fr;
    color: var(--color-text-dark);
    font-size: var(--font-size-md);
    font-weight: bold;
    justify-content: left;
    margin-inline: var(--spacing-card-md);
    margin-bottom: var(--spacing-card-sm);
    column-gap: var(--spacing-card-sm);

    div:first-child {
      grid-area: download;
    }

    div:nth-child(2) {
      grid-area: title;
    }

    div:nth-child(3) {
      grid-area: supports;
    }

    div:nth-child(4) {
      grid-area: stats;
    }
  }

  .version-button {
    display: grid;
    grid-template:
      "download title supports stats"
      "download metadata supports stats"
      "download dummy supports stats";
    grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1.25fr 1fr 1fr;
    column-gap: var(--spacing-card-sm);
    justify-content: left;
    padding: var(--spacing-card-md);

    .download-button {
      grid-area: download;
    }
    .version__title {
      grid-area: title;
      font-weight: bold;

      svg {
        vertical-align: top;
      }
    }
    .version__metadata {
      grid-area: metadata;
      display: flex;
      flex-direction: row;
      flex-wrap: wrap;
      gap: var(--spacing-card-xs);
      margin-top: var(--spacing-card-xs);
    }
    .version__supports {
      grid-area: supports;
      display: flex;
      flex-direction: column;
      gap: var(--spacing-card-xs);
    }
    .version__stats {
      grid-area: stats;
      display: flex;
      flex-direction: column;
      gap: var(--spacing-card-xs);
    }
  }
}

@media screen and (max-width: 1024px) {
  .all-versions {
    .header {
      grid-template: "download title";
      grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1fr;

      div:nth-child(3) {
        display: none;
      }

      div:nth-child(4) {
        display: none;
      }
    }

    .version-button {
      grid-template: "download title" "download metadata" "download supports" "download stats";
      grid-template-columns: calc(2.25rem + var(--spacing-card-sm)) 1fr;
      row-gap: var(--spacing-card-xs);

      .version__supports {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        column-gap: var(--spacing-card-xs);
      }
      .version__metadata {
        margin: 0;
      }
    }
  }
}

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
