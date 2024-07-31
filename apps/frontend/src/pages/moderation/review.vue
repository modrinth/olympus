<template>
  <section class="universal-card">
    <h2>Review projects</h2>
    <div class="input-group">
      <Chips
        v-model="projectType"
        :items="projectTypes"
        :format-label="(x) => (x === 'all' ? 'All' : $formatProjectType(x) + 's')"
      />
      <button v-if="oldestFirst" class="iconified-button push-right" @click="oldestFirst = false">
        <SortDescIcon />Sorting by oldest
      </button>
      <button v-else class="iconified-button push-right" @click="oldestFirst = true">
        <SortAscIcon />Sorting by newest
      </button>
      <button
        class="btn btn-highlight"
        :disabled="projectsFiltered.length === 0"
        @click="goToProjects()"
      >
        <ModerationIcon /> Start moderating
      </button>
    </div>
    <p v-if="projectType !== 'all'" class="project-count">
      Showing {{ projectsFiltered.length }} {{ projectTypePlural }} of {{ projects.length }} total
      projects in the queue.
    </p>
    <p v-else class="project-count">There are {{ projects.length }} projects in the queue.</p>
    <p v-if="projectsOver24Hours.length > 0" class="warning project-count">
      <WarningIcon /> {{ projectsOver24Hours.length }} {{ projectTypePlural }}
      have been in the queue for over 24 hours.
    </p>
    <p v-if="projectsOver48Hours.length > 0" class="danger project-count">
      <WarningIcon /> {{ projectsOver48Hours.length }} {{ projectTypePlural }}
      have been in the queue for over 48 hours.
    </p>
    <div
      v-for="project in projectsFiltered.sort((a, b) => {
        if (oldestFirst) {
          return b.age - a.age;
        } else {
          return a.age - b.age;
        }
      })"
      :key="`project-${project.id}`"
      class="universal-card recessed project"
    >
      <div class="project-title">
        <div class="mobile-row">
          <nuxt-link
            :to="`/${project.inferred_project_type}/${project.slug}`"
            class="iconified-stacked-link"
          >
            <Avatar :src="project.icon_url" size="xs" no-shadow raised />
            <span class="stacked">
              <span class="title">{{ project.name }}</span>
              <span>{{ $formatProjectType(project.inferred_project_type) }}</span>
            </span>
          </nuxt-link>
        </div>
        <div class="mobile-row">
          by
          <nuxt-link
            v-if="project.owner"
            :to="`/user/${project.owner.user.username}`"
            class="iconified-link"
          >
            <Avatar :src="project.owner.user.avatar_url" circle size="xxs" raised />
            <span>{{ project.owner.user.username }}</span>
          </nuxt-link>
          <nuxt-link
            v-else-if="project.org"
            :to="`/organization/${project.org.slug}`"
            class="iconified-link"
          >
            <Avatar :src="project.org.icon_url" circle size="xxs" raised />
            <span>{{ project.org.name }}</span>
          </nuxt-link>
        </div>
        <div class="mobile-row">
          is requesting to be
          <Badge :type="project.requested_status ? project.requested_status : 'approved'" />
        </div>
      </div>
      <div class="input-group">
        <nuxt-link
          :to="`/${project.inferred_project_type}/${project.slug}`"
          class="iconified-button raised-button"
          ><EyeIcon /> View project</nuxt-link
        >
      </div>
      <span v-if="project.queued" :class="`submitter-info ${project.age_warning}`">
        <WarningIcon v-if="project.age_warning" />
        Submitted
        <span v-tooltip="$dayjs(project.queued).format('MMMM D, YYYY [at] h:mm A')">{{
          fromNow(project.queued)
        }}</span>
      </span>
      <span v-else class="submitter-info"><UnknownIcon /> Unknown queue date</span>
    </div>
  </section>
</template>
<script setup>
import { Chips, Avatar, Badge } from "@modrinth/ui";
import UnknownIcon from "~/assets/images/utils/unknown.svg?component";
import EyeIcon from "~/assets/images/utils/eye.svg?component";
import SortAscIcon from "~/assets/images/utils/sort-asc.svg?component";
import SortDescIcon from "~/assets/images/utils/sort-desc.svg?component";
import WarningIcon from "~/assets/images/utils/issues.svg?component";
import ModerationIcon from "~/assets/images/sidebar/admin.svg?component";
import { formatProjectType } from "~/plugins/shorthands.js";

useHead({
  title: "Review projects - Modrinth",
});

const app = useNuxtApp();

const router = useRouter();

const now = app.$dayjs();
const TIME_24H = 86400000;
const TIME_48H = TIME_24H * 2;

const { data: projects } = await useAsyncData("moderation/projects?count=1000", () =>
  useBaseFetch("moderation/projects?count=1000", { internal: true }),
);
const members = ref([]);
const projectType = ref("all");
const oldestFirst = ref(true);

const projectsFiltered = computed(() =>
  projects.value.filter(
    (x) =>
      projectType.value === "all" ||
      app.$getProjectTypeForUrl(x.project_types[0], x.loaders) === projectType.value,
  ),
);

const projectsOver24Hours = computed(() =>
  projectsFiltered.value.filter((project) => project.age >= TIME_24H && project.age < TIME_48H),
);
const projectsOver48Hours = computed(() =>
  projectsFiltered.value.filter((project) => project.age >= TIME_48H),
);
const projectTypePlural = computed(() =>
  projectType.value === "all"
    ? "projects"
    : (formatProjectType(projectType.value) + "s").toLowerCase(),
);

const projectTypes = computed(() => {
  const set = new Set();
  set.add("all");

  if (projects.value) {
    for (const project of projects.value) {
      set.add(project.inferred_project_type);
    }
  }

  return [...set];
});

if (projects.value) {
  const teamIds = projects.value.map((x) => x.team_id);
  const organizationIds = projects.value.filter((x) => x.organization).map((x) => x.organization);

  const url = `teams?ids=${encodeURIComponent(JSON.stringify(teamIds))}`;
  const orgUrl = `organizations?ids=${encodeURIComponent(JSON.stringify(organizationIds))}`;
  const { data: result } = await useAsyncData(url, () => useBaseFetch(url));
  const { data: orgs } = await useAsyncData(orgUrl, () => useBaseFetch(orgUrl, { apiVersion: 3 }));

  if (result.value) {
    members.value = result.value;

    projects.value = projects.value.map((project) => {
      project.owner = members.value
        .flat()
        .find((x) => x.team_id === project.team_id && x.role === "Owner");
      project.org = orgs.value.find((x) => x.id === project.organization);
      project.age = project.queued ? now - app.$dayjs(project.queued) : Number.MAX_VALUE;
      project.age_warning = "";
      if (project.age > TIME_24H * 2) {
        project.age_warning = "danger";
      } else if (project.age > TIME_24H) {
        project.age_warning = "warning";
      }
      project.inferred_project_type = app.$getProjectTypeForUrl(
        project.project_types[0],
        project.loaders,
      );
      return project;
    });
  }
}
async function goToProjects() {
  const project = projectsFiltered.value[0];
  await router.push({
    name: "type-id",
    params: {
      type: project.project_types[0],
      id: project.slug ? project.slug : project.id,
    },
    state: {
      showChecklist: true,
      projects: projectsFiltered.value.slice(1).map((x) => (x.slug ? x.slug : x.id)),
    },
  });
}
</script>
<style lang="scss" scoped>
.project {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);
  @media screen and (min-width: 650px) {
    display: grid;
    grid-template: "title action" "date action";
    grid-template-columns: 1fr auto;
  }
}

.submitter-info {
  margin: 0;
  grid-area: date;

  svg {
    vertical-align: top;
  }
}

.warning {
  color: var(--color-orange);
}

.danger {
  color: var(--color-red);
  font-weight: bold;
}

.project-count {
  margin-block: var(--spacing-card-md);

  svg {
    vertical-align: top;
  }
}

.input-group {
  grid-area: action;
}

.project-title {
  display: flex;
  gap: var(--spacing-card-xs);
  align-items: center;
  flex-wrap: wrap;

  .mobile-row {
    display: contents;
  }

  @media screen and (max-width: 800px) {
    flex-direction: column;
    align-items: flex-start;

    .mobile-row {
      display: flex;
      flex-direction: row;
      gap: var(--spacing-card-xs);
      align-items: center;
      flex-wrap: wrap;
    }
  }
}

:deep(.avatar) {
  flex-shrink: 0;

  &.size-xs {
    margin-right: var(--spacing-card-xs);
  }
}
</style>
