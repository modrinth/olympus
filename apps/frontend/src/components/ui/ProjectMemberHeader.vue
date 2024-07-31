<template>
  <div v-if="showInvitation" class="universal-card information invited">
    <h2>Invitation to join project</h2>
    <p>
      You've been invited be a member of this project with the role of '{{ currentMember.role }}'.
    </p>
    <div class="input-group">
      <button class="iconified-button brand-button" @click="acceptInvite()">
        <CheckIcon />Accept
      </button>
      <button class="iconified-button danger-button" @click="declineInvite()">
        <CrossIcon />Decline
      </button>
    </div>
  </div>
  <div
    v-if="
      currentMember &&
      nags.filter((x) => x.condition).length > 0 &&
      (project.status === 'draft' || tags.rejectedStatuses.includes(project.status))
    "
    class="author-actions universal-card"
  >
    <div class="header__row">
      <div class="header__title">
        <h2>Publishing checklist</h2>
        <div class="checklist">
          <span class="checklist__title">Progress:</span>
          <div class="checklist__items">
            <div
              v-for="nag in nags"
              :key="`checklist-${nag.id}`"
              v-tooltip="nag.title"
              :aria-label="nag.title"
              class="circle"
              :class="'circle ' + (!nag.condition ? 'done' : '') + nag.status"
            >
              <CheckIcon v-if="!nag.condition" />
              <RequiredIcon v-else-if="nag.status === 'required'" />
              <SuggestionIcon v-else-if="nag.status === 'suggestion'" />
              <ModerationIcon v-else-if="nag.status === 'review'" />
            </div>
          </div>
        </div>
      </div>
      <div class="input-group">
        <button
          class="square-button"
          :class="{ 'not-collapsed': !collapsed }"
          @click="toggleCollapsed()"
        >
          <DropdownIcon />
        </button>
      </div>
    </div>
    <div v-if="!collapsed" class="grid-display width-16">
      <div
        v-for="nag in nags.filter((x) => x.condition && !x.hide)"
        :key="nag.id"
        class="grid-display__item"
      >
        <span class="label">
          <RequiredIcon
            v-if="nag.status === 'required'"
            v-tooltip="'Required'"
            aria-label="Required"
            :class="nag.status"
          />
          <SuggestionIcon
            v-else-if="nag.status === 'suggestion'"
            v-tooltip="'Suggestion'"
            aria-label="Suggestion"
            :class="nag.status"
          />
          <ModerationIcon
            v-else-if="nag.status === 'review'"
            v-tooltip="'Review'"
            aria-label="Review"
            :class="nag.status"
          />{{ nag.title }}</span
        >
        {{ nag.description }}
        <NuxtLink
          v-if="nag.link"
          :class="{ invisible: nag.link.hide }"
          class="goto-link"
          :to="`/${project.project_type}/${project.slug ? project.slug : project.id}/${
            nag.link.path
          }`"
        >
          {{ nag.link.title }}
          <ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
        </NuxtLink>
        <button
          v-else-if="nag.action"
          class="iconified-button moderation-button"
          :disabled="nag.action.disabled()"
          @click="nag.action.onClick"
        >
          <SendIcon />
          {{ nag.action.title }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { formatProjectType } from "~/plugins/shorthands.js";

import ChevronRightIcon from "~/assets/images/utils/chevron-right.svg?component";
import DropdownIcon from "~/assets/images/utils/dropdown.svg?component";
import CheckIcon from "~/assets/images/utils/check.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import RequiredIcon from "~/assets/images/utils/asterisk.svg?component";
import SuggestionIcon from "~/assets/images/utils/lightbulb.svg?component";
import ModerationIcon from "~/assets/images/sidebar/admin.svg?component";
import SendIcon from "~/assets/images/utils/send.svg?component";
import { acceptTeamInvite, removeTeamMember } from "~/helpers/teams.js";

const props = defineProps({
  project: {
    type: Object,
    required: true,
  },
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
  currentMember: {
    type: Object,
    default: null,
  },
  allMembers: {
    type: Object,
    default: null,
  },
  isSettings: {
    type: Boolean,
    default: false,
  },
  collapsed: {
    type: Boolean,
    default: false,
  },
  routeName: {
    type: String,
    default: "",
  },
  auth: {
    type: Object,
    required: true,
  },
  tags: {
    type: Object,
    required: true,
  },
  setProcessing: {
    type: Function,
    default() {
      return () => {
        addNotification({
          group: "main",
          title: "An error occurred",
          text: "setProcessing function not found",
          type: "error",
        });
      };
    },
  },
  toggleCollapsed: {
    type: Function,
    default() {
      return () => {
        addNotification({
          group: "main",
          title: "An error occurred",
          text: "toggleCollapsed function not found",
          type: "error",
        });
      };
    },
  },
  updateMembers: {
    type: Function,
    default() {
      return () => {
        addNotification({
          group: "main",
          title: "An error occurred",
          text: "updateMembers function not found",
          type: "error",
        });
      };
    },
  },
});

const featuredGalleryImage = computed(() => props.project.gallery.find((img) => img.featured));

const nags = computed(() => [
  {
    condition: props.versions.length < 1,
    title: "Upload a version",
    id: "upload-version",
    description: "At least one version is required for a project to be submitted for review.",
    status: "required",
    link: {
      path: "versions",
      title: "Visit versions page",
      hide: props.routeName === "type-id-versions",
    },
  },
  {
    condition:
      props.project.body === "" || props.project.body.startsWith("# Placeholder description"),
    title: "Add a description",
    id: "add-description",
    description:
      "A description that clearly describes the project's purpose and function is required.",
    status: "required",
    link: {
      path: "settings/description",
      title: "Visit description settings",
      hide: props.routeName === "type-id-settings-description",
    },
  },
  {
    condition: !props.project.icon_url,
    title: "Add an icon",
    id: "add-icon",
    description:
      "Your project should have a nice-looking icon to uniquely identify your project at a glance.",
    status: "suggestion",
    link: {
      path: "settings",
      title: "Visit general settings",
      hide: props.routeName === "type-id-settings",
    },
  },
  {
    condition: props.project.gallery.length === 0 || !featuredGalleryImage.value,
    title: "Feature a gallery image",
    id: "feature-gallery-image",
    description: "Featured gallery images may be the first impression of many users.",
    status: "suggestion",
    link: {
      path: "gallery",
      title: "Visit gallery page",
      hide: props.routeName === "type-id-gallery",
    },
  },
  {
    hide: props.project.versions.length === 0,
    condition: props.project.categories.length < 1,
    title: "Select tags",
    id: "select-tags",
    description: "Select all tags that apply to your project.",
    status: "suggestion",
    link: {
      path: "settings/tags",
      title: "Visit tag settings",
      hide: props.routeName === "type-id-settings-tags",
    },
  },
  {
    condition: !(
      props.project.issues_url ||
      props.project.source_url ||
      props.project.wiki_url ||
      props.project.discord_url ||
      props.project.donation_urls.length > 0
    ),
    title: "Add external links",
    id: "add-links",
    description:
      "Add any relevant links targeted outside of Modrinth, such as sources, issues, or a Discord invite.",
    status: "suggestion",
    link: {
      path: "settings/links",
      title: "Visit links settings",
      hide: props.routeName === "type-id-settings-links",
    },
  },
  {
    hide:
      props.project.versions.length === 0 ||
      props.project.project_type === "resourcepack" ||
      props.project.project_type === "plugin" ||
      props.project.project_type === "shader" ||
      props.project.project_type === "datapack",
    condition:
      props.project.client_side === "unknown" ||
      props.project.server_side === "unknown" ||
      (props.project.client_side === "unsupported" && props.project.server_side === "unsupported"),
    title: "Select supported environments",
    id: "select-environments",
    description: `Select if the ${formatProjectType(
      props.project.project_type,
    ).toLowerCase()} functions on the client-side and/or server-side.`,
    status: "required",
    link: {
      path: "settings",
      title: "Visit general settings",
      hide: props.routeName === "type-id-settings",
    },
  },
  {
    condition: props.project.license.id === "LicenseRef-Unknown",
    title: "Select license",
    id: "select-license",
    description: `Select the license your ${formatProjectType(
      props.project.project_type,
    ).toLowerCase()} is distributed under.`,
    status: "required",
    link: {
      path: "settings/license",
      title: "Visit license settings",
      hide: props.routeName === "type-id-settings-license",
    },
  },
  {
    condition: props.project.status === "draft",
    title: "Submit for review",
    id: "submit-for-review",
    description:
      "Your project is only viewable by members of the project. It must be reviewed by moderators in order to be published.",
    status: "review",
    link: null,
    action: {
      onClick: submitForReview,
      title: "Submit for review",
      disabled: () => nags.value.filter((x) => x.condition && x.status === "required").length > 0,
    },
  },
  {
    condition: props.tags.rejectedStatuses.includes(props.project.status),
    title: "Resubmit for review",
    id: "resubmit-for-review",
    description: `Your project has been ${props.project.status} by
            Modrinth's staff. In most cases, you can resubmit for review after
            addressing the staff's message.`,
    status: "review",
    link: {
      path: "moderation",
      title: "Visit moderation page",
      hide: props.routeName === "type-id-moderation",
    },
  },
]);

const showInvitation = computed(() => {
  if (props.allMembers && props.auth) {
    const member = props.allMembers.find((x) => x.user.id === props.auth.user.id);
    return member && !member.accepted;
  }
  return false;
});

const acceptInvite = () => {
  acceptTeamInvite(props.project.team);
  props.updateMembers();
};

const declineInvite = () => {
  removeTeamMember(props.project.team, props.auth.user.id);
  props.updateMembers();
};

const submitForReview = async () => {
  if (
    !props.acknowledgedMessage ||
    nags.value.filter((x) => x.condition && x.status === "required").length === 0
  ) {
    await props.setProcessing();
  }
};
</script>

<style lang="scss" scoped>
.invited {
}

.author-actions {
  &:empty {
    display: none;
  }

  .invisible {
    visibility: hidden;
  }

  .header__row {
    align-items: center;
    column-gap: var(--spacing-card-lg);
    row-gap: var(--spacing-card-md);
    max-width: 100%;

    .header__title {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      column-gap: var(--spacing-card-lg);
      row-gap: var(--spacing-card-md);
      flex-basis: min-content;

      h2 {
        margin: 0 auto 0 0;
      }
    }

    button {
      svg {
        transition: transform 0.25s ease-in-out;
      }

      &.not-collapsed svg {
        transform: rotate(180deg);
      }
    }
  }

  .grid-display__item .label {
    display: flex;
    gap: var(--spacing-card-xs);
    align-items: center;

    .required {
      color: var(--color-red);
    }

    .suggestion {
      color: var(--color-purple);
    }

    .review {
      color: var(--color-orange);
    }
  }

  .checklist {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: var(--spacing-card-xs);
    width: fit-content;
    flex-wrap: wrap;
    max-width: 100%;

    .checklist__title {
      font-weight: bold;
      margin-right: var(--spacing-card-xs);
      color: var(--color-text-dark);
    }

    .checklist__items {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: var(--spacing-card-xs);
      width: fit-content;
      max-width: 100%;
    }

    .circle {
      --circle-size: 2rem;
      --background-color: var(--color-bg);
      --content-color: var(--color-gray);
      width: var(--circle-size);
      height: var(--circle-size);
      border-radius: 50%;
      background-color: var(--background-color);
      display: flex;
      justify-content: center;
      align-items: center;

      svg {
        color: var(--content-color);
        width: calc(var(--circle-size) / 2);
        height: calc(var(--circle-size) / 2);
      }

      &.required {
        --content-color: var(--color-red);
      }

      &.suggestion {
        --content-color: var(--color-purple);
      }

      &.review {
        --content-color: var(--color-orange);
      }

      &.done {
        --background-color: var(--color-green);
        --content-color: var(--color-brand-inverted);
      }
    }
  }
}
</style>
