<template>
  <div>
    <Card>
      <Breadcrumbs
        :current-title="version.name"
        :link-stack="[
          {
            href: `/project/${route.params.id}/versions`,
            label: 'Versions',
          },
        ]"
      />
      <div class="version-title">
        <h2>{{ version.name }}</h2>
      </div>
      <div class="button-group">
        <Button
          color="primary"
          :action="() => install(version.id)"
          :disabled="installing || (installed && installedVersion === version.id)"
        >
          <DownloadIcon v-if="!installed" />
          <SwapIcon v-else-if="installedVersion !== version.id" />
          <CheckIcon v-else />
          {{
            installing
              ? 'Installing...'
              : installed && installedVersion === version.id
                ? 'Installed'
                : 'Install'
          }}
        </Button>
        <a
          :href="`https://modrinth.com/report?item=version&itemID=${route.params.version}`"
          rel="external"
          class="btn"
        >
          <ReportIcon />
          Report
        </a>
        <a
          :href="`https://modrinth.com/mod/${route.params.id}/version/${route.params.version}`"
          rel="external"
          class="btn"
        >
          <ExternalIcon />
          Modrinth website
        </a>
      </div>
    </Card>
    <div class="version-container">
      <div class="description-cards">
        <Card>
          <h3 class="card-title">Changelog</h3>
          <div class="markdown-body" v-html="renderString(version.changelog ?? '')" />
        </Card>
        <Card>
          <h3 class="card-title">Files</h3>
          <Card
            v-for="file in version.files"
            :key="file.id"
            :class="{ primary: file.primary }"
            class="file"
          >
            <span class="label">
              <FileIcon />
              <span>
                <span class="title">
                  {{ file.filename }}
                </span>
                ({{ formatBytes(file.size) }})
                <span v-if="file.primary" class="primary-label"> Primary </span>
              </span>
            </span>
            <Button
              v-if="project.project_type !== 'modpack' || file.primary"
              class="download"
              :action="() => install(version.id)"
              :disabled="installed"
            >
              <DownloadIcon v-if="!installed" />
              <CheckIcon v-else />
              {{ installed ? 'Installed' : 'Install' }}
            </Button>
          </Card>
        </Card>
        <Card v-if="displayDependencies.length > 0">
          <h2>Dependencies</h2>
          <div v-for="dependency in displayDependencies" :key="dependency.title">
            <router-link v-if="dependency.link" class="btn dependency" :to="dependency.link">
              <Avatar size="sm" :src="dependency.icon" />
              <div>
                <span class="title"> {{ dependency.title }} </span> <br />
                <span> {{ dependency.subtitle }} </span>
              </div>
            </router-link>
            <div v-else class="dependency disabled" disabled="">
              <Avatar size="sm" :src="dependency.icon" />
              <div class="text">
                <div class="title">{{ dependency.title }}</div>
                <div>{{ dependency.subtitle }}</div>
              </div>
            </div>
          </div>
        </Card>
      </div>
      <Card class="metadata-card">
        <h3 class="card-title">Metadata</h3>
        <div class="metadata">
          <div class="metadata-item">
            <span class="metadata-label">Release Channel</span>
            <span class="metadata-value"
              ><Badge
                :color="releaseColor(version.version_type)"
                :type="
                  version.version_type.charAt(0).toUpperCase() + version.version_type.slice(1)
                "
            /></span>
          </div>
          <div class="metadata-item">
            <span class="metadata-label">Version Number</span>
            <span class="metadata-value">{{ version.version_number }}</span>
          </div>
          <div class="metadata-item">
            <span class="metadata-label">Loaders</span>
            <span class="metadata-value">{{
              version.loaders
                .map((loader) => loader.charAt(0).toUpperCase() + loader.slice(1))
                .join(', ')
            }}</span>
          </div>
          <div class="metadata-item">
            <span class="metadata-label">Game Versions</span>
            <span class="metadata-value"> {{ version.game_versions.join(', ') }} </span>
          </div>
          <div class="metadata-item">
            <span class="metadata-label">Downloads</span>
            <span class="metadata-value">{{ version.downloads }}</span>
          </div>
          <div class="metadata-item">
            <span class="metadata-label">Publication Date</span>
            <span class="metadata-value">
              {{
                new Date(version.date_published).toLocaleString('en-US', {
                  month: 'long',
                  day: 'numeric',
                  year: 'numeric',
                })
              }}
              at
              {{
                new Date(version.date_published).toLocaleString('en-US', {
                  hour: 'numeric',
                  minute: 'numeric',
                  second: 'numeric',
                  hour12: true,
                })
              }}
            </span>
          </div>
          <div v-if="author" class="metadata-item">
            <span class="metadata-label">Author</span>
            <a
              :href="`https://modrinth.com/user/${author.user.username}`"
              rel="external"
              class="metadata-value btn author"
            >
              <Avatar size="sm" :src="author.user.avatar_url" circle />
              <span>
                <strong>
                  {{ author.user.username }}
                </strong>
                <br />
                {{ author.role }}
              </span>
            </a>
          </div>
          <div class="metadata-item">
            <span class="metadata-label">Version ID</span>
            <span class="metadata-value"><CopyCode class="copycode" :text="version.id" /></span>
          </div>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup>
import { DownloadIcon, FileIcon, ReportIcon, ExternalIcon, CheckIcon } from '@modrinth/assets'
import { formatBytes, renderString } from '@modrinth/utils'
import { Breadcrumbs, Badge, Avatar, Card, Button, CopyCode } from '@modrinth/ui'
import { releaseColor } from '@/helpers/utils'
import { ref, watch, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { SwapIcon } from '@/assets/icons'
import { get_project_many, get_version_many } from '@/helpers/cache.js'

const breadcrumbs = useBreadcrumbs()

const route = useRoute()

const props = defineProps({
  project: {
    type: Object,
    required: true,
  },
  versions: {
    type: Array,
    required: true,
  },
  members: {
    type: Array,
    required: true,
  },
  install: {
    type: Function,
    required: true,
  },
  installed: {
    type: Boolean,
    required: true,
  },
  installing: {
    type: Boolean,
    required: true,
  },
  installedVersion: {
    type: String,
    required: true,
  },
})

const version = ref(props.versions.find((version) => version.id === route.params.version))
breadcrumbs.setName('Version', version.value.name)

watch(
  () => props.versions,
  async () => {
    if (route.params.version) {
      version.value = props.versions.find((version) => version.id === route.params.version)
      await refreshDisplayDependencies()
      breadcrumbs.setName('Version', version.value.name)
    }
  },
)

const author = computed(() =>
  props.members ? props.members.find((member) => member.user.id === version.value.author_id) : null,
)

const displayDependencies = ref({})

async function refreshDisplayDependencies() {
  const projectIds = new Set()
  const versionIds = new Set()
  if (version.value.dependencies) {
    for (const dependency of version.value.dependencies) {
      if (dependency.project_id) {
        projectIds.add(dependency.project_id)
      }
      if (dependency.version_id) {
        versionIds.add(dependency.version_id)
      }
    }
  }
  const [projectDeps, versionDeps] = await Promise.all([
    get_project_many([...projectIds]),
    get_version_many([...versionIds]),
  ])

  const dependencies = {
    projects: projectDeps,
    versions: versionDeps,
  }

  displayDependencies.value = version.value.dependencies.map((dependency) => {
    const version = dependencies.versions.find((obj) => obj.id === dependency.version_id)
    if (version) {
      const project = dependencies.projects.find(
        (obj) => obj.id === version.project_id || obj.id === dependency.project_id,
      )
      return {
        icon: project?.icon_url,
        title: project?.title || project?.name,
        subtitle: `Version ${version.version_number} is ${dependency.dependency_type}`,
        link: `/project/${project.slug}/version/${version.id}`,
      }
    } else {
      const project = dependencies.projects.find((obj) => obj.id === dependency.project_id)

      if (project) {
        return {
          icon: project?.icon_url,
          title: project?.title || project?.name,
          subtitle: `${dependency.dependency_type}`,
          link: `/project/${project.slug}`,
        }
      } else {
        return {
          icon: null,
          title: dependency.file_name,
          subtitle: `Added via overrides`,
          link: null,
        }
      }
    }
  })
}
await refreshDisplayDependencies()
</script>

<style scoped lang="scss">
.version-container {
  display: flex;
  flex-direction: row;
  gap: 1rem;
}

.version-title {
  margin-bottom: 1rem;
  h2 {
    font-size: var(--font-size-2xl);
    font-weight: 700;
    color: var(--color-contrast);
    margin: 0;
  }
}

.dependency {
  display: flex;
  padding: 0.5rem 1rem 0.5rem 0.5rem;
  gap: 0.5rem;
  background: var(--color-raised-bg);
  color: var(--color-base);
  width: 100%;

  .title {
    font-weight: bolder;
  }

  :deep(svg) {
    margin-right: 0 !important;
  }
}

.file {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  background: var(--color-button-bg);
  color: var(--color-base);
  padding: 0.5rem 1rem;

  .download {
    margin-left: auto;
    background-color: var(--color-raised-bg);
  }

  .label {
    display: flex;
    margin: auto 0 auto;
    gap: 0.5rem;

    .title {
      font-weight: bolder;
      word-break: break-all;
    }

    svg {
      min-width: 1.1rem;
      min-height: 1.1rem;
      width: 1.1rem;
      height: 1.1rem;
      margin: auto 0;
    }

    .primary-label {
      font-style: italic;
    }
  }
}

.primary {
  background: var(--color-brand-highlight);
  color: var(--color-contrast);
}

.button-group {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 0.5rem;
}

.card-title {
  font-size: var(--font-size-lg);
  color: var(--color-contrast);
  margin: 0 0 0.5rem;
}

.description-cards {
  width: 100%;
}

.metadata-card {
  width: 20rem;
  height: min-content;
}

.metadata {
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: 1rem;

  .metadata-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;

    .metadata-label {
      font-weight: bold;
    }
  }
}

.author {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
  align-items: center;
  text-decoration: none;
  color: var(--color-base);
  background: var(--color-raised-bg);
  padding: 0.5rem;
  width: 100%;
  box-shadow: none;
}

.markdown-body {
  :deep(hr),
  :deep(h1),
  :deep(h2),
  img {
    max-width: max(60rem, 90%) !important;
  }

  :deep(ul),
  :deep(ol) {
    margin-left: 2rem;
  }
}

.copycode {
  border: 0;
  color: var(--color-contrast);
}

.disabled {
  display: flex;
  flex-direction: row;
  vertical-align: center;
  align-items: center;
  cursor: not-allowed;
  border-radius: var(--radius-lg);

  .text {
    filter: brightness(0.5);
  }
}
</style>
