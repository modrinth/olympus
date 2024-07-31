<template>
  <div
    class="message"
    :class="{
      'has-body': message.body.type === 'text' && !forceCompact,
      'no-actions': noLinks,
      private: message.body.private,
    }"
  >
    <template v-if="members[message.author_id]">
      <ConditionalNuxtLink
        class="message__icon"
        :is-link="!noLinks"
        :to="`/user/${members[message.author_id].username}`"
        tabindex="-1"
        aria-hidden="true"
      >
        <Avatar
          class="message__icon"
          :src="members[message.author_id].avatar_url"
          circle
          :raised="raised"
        />
      </ConditionalNuxtLink>
      <span :class="`message__author role-${members[message.author_id].role}`">
        <LockIcon
          v-if="message.body.private"
          v-tooltip="'Only visible to moderators'"
          class="private-icon"
        />
        <ConditionalNuxtLink
          :is-link="!noLinks"
          :to="`/user/${members[message.author_id].username}`"
        >
          {{ members[message.author_id].username }}
        </ConditionalNuxtLink>
        <ScaleIcon v-if="members[message.author_id].role === 'moderator'" v-tooltip="'Moderator'" />
        <ModrinthIcon
          v-else-if="members[message.author_id].role === 'admin'"
          v-tooltip="'Modrinth Team'"
        />
        <MicrophoneIcon
          v-if="report && message.author_id === report.reporterUser.id"
          v-tooltip="'Reporter'"
          class="reporter-icon"
        />
      </span>
    </template>
    <template v-else>
      <div class="message__icon backed-svg circle moderation-color" :class="{ raised: raised }">
        <ScaleIcon />
      </div>
      <span class="message__author moderation-color">
        Moderator
        <ScaleIcon v-tooltip="'Moderator'" />
      </span>
    </template>
    <div
      v-if="message.body.type === 'text'"
      class="message__body markdown-body"
      v-html="formattedMessage"
    />
    <div v-else class="message__body status-message">
      <span v-if="message.body.type === 'deleted'"> posted a message that has been deleted. </span>
      <template v-else-if="message.body.type === 'status_change'">
        <span v-if="message.body.new_status === 'processing'">
          submitted the project for review.
        </span>
        <span v-else>
          changed the project's status from <Badge :type="message.body.old_status" /> to
          <Badge :type="message.body.new_status" />.
        </span>
      </template>
      <span v-else-if="message.body.type === 'thread_closure'">closed the thread.</span>
      <span v-else-if="message.body.type === 'thread_reopen'">reopened the thread.</span>
    </div>
    <span class="message__date">
      <span v-tooltip="$dayjs(message.created).format('MMMM D, YYYY [at] h:mm A')">
        {{ timeSincePosted }}
      </span>
    </span>
    <div v-if="isStaff(auth.user) && message.author_id === auth.user.id" class="message__actions">
      <OverflowMenu
        class="btn btn-transparent icon-only"
        :options="[
          {
            id: 'delete',
            action: () => deleteMessage(),
            color: 'red',
            hoverFilled: true,
          },
        ]"
      >
        <MoreHorizontalIcon />
        <template #delete> <TrashIcon /> Delete </template>
      </OverflowMenu>
    </div>
  </div>
</template>

<script setup>
import {
  MoreHorizontalIcon,
  TrashIcon,
  MicrophoneIcon,
  LockIcon,
  ModrinthIcon,
  ScaleIcon,
} from "@modrinth/assets";
import { OverflowMenu, ConditionalNuxtLink } from "@modrinth/ui";
import { renderString } from "@modrinth/utils";
import { Avatar } from "@modrinth/ui";
import { Badge } from "@modrinth/ui";
import { isStaff } from "~/helpers/users.js";

const props = defineProps({
  message: {
    type: Object,
    required: true,
  },
  report: {
    type: Object,
    default: null,
  },
  members: {
    type: Object,
    default: () => {},
  },
  forceCompact: {
    type: Boolean,
    default: false,
  },
  noLinks: {
    type: Boolean,
    default: false,
  },
  raised: {
    type: Boolean,
    default: false,
  },
  auth: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(["update-thread"]);

const formattedMessage = computed(() => {
  const body = renderString(props.message.body.body);
  if (props.forceCompact) {
    const hasImage = body.includes("<img");
    const noHtml = body.replace(/<\/?[^>]+(>|$)/g, "");
    if (noHtml.trim()) {
      return noHtml;
    } else if (hasImage) {
      return "sent an image.";
    } else {
      return "sent a message.";
    }
  }
  return body;
});

const formatRelativeTime = useRelativeTime();
const timeSincePosted = ref(formatRelativeTime(props.message.created));

async function deleteMessage() {
  await useBaseFetch(`message/${props.message.id}`, {
    method: "DELETE",
  });
  emit("update-thread");
}
</script>

<style lang="scss" scoped>
.message {
  --gap-size: var(--spacing-card-xs);
  display: flex;
  flex-direction: row;
  gap: var(--gap-size);
  flex-wrap: wrap;
  align-items: center;
  border-radius: var(--size-rounded-card);
  padding: var(--spacing-card-md);
  word-break: break-word;

  .avatar,
  .backed-svg {
    --size: 1.5rem;
  }

  &.has-body {
    --gap-size: var(--spacing-card-sm);
    display: grid;
    grid-template:
      "icon author actions"
      "icon body actions"
      "date date date";
    grid-template-columns: min-content auto 1fr;
    column-gap: var(--gap-size);
    row-gap: var(--spacing-card-xs);

    .message__icon {
      margin-bottom: auto;
    }

    .avatar,
    .backed-svg {
      --size: 3rem;
    }
  }

  &:not(.no-actions):hover,
  &:not(.no-actions):focus-within {
    background-color: var(--color-table-alternate-row);

    .message__actions {
      opacity: 1;
    }
  }

  &.no-actions {
    padding: 0;

    .message__actions {
      display: none;
    }
  }
}

.message__icon {
  grid-area: icon;
}

.message__author {
  grid-area: author;
  font-weight: bold;
  display: flex;
  gap: var(--spacing-card-xs);
  flex-wrap: wrap;
  flex-shrink: 0;
}

.message__date {
  grid-area: date;
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
}

.message__actions {
  grid-area: actions;
  margin-left: auto;

  @media (hover: hover) {
    opacity: 0;
  }
}

.message__body {
  grid-area: body;
}

.status-message > span {
  display: flex;
  align-items: center;
  gap: var(--spacing-card-xs);
  flex-wrap: wrap;
}
a {
  display: flex;
  align-items: center;
  text-decoration: none;
}

a:focus-visible + .message__author a,
a:hover + .message__author a,
.message__author a:focus-visible,
.message__author a:hover {
  text-decoration: underline;
  filter: var(--hover-filter);
}

a:active + .message__author a,
.message__author a:active {
  filter: var(--active-filter);
}

.moderation-color,
role-moderator {
  color: var(--color-orange);
}

.role-admin {
  color: var(--color-brand-green);
}

.reporter-icon {
  color: var(--color-purple);
}

.private-icon {
  color: var(--color-gray);
}

@media screen and (min-width: 600px) {
  .message {
    //grid-template:
    //  'icon author body'
    //  'date date date';
    //grid-template-columns: min-content auto 1fr;

    &.has-body {
      grid-template:
        "icon author actions"
        "icon body actions"
        "date date date";
      grid-template-columns: min-content auto 1fr;
    }
  }
}

@media screen and (min-width: 1024px) {
  .message {
    //grid-template: 'icon author body date';
    //grid-template-columns: min-content auto 1fr auto;

    &.has-body {
      grid-template:
        "icon author date actions"
        "icon body body actions";
      grid-template-columns: min-content auto 1fr;
      grid-template-rows: min-content 1fr auto;
    }
  }
}

.private {
  color: var(--color-icon);
}
</style>
