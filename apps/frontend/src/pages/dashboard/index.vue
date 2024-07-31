<template>
  <div class="dashboard-overview">
    <section class="universal-card dashboard-header">
      <Avatar :src="auth.user.avatar_url" size="md" circle :alt="auth.user.username" />
      <div class="username">
        <h1>
          {{ auth.user.username }}
        </h1>
        <NuxtLink class="goto-link" :to="`/user/${auth.user.username}`">
          Visit your profile
          <ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
        </NuxtLink>
      </div>
    </section>
    <div class="dashboard-notifications">
      <section class="universal-card">
        <div class="header__row">
          <h2 class="header__title text-2xl">Notifications</h2>
          <nuxt-link
            v-if="notifications.length > 0"
            class="goto-link"
            to="/dashboard/notifications"
          >
            See all <ChevronRightIcon />
          </nuxt-link>
        </div>
        <template v-if="notifications.length > 0">
          <NotificationItem
            v-for="notification in notifications"
            :key="notification.id"
            :notifications="notifications"
            class="universal-card recessed"
            :notification="notification"
            :auth="auth"
            raised
            compact
            @update:notifications="() => refresh()"
          />
          <nuxt-link
            v-if="extraNotifs > 0"
            class="goto-link view-more-notifs mt-4"
            to="/dashboard/notifications"
          >
            View {{ extraNotifs }} more notification{{ extraNotifs === 1 ? "" : "s" }}
            <ChevronRightIcon />
          </nuxt-link>
        </template>
        <div v-else class="universal-body">
          <p>You have no unread notifications.</p>
          <nuxt-link class="iconified-button !mt-4" to="/dashboard/notifications/history">
            <HistoryIcon /> View notification history
          </nuxt-link>
        </div>
      </section>
    </div>

    <div class="dashboard-analytics">
      <section class="universal-card">
        <h2>Analytics</h2>
        <div class="grid-display">
          <div class="grid-display__item">
            <div class="label">Total downloads</div>
            <div class="value">
              {{ $formatNumber(projects.reduce((agg, x) => agg + x.downloads, 0)) }}
            </div>
            <span
              >from
              {{ downloadsProjectCount }}
              project{{ downloadsProjectCount === 1 ? "" : "s" }}</span
            >
            <!--          <NuxtLink class="goto-link" to="/dashboard/analytics"-->
            <!--            >View breakdown-->
            <!--            <ChevronRightIcon-->
            <!--              class="featured-header-chevron"-->
            <!--              aria-hidden="true"-->
            <!--          /></NuxtLink>-->
          </div>
          <div class="grid-display__item">
            <div class="label">Total followers</div>
            <div class="value">
              {{ $formatNumber(projects.reduce((agg, x) => agg + x.followers, 0)) }}
            </div>
            <span>
              <span
                >from {{ followersProjectCount }} project{{
                  followersProjectCount === 1 ? "" : "s"
                }}</span
              ></span
            >
          </div>
          <div class="grid-display__item">
            <div class="label">Current balance</div>
            <div class="value">
              {{ $formatMoney(auth.user.payout_data.balance, true) }}
            </div>
            <NuxtLink
              v-if="auth.user.payout_data.balance > 0"
              class="goto-link"
              to="/dashboard/revenue"
            >
              Withdraw earnings
              <ChevronRightIcon class="featured-header-chevron" aria-hidden="true" />
            </NuxtLink>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
<script setup>
import { Avatar } from "@modrinth/ui";
import ChevronRightIcon from "~/assets/images/utils/chevron-right.svg?component";
import HistoryIcon from "~/assets/images/utils/history.svg?component";
import NotificationItem from "~/components/ui/NotificationItem.vue";
import { fetchExtraNotificationData, groupNotifications } from "~/helpers/notifications.js";

useHead({
  title: "Dashboard - Modrinth",
});

const auth = await useAuth();

const [{ data: projects }] = await Promise.all([
  useAsyncData(`user/${auth.value.user.id}/projects`, () =>
    useBaseFetch(`user/${auth.value.user.id}/projects`),
  ),
]);

const downloadsProjectCount = computed(
  () => projects.value.filter((project) => project.downloads > 0).length,
);
const followersProjectCount = computed(
  () => projects.value.filter((project) => project.followers > 0).length,
);

const { data, refresh } = await useAsyncData(async () => {
  const notifications = await useBaseFetch(`user/${auth.value.user.id}/notifications`);

  const filteredNotifications = notifications.filter((notif) => !notif.read);
  const slice = filteredNotifications.slice(0, 30); // send first 30 notifs to be grouped before trimming to 3

  return fetchExtraNotificationData(slice).then((notifications) => {
    notifications = groupNotifications(notifications).slice(0, 3);
    return { notifications, extraNotifs: filteredNotifications.length - slice.length };
  });
});

const notifications = computed(() => {
  if (data.value === null) {
    return [];
  }
  return data.value.notifications;
});

const extraNotifs = computed(() => (data.value ? data.value.extraNotifs : 0));
</script>
<style lang="scss">
.dashboard-overview {
  display: grid;
  grid-template:
    "header header"
    "notifications analytics" / 1fr auto;
  gap: var(--spacing-card-md);

  > .universal-card {
    margin: 0;
  }

  @media screen and (max-width: 750px) {
    display: flex;
    flex-direction: column;
  }
}

.dashboard-notifications {
  grid-area: notifications;
  //display: flex;
  //flex-direction: column;
  //gap: var(--spacing-card-md);

  a.view-more-notifs {
    display: flex;
    width: fit-content;
    margin-left: auto;
  }
}

.dashboard-analytics {
  grid-area: analytics;
}

.dashboard-header {
  display: flex;
  gap: var(--spacing-card-bg);
  grid-area: header;

  .username {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-card-sm);
    justify-content: center;
    word-break: break-word;

    h1 {
      margin: 0;
    }
  }

  @media screen and (max-width: 650px) {
    .avatar {
      width: 4rem;
      height: 4rem;
    }

    .username {
      h1 {
        font-size: var(--font-size-xl);
      }
    }
  }
}
</style>
