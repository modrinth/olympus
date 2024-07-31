<template>
  <div v-if="user">
    <ModalCreation ref="modal_creation" />
    <CollectionCreateModal ref="modal_collection_creation" />
    <div class="user-header-wrapper">
      <div class="user-header">
        <Avatar :src="user.avatar_url" size="md" circle :alt="user.username" />
        <h1 class="username">
          {{ user.username }}
        </h1>
      </div>
    </div>
    <div class="normal-page">
      <div class="normal-page__sidebar">
        <div class="card sidebar">
          <h1 class="mobile-username">
            {{ user.username }}
          </h1>
          <div class="card__overlay">
            <NuxtLink
              v-if="auth.user && auth.user.id === user.id"
              to="/settings/profile"
              class="iconified-button"
            >
              <EditIcon />
              {{ formatMessage(commonMessages.editButton) }}
            </NuxtLink>
            <button
              v-else-if="auth.user"
              class="iconified-button"
              @click="() => reportUser(user.id)"
            >
              <ReportIcon aria-hidden="true" />
              {{ formatMessage(messages.profileReportButton) }}
            </button>
            <nuxt-link v-else class="iconified-button" to="/auth/sign-in">
              <ReportIcon aria-hidden="true" />
              {{ formatMessage(messages.profileReportButton) }}
            </nuxt-link>
          </div>
          <div class="sidebar__item">
            <Badge v-if="tags.staffRoles.includes(user.role)" :type="user.role" />
            <Badge v-else-if="projects.length > 0" type="creator" />
          </div>
          <span v-if="user.bio" class="sidebar__item bio">{{ user.bio }}</span>
          <hr class="card-divider" />
          <div class="primary-stat">
            <DownloadIcon class="primary-stat__icon" aria-hidden="true" />
            <div class="primary-stat__text">
              <IntlFormatted
                :message-id="messages.profileDownloadsStats"
                :values="{ count: formatCompactNumber(sumDownloads) }"
              >
                <template #stat="{ children }">
                  <span class="primary-stat__counter">
                    <component :is="() => normalizeChildren(children)" />
                  </span>
                </template>
              </IntlFormatted>
            </div>
          </div>
          <div class="primary-stat">
            <HeartIcon class="primary-stat__icon" aria-hidden="true" />
            <div class="primary-stat__text">
              <IntlFormatted
                :message-id="messages.profileProjectsFollowersStats"
                :values="{ count: formatCompactNumber(sumFollows) }"
              >
                <template #stat="{ children }">
                  <span class="primary-stat__counter">
                    <component :is="() => normalizeChildren(children)" />
                  </span>
                </template>
              </IntlFormatted>
            </div>
          </div>
          <div class="stats-block__item secondary-stat">
            <SunriseIcon class="secondary-stat__icon" aria-hidden="true" />
            <span
              v-tooltip="
                formatMessage(commonMessages.dateAtTimeTooltip, {
                  date: new Date(user.created),
                  time: new Date(user.created),
                })
              "
              class="secondary-stat__text date"
            >
              {{
                formatMessage(messages.profileJoinedAt, { ago: formatRelativeTime(user.created) })
              }}
            </span>
          </div>
          <hr class="card-divider" />
          <div class="stats-block__item secondary-stat">
            <UserIcon class="secondary-stat__icon" aria-hidden="true" />
            <span class="secondary-stat__text">
              <IntlFormatted :message-id="messages.profileUserId">
                <template #~id>
                  <CopyCode :text="user.id" />
                </template>
              </IntlFormatted>
            </span>
          </div>
          <template v-if="organizations.length > 0">
            <hr class="card-divider" />
            <div class="stats-block__item">
              <IntlFormatted :message-id="messages.profileOrganizations" />
              <div class="organizations-grid">
                <nuxt-link
                  v-for="org in organizations"
                  :key="org.id"
                  v-tooltip="org.name"
                  class="organization"
                  :to="`/organization/${org.slug}`"
                >
                  <Avatar :src="org.icon_url" :alt="'Icon for ' + org.name" size="xs" />
                </nuxt-link>
              </div>
            </div>
          </template>
        </div>
      </div>
      <div class="normal-page__content">
        <Promotion :external="false" query-param="" />
        <nav class="navigation-card">
          <NavRow
            :links="[
              {
                label: formatMessage(commonMessages.allProjectType),
                href: `/user/${user.username}`,
              },
              ...projectTypes.map((x) => {
                return {
                  label: formatMessage(getProjectTypeMessage(x, true)),
                  href: `/user/${user.username}/${x}s`,
                };
              }),
            ]"
          />
          <div class="input-group">
            <NuxtLink
              v-if="auth.user && auth.user.id === user.id"
              class="iconified-button"
              to="/dashboard/projects"
            >
              <SettingsIcon />
              {{ formatMessage(messages.profileManageProjectsButton) }}
            </NuxtLink>
            <button
              v-if="route.params.projectType !== 'collections'"
              v-tooltip="
                formatMessage(commonMessages[`${cosmetics.searchDisplayMode.user}InputView`])
              "
              :aria-label="
                formatMessage(commonMessages[`${cosmetics.searchDisplayMode.user}InputView`])
              "
              class="square-button"
              @click="cycleSearchDisplayMode()"
            >
              <GridIcon v-if="cosmetics.searchDisplayMode.user === 'grid'" />
              <ImageIcon v-else-if="cosmetics.searchDisplayMode.user === 'gallery'" />
              <ListIcon v-else />
            </button>
          </div>
        </nav>
        <div v-if="projects.length > 0">
          <div
            v-if="route.params.projectType !== 'collections'"
            :class="'project-list display-mode--' + cosmetics.searchDisplayMode.user"
          >
            <ProjectCard
              v-for="project in (route.params.projectType !== undefined
                ? projects.filter(
                    (x) =>
                      x.project_type ===
                      route.params.projectType.substr(0, route.params.projectType.length - 1),
                  )
                : projects
              )
                .slice()
                .sort((a, b) => b.downloads - a.downloads)"
              :id="project.slug || project.id"
              :key="project.id"
              :name="project.title"
              :display="cosmetics.searchDisplayMode.user"
              :featured-image="project.gallery.find((element) => element.featured)?.url"
              :description="project.description"
              :created-at="project.published"
              :updated-at="project.updated"
              :downloads="project.downloads.toString()"
              :follows="project.followers.toString()"
              :icon-url="project.icon_url"
              :categories="project.categories"
              :client-side="project.client_side"
              :server-side="project.server_side"
              :status="
                auth.user && (auth.user.id === user.id || tags.staffRoles.includes(auth.user.role))
                  ? project.status
                  : null
              "
              :type="project.project_type"
              :color="project.color"
            />
          </div>
        </div>
        <div v-else-if="route.params.projectType !== 'collections'" class="error">
          <UpToDate class="icon" /><br />
          <span v-if="auth.user && auth.user.id === user.id" class="preserve-lines text">
            <IntlFormatted :message-id="messages.profileNoProjectsAuthLabel">
              <template #create-link="{ children }">
                <a class="link" @click.prevent="$refs.modal_creation.show()">
                  <component :is="() => children" />
                </a>
              </template>
            </IntlFormatted>
          </span>
          <span v-else class="text">{{ formatMessage(messages.profileNoProjectsLabel) }}</span>
        </div>
        <div v-if="['collections'].includes(route.params.projectType)" class="collections-grid">
          <nuxt-link
            v-for="collection in collections"
            :key="collection.id"
            :to="`/collection/${collection.id}`"
            class="card collection-item"
          >
            <div class="collection">
              <Avatar :src="collection.icon_url" class="icon" />
              <div class="details">
                <h2 class="title">{{ collection.name }}</h2>
                <div class="stats">
                  <LibraryIcon />
                  Collection
                </div>
              </div>
            </div>
            <div class="description">
              {{ collection.description }}
            </div>
            <div class="stat-bar">
              <div class="stats"><BoxIcon /> {{ collection.projects?.length || 0 }} projects</div>
              <div class="stats">
                <template v-if="collection.status === 'listed'">
                  <WorldIcon />
                  <span> Public </span>
                </template>
                <template v-else-if="collection.status === 'unlisted'">
                  <LinkIcon />
                  <span> Unlisted </span>
                </template>
                <template v-else-if="collection.status === 'private'">
                  <LockIcon />
                  <span> Private </span>
                </template>
                <template v-else-if="collection.status === 'rejected'">
                  <XIcon />
                  <span> Rejected </span>
                </template>
              </div>
            </div>
          </nuxt-link>
        </div>
        <div
          v-if="route.params.projectType === 'collections' && collections.length === 0"
          class="error"
        >
          <UpToDate class="icon" /><br />
          <span v-if="auth.user && auth.user.id === user.id" class="preserve-lines text">
            <IntlFormatted :message-id="messages.profileNoCollectionsAuthLabel">
              <template #create-link="{ children }">
                <a class="link" @click.prevent="$refs.modal_collection_creation.show()">
                  <component :is="() => children" />
                </a>
              </template>
            </IntlFormatted>
          </span>
          <span v-else class="text">{{ formatMessage(messages.profileNoCollectionsLabel) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup>
import { LibraryIcon, BoxIcon, LinkIcon, LockIcon, XIcon } from "@modrinth/assets";
import { Promotion, Badge, CopyCode, Avatar } from "@modrinth/ui";
import ProjectCard from "~/components/ui/ProjectCard.vue";
import { reportUser } from "~/utils/report-helpers.ts";

import ReportIcon from "~/assets/images/utils/report.svg?component";
import SunriseIcon from "~/assets/images/utils/sunrise.svg?component";
import DownloadIcon from "~/assets/images/utils/download.svg?component";
import SettingsIcon from "~/assets/images/utils/settings.svg?component";
import UpToDate from "~/assets/images/illustrations/up_to_date.svg?component";
import UserIcon from "~/assets/images/utils/user.svg?component";
import EditIcon from "~/assets/images/utils/edit.svg?component";
import HeartIcon from "~/assets/images/utils/heart.svg?component";
import GridIcon from "~/assets/images/utils/grid.svg?component";
import ListIcon from "~/assets/images/utils/list.svg?component";
import ImageIcon from "~/assets/images/utils/image.svg?component";
import WorldIcon from "~/assets/images/utils/world.svg?component";
import ModalCreation from "~/components/ui/ModalCreation.vue";
import NavRow from "~/components/ui/NavRow.vue";
import CollectionCreateModal from "~/components/ui/CollectionCreateModal.vue";

const data = useNuxtApp();
const route = useNativeRoute();
const auth = await useAuth();
const cosmetics = useCosmetics();
const tags = useTags();

const vintl = useVIntl();
const { formatMessage } = vintl;

const formatCompactNumber = useCompactNumber();

const formatRelativeTime = useRelativeTime();

const messages = defineMessages({
  profileDownloadsStats: {
    id: "profile.stats.downloads",
    defaultMessage:
      "{count, plural, one {<stat>{count}</stat> download} other {<stat>{count}</stat> downloads}}",
  },
  profileProjectsFollowersStats: {
    id: "profile.stats.projects-followers",
    defaultMessage:
      "{count, plural, one {<stat>{count}</stat> follower} other {<stat>{count}</stat> followers}} of projects",
  },
  profileJoinedAt: {
    id: "profile.joined-at",
    defaultMessage: "Joined {ago}",
  },
  profileUserId: {
    id: "profile.user-id",
    defaultMessage: "User ID: {id}",
  },
  profileOrganizations: {
    id: "profile.label.organizations",
    defaultMessage: "Organizations",
  },
  profileManageProjectsButton: {
    id: "profile.button.manage-projects",
    defaultMessage: "Manage projects",
  },
  profileMetaDescription: {
    id: "profile.meta.description",
    defaultMessage: "Download {username}'s projects on Modrinth",
  },
  profileMetaDescriptionWithBio: {
    id: "profile.meta.description-with-bio",
    defaultMessage: "{bio} - Download {username}'s projects on Modrinth",
  },
  profileReportButton: {
    id: "profile.button.report",
    defaultMessage: "Report",
  },
  profileNoProjectsLabel: {
    id: "profile.label.no-projects",
    defaultMessage: "This user has no projects!",
  },
  profileNoProjectsAuthLabel: {
    id: "profile.label.no-projects-auth",
    defaultMessage:
      "You don't have any projects.\nWould you like to <create-link>create one</create-link>?",
  },
  profileNoCollectionsLabel: {
    id: "profile.label.no-collections",
    defaultMessage: "This user has no collections!",
  },
  profileNoCollectionsAuthLabel: {
    id: "profile.label.no-collections-auth",
    defaultMessage:
      "You don't have any collections.\nWould you like to <create-link>create one</create-link>?",
  },
  userNotFoundError: {
    id: "profile.error.not-found",
    defaultMessage: "User not found",
  },
});

let user, projects, organizations, collections;
try {
  [{ data: user }, { data: projects }, { data: organizations }, { data: collections }] =
    await Promise.all([
      useAsyncData(`user/${route.params.id}`, () => useBaseFetch(`user/${route.params.id}`)),
      useAsyncData(
        `user/${route.params.id}/projects`,
        () => useBaseFetch(`user/${route.params.id}/projects`),
        {
          transform: (projects) => {
            for (const project of projects) {
              project.categories = project.categories.concat(project.loaders);
              project.project_type = data.$getProjectTypeForUrl(
                project.project_type,
                project.categories,
                tags.value,
              );
            }

            return projects;
          },
        },
      ),
      useAsyncData(`user/${route.params.id}/organizations`, () =>
        useBaseFetch(`user/${route.params.id}/organizations`, {
          apiVersion: 3,
        }),
      ),
      useAsyncData(`user/${route.params.id}/collections`, () =>
        useBaseFetch(`user/${route.params.id}/collections`, { apiVersion: 3 }),
      ),
    ]);
} catch {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  });
}

if (!user.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  });
}

if (user.value.username !== route.params.id) {
  await navigateTo(`/user/${user.value.username}`, { redirectCode: 301 });
}

const title = computed(() => `${user.value.username} - Modrinth`);
const description = computed(() =>
  user.value.bio
    ? formatMessage(messages.profileMetaDescriptionWithBio, {
        bio: user.value.bio,
        username: user.value.username,
      })
    : formatMessage(messages.profileMetaDescription, { username: user.value.username }),
);

useSeoMeta({
  title: () => title.value,
  description: () => description.value,
  ogTitle: () => title.value,
  ogDescription: () => description.value,
  ogImage: () => user.value.avatar_url ?? "https://cdn.modrinth.com/placeholder.png",
});

const projectTypes = computed(() => {
  const obj = {};

  if (collections.value.length > 0) {
    obj.collection = true;
  }

  for (const project of projects.value) {
    obj[project.project_type] = true;
  }

  delete obj.project;

  return Object.keys(obj);
});
const sumDownloads = computed(() => {
  let sum = 0;

  for (const project of projects.value) {
    sum += project.downloads;
  }

  return sum;
});
const sumFollows = computed(() => {
  let sum = 0;

  for (const project of projects.value) {
    sum += project.followers;
  }

  return sum;
});

function cycleSearchDisplayMode() {
  cosmetics.value.searchDisplayMode.user = data.$cycleValue(
    cosmetics.value.searchDisplayMode.user,
    tags.value.projectViewModes,
  );
}
</script>
<script>
export default defineNuxtComponent({
  methods: {},
});
</script>

<style lang="scss" scoped>
.organizations-grid {
  // 5 wide
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-start;

  grid-gap: var(--gap-sm);
  margin-top: 0.5rem;
}

.collections-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);

  @media screen and (max-width: 800px) {
    grid-template-columns: repeat(1, 1fr);
  }

  gap: var(--gap-lg);

  .collection-item {
    display: flex;
    flex-direction: column;
    gap: var(--gap-md);
  }

  .description {
    // Grow to take up remaining space
    flex-grow: 1;

    color: var(--color-text);
    font-size: 16px;
  }

  .stat-bar {
    display: flex;
    align-items: center;
    gap: var(--gap-md);
    margin-top: auto;
  }

  .stats {
    display: flex;
    align-items: center;
    gap: var(--gap-xs);

    svg {
      color: var(--color-secondary);
    }
  }

  .collection {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--gap-md);

    .icon {
      width: 100% !important;
      height: 6rem !important;
      max-width: unset !important;
      max-height: unset !important;
      aspect-ratio: 1 / 1;
      object-fit: cover;
    }

    .details {
      display: flex;
      flex-direction: column;
      gap: var(--gap-sm);

      .title {
        color: var(--color-contrast);
        font-weight: 600;
        font-size: var(--font-size-lg);
        margin: 0;
      }
    }
  }
}

.user-header-wrapper {
  display: flex;
  margin: 0 auto -1.5rem;
  max-width: 80rem;

  .user-header {
    position: relative;
    z-index: 4;
    display: flex;
    width: 100%;
    padding: 0 1rem;
    gap: 1rem;
    align-items: center;

    .username {
      display: none;
      font-size: 2rem;
      margin-bottom: 2.5rem;
    }
  }
}

.mobile-username {
  margin: 0.25rem 0;
}

@media screen and (min-width: 501px) {
  .mobile-username {
    display: none;
  }

  .user-header-wrapper .user-header .username {
    display: block;
  }
}

.sidebar {
  padding-top: 2.5rem;
}

.sidebar__item:not(:last-child) {
  margin: 0 0 0.75rem 0;
}

.profile-picture {
  border-radius: var(--size-rounded-lg);
  height: 8rem;
  width: 8rem;
}

.username {
  font-size: var(--font-size-xl);
}

.bio {
  display: block;
  overflow-wrap: break-word;
}

.secondary-stat {
  align-items: center;
  display: flex;
  margin-bottom: 0.8rem;
}

.secondary-stat__icon {
  height: 1rem;
  width: 1rem;
}

.secondary-stat__text {
  margin-left: 0.4rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.date {
  cursor: default;
}

.inputs {
  margin-bottom: 1rem;

  input {
    margin-top: 0.5rem;
    width: 100%;
  }

  label {
    margin-bottom: 0;
  }
}

.textarea-wrapper {
  height: 10rem;
}

@media (max-width: 400px) {
  .sidebar {
    padding-top: 3rem;
  }
}
</style>
