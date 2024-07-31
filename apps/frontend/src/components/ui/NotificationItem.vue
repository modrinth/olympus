<template>
  <div
    class="notification"
    :class="{
      'has-body': hasBody,
      compact: compact,
      read: notification.read,
    }"
  >
    <nuxt-link
      v-if="!type"
      :to="notification.link"
      class="notification__icon backed-svg"
      :class="{ raised: raised }"
    >
      <NotificationIcon />
    </nuxt-link>
    <DoubleIcon v-else class="notification__icon">
      <template #primary>
        <nuxt-link v-if="project" :to="getProjectLink(project)" tabindex="-1">
          <Avatar size="xs" :src="project.icon_url" :raised="raised" no-shadow />
        </nuxt-link>
        <nuxt-link
          v-else-if="organization"
          :to="`/organization/${organization.slug}`"
          tabindex="-1"
        >
          <Avatar size="xs" :src="organization.icon_url" :raised="raised" no-shadow />
        </nuxt-link>
        <nuxt-link v-else-if="user" :to="getUserLink(user)" tabindex="-1">
          <Avatar size="xs" :src="user.avatar_url" :raised="raised" no-shadow />
        </nuxt-link>
        <Avatar v-else size="xs" :raised="raised" no-shadow />
      </template>
      <template #secondary>
        <ModerationIcon
          v-if="type === 'moderator_message' || type === 'status_change'"
          class="moderation-color"
        />
        <InvitationIcon v-else-if="type === 'team_invite' && project" class="creator-color" />
        <InvitationIcon
          v-else-if="type === 'organization_invite' && organization"
          class="creator-color"
        />
        <VersionIcon v-else-if="type === 'project_update' && project && version" />
        <NotificationIcon v-else />
      </template>
    </DoubleIcon>
    <div class="notification__title">
      <template v-if="type === 'project_update' && project && version">
        A project you follow,
        <nuxt-link :to="getProjectLink(project)" class="title-link">{{ project.title }}</nuxt-link
        >, has been updated:
      </template>
      <template v-else-if="type === 'team_invite' && project">
        <nuxt-link
          :to="`/user/${invitedBy.username}`"
          class="iconified-link title-link inline-flex"
        >
          <Avatar
            :src="invitedBy.avatar_url"
            circle
            size="xxs"
            no-shadow
            :raised="raised"
            class="inline-flex"
          />
          <span class="space">&nbsp;</span>
          <span>{{ invitedBy.username }}</span>
        </nuxt-link>
        <span>
          has invited you to join
          <nuxt-link :to="getProjectLink(project)" class="title-link">
            {{ project.title }} </nuxt-link
          >.
        </span>
      </template>
      <template v-else-if="type === 'organization_invite' && organization">
        <nuxt-link
          :to="`/user/${invitedBy.username}`"
          class="iconified-link title-link inline-flex"
        >
          <Avatar
            :src="invitedBy.avatar_url"
            circle
            size="xxs"
            no-shadow
            :raised="raised"
            class="inline-flex"
          />
          <span class="space">&nbsp;</span>
          <span>{{ invitedBy.username }}</span>
        </nuxt-link>
        <span>
          has invited you to join
          <nuxt-link :to="`/organization/${organization.slug}`" class="title-link">
            {{ organization.name }} </nuxt-link
          >.
        </span>
      </template>
      <template v-else-if="type === 'status_change' && project">
        <nuxt-link :to="getProjectLink(project)" class="title-link">
          {{ project.title }}
        </nuxt-link>
        <template v-if="tags.rejectedStatuses.includes(notification.body.new_status)">
          has been <Badge :type="notification.body.new_status" />
        </template>
        <template v-else>
          updated from
          <Badge :type="notification.body.old_status" />
          to
          <Badge :type="notification.body.new_status" />
        </template>
        by the moderators.
      </template>
      <template v-else-if="type === 'moderator_message' && thread && project && !report">
        Your project,
        <nuxt-link :to="getProjectLink(project)" class="title-link">{{ project.title }}</nuxt-link
        >, has received
        <template v-if="notification.grouped_notifs"> messages </template>
        <template v-else>a message</template>
        from the moderators.
      </template>
      <template v-else-if="type === 'moderator_message' && thread && report">
        A moderator replied to your report of
        <template v-if="version">
          version
          <nuxt-link :to="getVersionLink(project, version)" class="title-link">
            {{ version.name }}
          </nuxt-link>
          of project
        </template>
        <nuxt-link v-if="project" :to="getProjectLink(project)" class="title-link">
          {{ project.title }}
        </nuxt-link>
        <nuxt-link v-else-if="user" :to="getUserLink(user)" class="title-link">
          {{ user.username }} </nuxt-link
        >.
      </template>
      <nuxt-link v-else :to="notification.link" class="title-link">
        <span v-html="renderString(notification.title)" />
      </nuxt-link>
      <!--      <span v-else class="known-errors">Error reading notification.</span>-->
    </div>
    <div v-if="hasBody" class="notification__body">
      <ThreadSummary
        v-if="type === 'moderator_message' && thread"
        :thread="thread"
        :link="threadLink"
        :raised="raised"
        :messages="getMessages()"
        class="thread-summary"
        :auth="auth"
      />
      <div v-else-if="type === 'project_update'" class="version-list">
        <div
          v-for="notif in (notification.grouped_notifs
            ? [notification, ...notification.grouped_notifs]
            : [notification]
          ).filter((x) => x.extra_data.version)"
          :key="notif.id"
          class="version-link"
        >
          <VersionIcon />
          <nuxt-link
            :to="getVersionLink(notif.extra_data.project, notif.extra_data.version)"
            class="text-link"
          >
            {{ notif.extra_data.version.name }}
          </nuxt-link>
          <span class="version-info">
            for
            <Categories
              :categories="notif.extra_data.version.loaders"
              :type="notif.extra_data.project.project_type"
              class="categories"
            />
            {{ $formatVersion(notif.extra_data.version.game_versions) }}
            <span
              v-tooltip="
                $dayjs(notif.extra_data.version.date_published).format('MMMM D, YYYY [at] h:mm A')
              "
              class="date"
            >
              {{ fromNow(notif.extra_data.version.date_published) }}
            </span>
          </span>
        </div>
      </div>
      <template v-else>
        {{ notification.text }}
      </template>
    </div>
    <span class="notification__date">
      <span v-if="notification.read" class="read-badge inline-flex"> <ReadIcon /> Read </span>
      <span
        v-tooltip="$dayjs(notification.created).format('MMMM D, YYYY [at] h:mm A')"
        class="inline-flex"
      >
        <CalendarIcon class="mr-1" /> Received {{ fromNow(notification.created) }}
      </span>
    </span>
    <div v-if="compact" class="notification__actions">
      <template v-if="type === 'team_invite' || type === 'organization_invite'">
        <button
          v-tooltip="`Accept`"
          class="iconified-button square-button brand-button button-transparent"
          @click="
            () => {
              acceptTeamInvite(notification.body.team_id);
              read();
            }
          "
        >
          <CheckIcon />
        </button>
        <button
          v-tooltip="`Decline`"
          class="iconified-button square-button danger-button button-transparent"
          @click="
            () => {
              removeSelfFromTeam(notification.body.team_id);
              read();
            }
          "
        >
          <CrossIcon />
        </button>
      </template>
      <button
        v-else-if="!notification.read"
        v-tooltip="`Mark as read`"
        class="iconified-button square-button button-transparent"
        @click="read()"
      >
        <CrossIcon />
      </button>
    </div>
    <div v-else class="notification__actions">
      <div v-if="type !== null" class="input-group">
        <template
          v-if="(type === 'team_invite' || type === 'organization_invite') && !notification.read"
        >
          <button
            class="iconified-button brand-button"
            @click="
              () => {
                acceptTeamInvite(notification.body.team_id);
                read();
              }
            "
          >
            <CheckIcon /> Accept
          </button>
          <button
            class="iconified-button danger-button"
            @click="
              () => {
                removeSelfFromTeam(notification.body.team_id);
                read();
              }
            "
          >
            <CrossIcon /> Decline
          </button>
        </template>
        <button
          v-else-if="!notification.read"
          class="iconified-button"
          :class="{ 'raised-button': raised }"
          @click="read()"
        >
          <CheckIcon /> Mark as read
        </button>
        <CopyCode v-if="flags.developerMode" :text="notification.id" />
      </div>
      <div v-else class="input-group">
        <nuxt-link
          v-if="notification.link && notification.link !== '#'"
          class="iconified-button"
          :class="{ 'raised-button': raised }"
          :to="notification.link"
          target="_blank"
        >
          <ExternalIcon />
          Open link
        </nuxt-link>
        <button
          v-for="(action, actionIndex) in notification.actions"
          :key="actionIndex"
          class="iconified-button"
          :class="{ 'raised-button': raised }"
          @click="performAction(notification, actionIndex)"
        >
          <CheckIcon v-if="action.title === 'Accept'" />
          <CrossIcon v-else-if="action.title === 'Deny'" />
          {{ action.title }}
        </button>
        <button
          v-if="notification.actions.length === 0 && !notification.read"
          class="iconified-button"
          :class="{ 'raised-button': raised }"
          @click="performAction(notification, null)"
        >
          <CheckIcon /> Mark as read
        </button>
        <CopyCode v-if="flags.developerMode" :text="notification.id" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { renderString } from "@modrinth/utils";
import InvitationIcon from "~/assets/images/utils/user-plus.svg?component";
import ModerationIcon from "~/assets/images/sidebar/admin.svg?component";
import NotificationIcon from "~/assets/images/sidebar/notifications.svg?component";
import ReadIcon from "~/assets/images/utils/check-circle.svg?component";
import CalendarIcon from "~/assets/images/utils/calendar.svg?component";
import VersionIcon from "~/assets/images/utils/version.svg?component";
import CheckIcon from "~/assets/images/utils/check.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import ExternalIcon from "~/assets/images/utils/external.svg?component";
import ThreadSummary from "~/components/ui/thread/ThreadSummary.vue";
import { getProjectLink, getVersionLink } from "~/helpers/projects.js";
import { getUserLink } from "~/helpers/users.js";
import { acceptTeamInvite, removeSelfFromTeam } from "~/helpers/teams.js";
import { markAsRead } from "~/helpers/notifications.js";
import { DoubleIcon } from "@modrinth/ui";
import { Avatar } from "@modrinth/ui";
import { Badge } from "@modrinth/ui";
import { CopyCode } from "@modrinth/ui";
import Categories from "~/components/ui/search/Categories.vue";

const app = useNuxtApp();
const emit = defineEmits(["update:notifications"]);

const props = defineProps({
  notification: {
    type: Object,
    required: true,
  },
  notifications: {
    type: Array,
    required: true,
  },
  raised: {
    type: Boolean,
    default: false,
  },
  compact: {
    type: Boolean,
    default: false,
  },
  auth: {
    type: Object,
    required: true,
  },
});

const flags = useFeatureFlags();
const tags = useTags();

const type = computed(() =>
  !props.notification.body || props.notification.body.type === "legacy_markdown"
    ? null
    : props.notification.body.type,
);
const thread = computed(() => props.notification.extra_data.thread);
const report = computed(() => props.notification.extra_data.report);
const project = computed(() => props.notification.extra_data.project);
const version = computed(() => props.notification.extra_data.version);
const user = computed(() => props.notification.extra_data.user);
const organization = computed(() => props.notification.extra_data.organization);
const invitedBy = computed(() => props.notification.extra_data.invited_by);

const threadLink = computed(() => {
  if (report.value) {
    return `/dashboard/report/${report.value.id}`;
  } else if (project.value) {
    return `${getProjectLink(project.value)}/moderation#messages`;
  }
  return "#";
});

const hasBody = computed(() => !type.value || thread.value || type.value === "project_update");

async function read() {
  try {
    const ids = [
      props.notification.id,
      ...(props.notification.grouped_notifs
        ? props.notification.grouped_notifs.map((notif) => notif.id)
        : []),
    ];
    const updateNotifs = await markAsRead(ids);
    const newNotifs = updateNotifs(props.notifications);
    emit("update:notifications", newNotifs);
  } catch (err) {
    app.$notify({
      group: "main",
      title: "Error marking notification as read",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

async function performAction(notification, actionIndex) {
  startLoading();
  try {
    await read();

    if (actionIndex !== null) {
      await useBaseFetch(`${notification.actions[actionIndex].action_route[1]}`, {
        method: notification.actions[actionIndex].action_route[0].toUpperCase(),
      });
    }
  } catch (err) {
    app.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });
  }
  stopLoading();
}

function getMessages() {
  const messages = [];
  if (props.notification.body.message_id) {
    messages.push(props.notification.body.message_id);
  }
  if (props.notification.grouped_notifs) {
    for (const notif of props.notification.grouped_notifs) {
      if (notif.body.message_id) {
        messages.push(notif.body.message_id);
      }
    }
  }
  return messages;
}
</script>

<style lang="scss" scoped>
.notification {
  display: grid;
  grid-template:
    "icon title"
    "actions actions"
    "date date";
  grid-template-columns: min-content 1fr;
  grid-template-rows: min-content min-content min-content;
  gap: var(--spacing-card-sm);

  &.compact {
    grid-template:
      "icon title actions"
      "date date date";
    grid-template-columns: min-content 1fr auto;
    grid-template-rows: auto min-content;
  }

  &.has-body {
    grid-template:
      "icon title"
      "body body"
      "actions actions"
      "date date";
    grid-template-columns: min-content 1fr;
    grid-template-rows: min-content auto auto min-content;

    &.compact {
      grid-template:
        "icon title actions"
        "body body body"
        "date date date";
      grid-template-columns: min-content 1fr auto;
      grid-template-rows: min-content auto min-content;
    }
  }

  .label__title,
  .label__description,
  h1,
  h2,
  h3,
  h4,
  :deep(p) {
    margin: 0 !important;
  }

  .notification__icon {
    grid-area: icon;
  }

  .notification__title {
    grid-area: title;
    color: var(--color-heading);
    margin-block: auto;
    display: inline-block;
    vertical-align: middle;
    line-height: 1.25rem;

    .iconified-link {
      display: inline;

      img {
        vertical-align: middle;
        position: relative;
        top: -2px;
      }
    }
  }
  .notification__body {
    grid-area: body;

    .version-list {
      margin: 0;
      padding: 0;
      list-style-type: none;
      display: flex;
      flex-direction: column;
      flex-wrap: wrap;
      gap: var(--spacing-card-sm);
      grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));

      .version-link {
        display: flex;
        flex-direction: row;
        gap: var(--spacing-card-xs);
        align-items: center;
        flex-wrap: wrap;

        .version-info {
          display: contents;

          :deep(span) {
            color: var(--color-text);
          }

          .date {
            color: var(--color-text-secondary);
            font-size: var(--font-size-sm);
          }
        }
      }
    }
  }

  .notification__date {
    grid-area: date;
    color: var(--color-text-secondary);

    svg {
      vertical-align: top;
    }

    .read-badge {
      font-weight: bold;
      color: var(--color-text);
      margin-right: var(--spacing-card-xs);
    }
  }

  .notification__actions {
    grid-area: actions;
    display: flex;
    flex-direction: row;
    gap: var(--spacing-card-sm);
  }

  .unknown-type {
    color: var(--color-red);
  }

  .title-link {
    &:not(:hover) {
      text-decoration: none;
    }
    font-weight: bold;
  }

  .moderation-color {
    color: var(--color-orange);
  }

  .creator-color {
    color: var(--color-blue);
  }
}
</style>
