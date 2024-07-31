<template>
  <div class="report">
    <div v-if="report.item_type === 'project'" class="item-info">
      <nuxt-link
        :to="`/${$getProjectTypeForUrl(report.project.project_type, report.project.loaders)}/${
          report.project.slug
        }`"
        class="iconified-stacked-link"
      >
        <Avatar :src="report.project.icon_url" size="xs" no-shadow :raised="raised" />
        <div class="stacked">
          <span class="title">{{ report.project.title }}</span>
          <span>{{
            $formatProjectType(
              $getProjectTypeForUrl(report.project.project_type, report.project.loaders),
            )
          }}</span>
        </div>
      </nuxt-link>
    </div>
    <div v-else-if="report.item_type === 'user'" class="item-info">
      <nuxt-link :to="`/user/${report.user.username}`" class="iconified-stacked-link">
        <Avatar :src="report.user.avatar_url" circle size="xs" no-shadow :raised="raised" />
        <div class="stacked">
          <span class="title">{{ report.user.username }}</span>
          <span>User</span>
        </div>
      </nuxt-link>
    </div>
    <div v-else-if="report.item_type === 'version'" class="item-info">
      <nuxt-link
        :to="`/project/${report.project.slug}/version/${report.version.id}`"
        class="iconified-link"
      >
        <div class="backed-svg" :class="{ raised: raised }"><VersionIcon /></div>
        <span class="title">{{ report.version.name }}</span>
      </nuxt-link>
      of
      <nuxt-link :to="`/project/${report.project.slug}`" class="iconified-stacked-link">
        <Avatar :src="report.project.icon_url" size="xs" no-shadow :raised="raised" />
        <div class="stacked">
          <span class="title">{{ report.project.title }}</span>
          <span>{{
            $formatProjectType(
              $getProjectTypeForUrl(report.project.project_type, report.project.loaders),
            )
          }}</span>
        </div>
      </nuxt-link>
    </div>
    <div v-else class="item-info">
      <div class="backed-svg" :class="{ raised: raised }"><UnknownIcon /></div>
      <span>Unknown report type</span>
    </div>
    <div class="report-type">
      <Badge v-if="report.closed" type="closed" />
      <Badge :type="`Reported for ${report.report_type}`" color="orange" />
    </div>
    <div v-if="showMessage" class="markdown-body" v-html="renderHighlightedString(report.body)" />
    <ThreadSummary
      v-if="thread"
      :thread="thread"
      class="thread-summary"
      :raised="raised"
      :link="`/${moderation ? 'moderation' : 'dashboard'}/report/${report.id}`"
      :auth="auth"
    />
    <div class="reporter-info">
      <ReportIcon class="inline-svg" /> Reported by
      <span v-if="auth.user.id === report.reporterUser.id">you</span>
      <nuxt-link v-else :to="`/user/${report.reporterUser.username}`" class="iconified-link">
        <Avatar
          :src="report.reporterUser.avatar_url"
          circle
          size="xxs"
          no-shadow
          :raised="raised"
        />
        <span>{{ report.reporterUser.username }}</span>
      </nuxt-link>
      <span>&nbsp;</span>
      <span v-tooltip="$dayjs(report.created).format('MMMM D, YYYY [at] h:mm A')">{{
        fromNow(report.created)
      }}</span>
      <CopyCode v-if="flags.developerMode" :text="report.id" class="report-id" />
    </div>
  </div>
</template>

<script setup>
import { renderHighlightedString } from "~/helpers/highlight.js";
import { Avatar } from "@modrinth/ui";
import { Badge } from "@modrinth/ui";
import ReportIcon from "~/assets/images/utils/report.svg?component";
import UnknownIcon from "~/assets/images/utils/unknown.svg?component";
import VersionIcon from "~/assets/images/utils/version.svg?component";
import ThreadSummary from "~/components/ui/thread/ThreadSummary.vue";
import { CopyCode } from "@modrinth/ui";

defineProps({
  report: {
    type: Object,
    required: true,
  },
  raised: {
    type: Boolean,
    default: false,
  },
  thread: {
    type: Object,
    default: null,
  },
  showMessage: {
    type: Boolean,
    default: true,
  },
  moderation: {
    type: Boolean,
    default: false,
  },
  auth: {
    type: Object,
    required: true,
  },
});

const flags = useFeatureFlags();
</script>

<style lang="scss" scoped>
.report {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);
  flex-wrap: wrap;

  .report-type {
    grid-area: type;
    display: flex;
    flex-direction: row;
    gap: var(--spacing-card-sm);
    margin-top: var(--spacing-card-xs);
  }

  .item-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-card-xs);
    color: var(--color-heading);
    grid-area: title;

    img,
    .backed-svg {
      margin-right: var(--spacing-card-xs);
    }
  }

  .markdown-body {
    grid-area: body;
  }

  .reporter-info {
    grid-area: reporter;
    gap: var(--spacing-card-xs);
    color: var(--color-text-secondary);

    img {
      vertical-align: middle;
      position: relative;
      top: -1px;
      margin-right: var(--spacing-card-xs);
    }

    a {
      gap: var(--spacing-card-xs);
    }
  }

  .action {
    grid-area: action;
  }

  .thread-summary {
    grid-area: summary;
  }

  &:not(:last-child) {
    margin-bottom: var(--spacing-card-md);
  }

  .report-id {
    margin-left: var(--spacing-card-sm);
  }
}
</style>
