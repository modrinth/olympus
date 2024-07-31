<template>
  <div ref="main_page" class="layout" :class="{ 'expanded-mobile-nav': isBrowseMenuOpen }">
    <div
      v-if="auth.user && !auth.user.email_verified && route.path !== '/auth/verify-email'"
      class="email-nag"
    >
      <template v-if="auth.user.email">
        <span>{{ formatMessage(verifyEmailBannerMessages.title) }}</span>
        <button class="btn" @click="resendVerifyEmail">
          {{ formatMessage(verifyEmailBannerMessages.action) }}
        </button>
      </template>
      <template v-else>
        <span>{{ formatMessage(addEmailBannerMessages.title) }}</span>
        <nuxt-link class="btn" to="/settings/account">
          <SettingsIcon />
          {{ formatMessage(addEmailBannerMessages.action) }}
        </nuxt-link>
      </template>
    </div>
    <div
      v-if="
        config.public.apiBaseUrl.startsWith('https://staging-api.modrinth.com') &&
        !cosmetics.hideStagingBanner
      "
      class="site-banner site-banner--warning"
    >
      <div class="site-banner__title">
        <IssuesIcon />
        <span>{{ formatMessage(stagingBannerMessages.title) }}</span>
      </div>
      <div class="site-banner__description">
        {{ formatMessage(stagingBannerMessages.description) }}
      </div>
      <div class="site-banner__actions">
        <Button transparent icon-only :action="hideStagingBanner"><XIcon /></Button>
      </div>
    </div>
    <header class="site-header" role="presentation">
      <section class="navbar columns" role="navigation">
        <section class="logo column" role="presentation">
          <NuxtLink class="button-base" to="/" aria-label="Modrinth home page">
            <BrandTextLogo aria-hidden="true" class="text-logo" />
          </NuxtLink>
        </section>
        <section class="nav-group columns" role="presentation">
          <section class="nav" aria-label="Page links">
            <NavRow class="navigation" :links="navRoutes" />
          </section>
          <section class="column-grow user-outer" aria-label="Account links">
            <section class="user-controls">
              <nuxt-link
                v-if="auth.user"
                to="/dashboard/notifications"
                class="control-button button-transparent"
                :title="formatMessage(commonMessages.notificationsLabel)"
              >
                <NotificationIcon aria-hidden="true" />
              </nuxt-link>
              <button
                class="control-button button-transparent"
                :title="formatMessage(messages.changeTheme)"
                @click="changeTheme"
              >
                <MoonIcon v-if="$theme.active === 'light'" aria-hidden="true" />
                <SunIcon v-else aria-hidden="true" />
              </button>
              <div
                v-if="auth.user"
                class="dropdown"
                :class="{ closed: !isDropdownOpen }"
                tabindex="0"
                @mouseover="isDropdownOpen = true"
                @focus="isDropdownOpen = true"
                @mouseleave="isDropdownOpen = false"
              >
                <button class="control" value="Profile Dropdown">
                  <Avatar
                    :src="auth.user.avatar_url"
                    class="user-icon"
                    :alt="formatMessage(messages.yourAvatarAlt)"
                    aria-hidden="true"
                    circle
                  />
                  <DropdownIcon class="caret" />
                </button>
                <div class="content card">
                  <NuxtLink class="item button-transparent" :to="`/user/${auth.user.username}`">
                    <div class="title profile-link">
                      <div class="username">@{{ auth.user.username }}</div>
                      <div class="prompt">{{ formatMessage(commonMessages.visitYourProfile) }}</div>
                    </div>
                  </NuxtLink>
                  <hr class="divider" />
                  <button class="item button-transparent" @click="$refs.modal_creation.show()">
                    <PlusIcon class="icon" />
                    <span class="title">
                      {{ formatMessage(commonMessages.createAProjectButton) }}
                    </span>
                  </button>
                  <hr class="divider" />
                  <NuxtLink class="item button-transparent" to="/dashboard/collections">
                    <LibraryIcon class="icon" />
                    <span class="title">{{ formatMessage(commonMessages.collectionsLabel) }}</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/dashboard/notifications">
                    <NotificationIcon class="icon" />
                    <span class="title">{{
                      formatMessage(commonMessages.notificationsLabel)
                    }}</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/dashboard">
                    <ChartIcon class="icon" />
                    <span class="title">{{ formatMessage(commonMessages.dashboardLabel) }}</span>
                  </NuxtLink>
                  <NuxtLink class="item button-transparent" to="/settings">
                    <SettingsIcon class="icon" />
                    <span class="title">{{ formatMessage(commonMessages.settingsLabel) }}</span>
                  </NuxtLink>
                  <NuxtLink
                    v-if="tags.staffRoles.includes(auth.user.role)"
                    class="item button-transparent"
                    to="/moderation"
                  >
                    <ModerationIcon class="icon" />
                    <span class="title">{{ formatMessage(commonMessages.moderationLabel) }}</span>
                  </NuxtLink>
                  <NuxtLink v-if="flags.developerMode" class="item button-transparent" to="/flags">
                    <ReportIcon class="icon" />
                    <span class="title">Feature flags</span>
                  </NuxtLink>
                  <NuxtLink
                    v-if="!cosmetics.hideModrinthAppPromos"
                    class="item button-transparent primary-color"
                    to="/app"
                  >
                    <DownloadIcon class="icon" />
                    <span class="title">
                      {{ formatMessage(messages.getModrinthApp) }}
                    </span>
                  </NuxtLink>
                  <hr class="divider" />
                  <button class="item button-transparent" @click="logoutUser()">
                    <LogOutIcon class="icon" />
                    <span class="dropdown-item__text">
                      {{ formatMessage(commonMessages.signOutButton) }}
                    </span>
                  </button>
                </div>
              </div>
              <section v-else class="auth-prompt">
                <nuxt-link class="iconified-button raised-button" to="/auth/sign-in">
                  <LogInIcon /> {{ formatMessage(commonMessages.signInButton) }}
                </nuxt-link>
                <nuxt-link
                  v-if="$route.path !== '/app' && !cosmetics.hideModrinthAppPromos"
                  class="btn btn-outline btn-primary app-btn"
                  to="/app"
                >
                  <DownloadIcon /> {{ formatMessage(messages.getModrinthApp) }}
                </nuxt-link>
              </section>
            </section>
          </section>
        </section>
      </section>
      <section class="mobile-navigation">
        <div
          class="nav-menu nav-menu-browse"
          :class="{ expanded: isBrowseMenuOpen }"
          @focusin="isBrowseMenuOpen = true"
          @focusout="isBrowseMenuOpen = false"
        >
          <div class="links cascade-links">
            <NuxtLink
              v-for="navRoute in navRoutes"
              :key="navRoute.href"
              :to="navRoute.href"
              class="iconified-button"
            >
              {{ navRoute.label }}
            </NuxtLink>
          </div>
        </div>
        <div
          class="nav-menu nav-menu-mobile"
          :class="{ expanded: isMobileMenuOpen }"
          @focusin="isMobileMenuOpen = true"
          @focusout="isMobileMenuOpen = false"
        >
          <div class="account-container">
            <NuxtLink
              v-if="auth.user"
              :to="`/user/${auth.user.username}`"
              class="iconified-button account-button"
            >
              <Avatar
                :src="auth.user.avatar_url"
                class="user-icon"
                :alt="formatMessage(messages.yourAvatarAlt)"
                aria-hidden="true"
                circle
              />
              <div class="account-text">
                <div>@{{ auth.user.username }}</div>
                <div>{{ formatMessage(commonMessages.visitYourProfile) }}</div>
              </div>
            </NuxtLink>
            <nuxt-link v-else class="iconified-button brand-button" to="/auth/sign-in">
              <LogInIcon /> {{ formatMessage(commonMessages.signInButton) }}
            </nuxt-link>
          </div>
          <div class="links">
            <template v-if="auth.user">
              <button class="iconified-button danger-button" @click="logoutUser()">
                <LogOutIcon aria-hidden="true" />
                {{ formatMessage(commonMessages.signOutButton) }}
              </button>
              <button class="iconified-button" @click="$refs.modal_creation.show()">
                <PlusIcon aria-hidden="true" />
                {{ formatMessage(commonMessages.createAProjectButton) }}
              </button>
              <NuxtLink class="iconified-button" to="/dashboard/collections">
                <LibraryIcon class="icon" />
                {{ formatMessage(commonMessages.collectionsLabel) }}
              </NuxtLink>
              <NuxtLink
                v-if="auth.user.role === 'moderator' || auth.user.role === 'admin'"
                class="iconified-button"
                to="/moderation"
              >
                <ModerationIcon aria-hidden="true" />
                {{ formatMessage(commonMessages.moderationLabel) }}
              </NuxtLink>
              <NuxtLink v-if="flags.developerMode" class="iconified-button" to="/flags">
                <ReportIcon aria-hidden="true" />
                Feature flags
              </NuxtLink>
            </template>
            <NuxtLink class="iconified-button" to="/settings">
              <SettingsIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.settingsLabel) }}
            </NuxtLink>
            <button class="iconified-button" @click="changeTheme">
              <MoonIcon v-if="$theme.active === 'light'" class="icon" />
              <SunIcon v-else class="icon" />
              <span class="dropdown-item__text">
                {{ formatMessage(messages.changeTheme) }}
              </span>
            </button>
          </div>
        </div>
        <div class="mobile-navbar" :class="{ expanded: isBrowseMenuOpen || isMobileMenuOpen }">
          <NuxtLink
            to="/"
            class="tab button-animation"
            :title="formatMessage(navMenuMessages.home)"
          >
            <HomeIcon />
          </NuxtLink>
          <button
            class="tab button-animation"
            :class="{ 'router-link-exact-active': isBrowseMenuOpen }"
            :title="formatMessage(navMenuMessages.search)"
            @click="toggleBrowseMenu()"
          >
            <template v-if="auth.user">
              <SearchIcon />
            </template>
            <template v-else>
              <SearchIcon class="smaller" />
              {{ formatMessage(navMenuMessages.search) }}
            </template>
          </button>
          <template v-if="auth.user">
            <NuxtLink
              to="/dashboard/notifications"
              class="tab button-animation"
              :class="{
                'no-active': isMobileMenuOpen || isBrowseMenuOpen,
              }"
              :title="formatMessage(commonMessages.notificationsLabel)"
              @click="
                () => {
                  isMobileMenuOpen = false;
                  isBrowseMenuOpen = false;
                }
              "
            >
              <NotificationIcon />
            </NuxtLink>
            <NuxtLink
              to="/dashboard"
              class="tab button-animation"
              :title="formatMessage(commonMessages.dashboardLabel)"
            >
              <ChartIcon />
            </NuxtLink>
          </template>
          <button
            class="tab button-animation"
            :title="formatMessage(messages.toggleMenu)"
            @click="toggleMobileMenu()"
          >
            <template v-if="!auth.user">
              <HamburgerIcon v-if="!isMobileMenuOpen" />
              <CrossIcon v-else />
            </template>
            <template v-else>
              <Avatar
                :src="auth.user.avatar_url"
                class="user-icon"
                :class="{ expanded: isMobileMenuOpen }"
                :alt="formatMessage(messages.yourAvatarAlt)"
                aria-hidden="true"
                circle
              />
            </template>
          </button>
        </div>
      </section>
    </header>
    <main>
      <ModalCreation v-if="auth.user" ref="modal_creation" />
      <slot id="main" />
    </main>
    <footer>
      <div class="logo-info" role="region" aria-label="Modrinth information">
        <BrandTextLogo
          aria-hidden="true"
          class="text-logo button-base mx-auto mb-4 lg:mx-0"
          @click="developerModeIncrement()"
        />
        <p class="mb-4">
          <IntlFormatted :message-id="footerMessages.openSource">
            <template #github-link="{ children }">
              <a
                :target="$external()"
                href="https://github.com/modrinth"
                class="text-link"
                rel="noopener"
              >
                <component :is="() => children" />
              </a>
            </template>
          </IntlFormatted>
        </p>
        <p class="mb-4">
          {{ config.public.branch }}@<a
            :target="$external()"
            :href="
              'https://github.com/' +
              config.public.owner +
              '/' +
              config.public.slug +
              '/tree/' +
              config.public.hash
            "
            class="text-link"
            rel="noopener"
            >{{ config.public.hash.substring(0, 7) }}</a
          >
        </p>
        <p>© Rinth, Inc.</p>
      </div>
      <div class="links links-1" role="region" aria-label="Legal">
        <h4 aria-hidden="true">{{ formatMessage(footerMessages.companyTitle) }}</h4>
        <nuxt-link to="/legal/terms"> {{ formatMessage(footerMessages.terms) }}</nuxt-link>
        <nuxt-link to="/legal/privacy"> {{ formatMessage(footerMessages.privacy) }}</nuxt-link>
        <nuxt-link to="/legal/rules"> {{ formatMessage(footerMessages.rules) }}</nuxt-link>
        <a :target="$external()" href="https://careers.modrinth.com">
          {{ formatMessage(footerMessages.careers) }}
          <span v-if="false" class="count-bubble">0</span>
        </a>
      </div>
      <div class="links links-2" role="region" aria-label="Resources">
        <h4 aria-hidden="true">{{ formatMessage(footerMessages.resourcesTitle) }}</h4>
        <a :target="$external()" href="https://support.modrinth.com">
          {{ formatMessage(footerMessages.support) }}
        </a>
        <a :target="$external()" href="https://blog.modrinth.com">
          {{ formatMessage(footerMessages.blog) }}
        </a>
        <a :target="$external()" href="https://docs.modrinth.com">
          {{ formatMessage(footerMessages.docs) }}
        </a>
        <a :target="$external()" href="https://status.modrinth.com">
          {{ formatMessage(footerMessages.status) }}
        </a>
      </div>
      <div class="links links-3" role="region" aria-label="Interact">
        <h4 aria-hidden="true">{{ formatMessage(footerMessages.interactTitle) }}</h4>
        <a rel="noopener" :target="$external()" href="https://discord.modrinth.com"> Discord </a>
        <a rel="noopener" :target="$external()" href="https://x.com/modrinth"> X (Twitter) </a>
        <a rel="noopener" :target="$external()" href="https://floss.social/@modrinth"> Mastodon </a>
        <a rel="noopener" :target="$external()" href="https://crowdin.com/project/modrinth">
          Crowdin
        </a>
      </div>
      <div class="buttons">
        <nuxt-link class="btn btn-outline btn-primary" to="/app">
          <DownloadIcon aria-hidden="true" />
          {{ formatMessage(messages.getModrinthApp) }}
        </nuxt-link>
        <button class="iconified-button raised-button" @click="changeTheme">
          <MoonIcon v-if="$theme.active === 'light'" aria-hidden="true" />
          <SunIcon v-else aria-hidden="true" />
          {{ formatMessage(messages.changeTheme) }}
        </button>
        <nuxt-link class="iconified-button raised-button" to="/settings">
          <SettingsIcon aria-hidden="true" />
          {{ formatMessage(commonMessages.settingsLabel) }}
        </nuxt-link>
      </div>
      <div class="not-affiliated-notice">
        {{ formatMessage(footerMessages.legalDisclaimer) }}
      </div>
    </footer>
  </div>
</template>
<script setup>
import {
  LogInIcon,
  DownloadIcon,
  LibraryIcon,
  XIcon,
  IssuesIcon,
  ReportIcon,
} from "@modrinth/assets";
import { Button, Avatar } from "@modrinth/ui";
import HamburgerIcon from "~/assets/images/utils/hamburger.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import SearchIcon from "~/assets/images/utils/search.svg?component";

import NotificationIcon from "~/assets/images/sidebar/notifications.svg?component";
import SettingsIcon from "~/assets/images/sidebar/settings.svg?component";
import ModerationIcon from "~/assets/images/sidebar/admin.svg?component";
import HomeIcon from "~/assets/images/sidebar/home.svg?component";

import MoonIcon from "~/assets/images/utils/moon.svg?component";
import SunIcon from "~/assets/images/utils/sun.svg?component";
import PlusIcon from "~/assets/images/utils/plus.svg?component";
import DropdownIcon from "~/assets/images/utils/dropdown.svg?component";
import LogOutIcon from "~/assets/images/utils/log-out.svg?component";
import ChartIcon from "~/assets/images/utils/chart.svg?component";

import NavRow from "~/components/ui/NavRow.vue";
import ModalCreation from "~/components/ui/ModalCreation.vue";
import { getProjectTypeMessage } from "~/utils/i18n-project-type.ts";
import { commonMessages } from "~/utils/common-messages.ts";

const { formatMessage } = useVIntl();

const app = useNuxtApp();
const auth = await useAuth();
const cosmetics = useCosmetics();
const flags = useFeatureFlags();
const tags = useTags();

const config = useRuntimeConfig();
const route = useNativeRoute();
const link = config.public.siteUrl + route.path.replace(/\/+$/, "");

const verifyEmailBannerMessages = defineMessages({
  title: {
    id: "layout.banner.verify-email.title",
    defaultMessage: "For security purposes, please verify your email address on Modrinth.",
  },
  action: {
    id: "layout.banner.verify-email.action",
    defaultMessage: "Re-send verification email",
  },
});

const addEmailBannerMessages = defineMessages({
  title: {
    id: "layout.banner.add-email.title",
    defaultMessage: "For security purposes, please enter your email on Modrinth.",
  },
  action: {
    id: "layout.banner.add-email.button",
    defaultMessage: "Visit account settings",
  },
});

const stagingBannerMessages = defineMessages({
  title: {
    id: "layout.banner.staging.title",
    defaultMessage: "You’re viewing Modrinth’s staging environment.",
  },
  description: {
    id: "layout.banner.staging.description",
    defaultMessage:
      "The staging environment is running on a copy of the production Modrinth database. This is used for testing and debugging purposes, and may be running in-development versions of the Modrinth backend or frontend newer than the production instance.",
  },
});

const navMenuMessages = defineMessages({
  home: {
    id: "layout.nav.home",
    defaultMessage: "Home",
  },
  search: {
    id: "layout.nav.search",
    defaultMessage: "Search",
  },
});

const messages = defineMessages({
  toggleMenu: {
    id: "layout.menu-toggle.action",
    defaultMessage: "Toggle menu",
  },
  yourAvatarAlt: {
    id: "layout.avatar.alt",
    defaultMessage: "Your avatar",
  },
  getModrinthApp: {
    id: "layout.action.get-modrinth-app",
    defaultMessage: "Get Modrinth App",
  },
  changeTheme: {
    id: "layout.action.change-theme",
    defaultMessage: "Change theme",
  },
});

const footerMessages = defineMessages({
  openSource: {
    id: "layout.footer.open-source",
    defaultMessage: "Modrinth is <github-link>open source</github-link>.",
  },
  companyTitle: {
    id: "layout.footer.company.title",
    defaultMessage: "Company",
  },
  terms: {
    id: "layout.footer.company.terms",
    defaultMessage: "Terms",
  },
  privacy: {
    id: "layout.footer.company.privacy",
    defaultMessage: "Privacy",
  },
  rules: {
    id: "layout.footer.company.rules",
    defaultMessage: "Rules",
  },
  careers: {
    id: "layout.footer.company.careers",
    defaultMessage: "Careers",
  },
  resourcesTitle: {
    id: "layout.footer.resources.title",
    defaultMessage: "Resources",
  },
  support: {
    id: "layout.footer.resources.support",
    defaultMessage: "Support",
  },
  blog: {
    id: "layout.footer.resources.blog",
    defaultMessage: "Blog",
  },
  docs: {
    id: "layout.footer.resources.docs",
    defaultMessage: "Docs",
  },
  status: {
    id: "layout.footer.resources.status",
    defaultMessage: "Status",
  },
  interactTitle: {
    id: "layout.footer.interact.title",
    defaultMessage: "Interact",
  },
  legalDisclaimer: {
    id: "layout.footer.legal-disclaimer",
    defaultMessage:
      "NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.",
  },
});

useHead({
  link: [
    {
      rel: "canonical",
      href: link,
    },
  ],
});
useSeoMeta({
  title: "Modrinth",
  description: () =>
    formatMessage({
      id: "layout.meta.description",
      defaultMessage:
        "Download Minecraft mods, plugins, datapacks, shaders, resourcepacks, and modpacks on Modrinth. " +
        "Discover and publish projects on Modrinth with a modern, easy to use interface and API.",
    }),
  publisher: "Modrinth",
  themeColor: "#1bd96a",
  colorScheme: "dark light",

  // OpenGraph
  ogTitle: "Modrinth",
  ogSiteName: "Modrinth",
  ogDescription: () =>
    formatMessage({
      id: "layout.meta.og-description",
      defaultMessage: "Discover and publish Minecraft content!",
    }),
  ogType: "website",
  ogImage: "https://cdn.modrinth.com/modrinth-new.png",
  ogUrl: link,

  // Twitter
  twitterCard: "summary",
  twitterSite: "@modrinth",
});

const developerModeCounter = ref(0);

const isDropdownOpen = ref(false);
const isMobileMenuOpen = ref(false);
const isBrowseMenuOpen = ref(false);
const navRoutes = computed(() => [
  {
    label: formatMessage(getProjectTypeMessage("mod", true)),
    href: "/mods",
  },
  {
    label: formatMessage(getProjectTypeMessage("plugin", true)),
    href: "/plugins",
  },
  {
    label: formatMessage(getProjectTypeMessage("datapack", true)),
    href: "/datapacks",
  },
  {
    label: formatMessage(getProjectTypeMessage("shader", true)),
    href: "/shaders",
  },
  {
    label: formatMessage(getProjectTypeMessage("resourcepack", true)),
    href: "/resourcepacks",
  },
  {
    label: formatMessage(getProjectTypeMessage("modpack", true)),
    href: "/modpacks",
  },
]);

onMounted(() => {
  if (window && import.meta.client) {
    window.history.scrollRestoration = "auto";
  }

  runAnalytics();
});

watch(
  () => route.path,
  () => {
    isMobileMenuOpen.value = false;
    isBrowseMenuOpen.value = false;

    if (import.meta.client) {
      document.body.style.overflowY = "scroll";
      document.body.setAttribute("tabindex", "-1");
      document.body.removeAttribute("tabindex");
    }

    updateCurrentDate();
    runAnalytics();
  },
);

function developerModeIncrement() {
  if (developerModeCounter.value >= 5) {
    flags.value.developerMode = !flags.value.developerMode;
    developerModeCounter.value = 0;
    saveFeatureFlags();
    if (flags.value.developerMode) {
      app.$notify({
        group: "main",
        title: "Developer mode activated",
        text: "Developer mode has been enabled",
        type: "success",
      });
    } else {
      app.$notify({
        group: "main",
        title: "Developer mode deactivated",
        text: "Developer mode has been disabled",
        type: "success",
      });
    }
  } else {
    developerModeCounter.value++;
  }
}

async function logoutUser() {
  await logout();
}

function runAnalytics() {
  const config = useRuntimeConfig();
  const replacedUrl = config.public.apiBaseUrl.replace("v2/", "");

  setTimeout(() => {
    $fetch(`${replacedUrl}analytics/view`, {
      method: "POST",
      body: {
        url: window.location.href,
      },
      headers: {
        Authorization: auth.value.token,
      },
    })
      .then(() => {})
      .catch(() => {});
  });
}
function toggleMobileMenu() {
  isMobileMenuOpen.value = !isMobileMenuOpen.value;
  if (isMobileMenuOpen.value) {
    isBrowseMenuOpen.value = false;
  }
}
function toggleBrowseMenu() {
  isBrowseMenuOpen.value = !isBrowseMenuOpen.value;

  if (isBrowseMenuOpen.value) {
    isMobileMenuOpen.value = false;
  }
}

const { cycle: changeTheme } = useTheme();

function hideStagingBanner() {
  cosmetics.value.hideStagingBanner = true;
}
</script>

<style lang="scss">
@import "~/assets/styles/global.scss";
// @import '@modrinth/assets';

.layout {
  min-height: 100vh;
  background-color: var(--color-bg);
  display: block;

  @media screen and (min-width: 1024px) {
    min-height: calc(100vh - var(--spacing-card-bg));
  }

  @media screen and (max-width: 750px) {
    margin-bottom: calc(var(--size-mobile-navbar-height) + 2rem);
  }

  .site-header {
    max-width: 100vw;

    @media screen and (min-width: 1024px) {
      margin-top: var(--spacing-card-md);
      margin-bottom: var(--spacing-card-md);
    }

    @media screen and (min-width: 1280px) {
      border-radius: var(--size-rounded-sm);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;
    }

    .navbar {
      padding: 0 var(--spacing-card-lg);
      margin: 0 var(--spacing-card-lg);
      max-width: 1280px;
      margin-left: auto;
      margin-right: auto;

      section.logo {
        display: flex;
        justify-content: space-between;
        color: var(--color-text-dark);
        z-index: 5;

        a {
          align-items: center;
          display: flex;

          &:not(:focus-visible) {
            outline: none;

            &.router-link-exact-active {
              outline: 2px solid transparent;
              border-radius: 0.25rem;
            }
          }
        }

        .small-logo {
          display: block;
        }

        svg {
          height: 1.75rem;
          width: auto;
        }

        button {
          background: none;
          border: none;
          margin: 0 0 0 0.5rem;
          padding: 0;

          svg {
            height: 1.5rem;
            width: 1.5rem;
          }
        }
      }

      section.nav-group {
        display: flex;
        flex-grow: 5;
        z-index: 5;

        section.nav {
          flex-grow: 5;

          .navigation {
            display: flex;
            width: fit-content;
            position: relative;
            top: 50%;
            transform: translateY(-50%);
            margin-left: 2rem;
            grid-gap: 1.5rem;

            a {
              margin-left: 0;
              margin-right: auto;
            }

            a.tab {
              padding: 0;
              margin-right: 1rem;
              display: flex;
              align-items: flex-start;

              &--alpha::after {
                content: "Alpha";
                background-color: var(--color-warning-bg);
                color: var(--color-warning-text);
                border-radius: 1rem;
                padding: 0.25rem 0.5rem;
                margin-left: 0.4rem;
                font-size: 0.7rem;
              }
            }
          }
        }

        .user-outer {
          z-index: 5;
        }

        section.user-controls {
          align-items: center;
          display: flex;
          flex-direction: row;
          justify-content: space-between;
          position: relative;
          top: 50%;
          transform: translateY(-50%);
          min-width: 6rem;
          gap: 0.25rem;

          .control-button {
            position: relative;
            display: flex;
            padding: 0.5rem;
            color: var(--color-text);
            border-radius: 2rem;
            transition: filter 0.1s ease-in-out;
            border: 2px solid transparent;
            box-sizing: border-box;

            svg {
              height: 1.25rem;
              width: 1.25rem;
            }

            &.bubble {
              &::after {
                background-color: var(--color-brand);
                border-radius: var(--size-rounded-max);
                content: "";
                height: 0.5rem;
                position: absolute;
                right: 0.25rem;
                top: 0.5rem;
                width: 0.5rem;
              }
            }

            //&.router-link-exact-active {
            //  color: var(--color-button-text-active);
            //  background-color: var(--color-button-bg);
            //}
          }

          .hide-desktop {
            display: none;
          }

          .dropdown {
            position: relative;
            margin-left: 0.5rem;

            .control {
              align-items: center;
              background: none;
              display: flex;
              justify-content: center;
              padding: 0;
              outline: none;

              .user-icon {
                --size: 2rem !important;
                outline: 2px solid var(--color-raised-bg);
                transition: outline-color 0.1s ease-in-out;
              }

              .caret {
                color: var(--color-button-text);
                margin-left: 0.25rem;
                width: 1rem;
              }
            }

            .content {
              border: 1px solid var(--color-divider-dark);
              list-style: none;
              margin: 0.5rem 0 0 0;
              max-width: 25rem;
              min-width: 12rem;
              opacity: 0;
              padding: 1rem;
              position: absolute;
              right: -1rem;
              transform: scaleY(0.9);
              transform-origin: top;
              transition:
                all 0.1s ease-in-out 0.05s,
                color 0s ease-in-out 0s,
                background-color 0s ease-in-out 0s,
                border-color 0s ease-in-out 0s;
              visibility: hidden;
              width: max-content;
              z-index: 1;
              box-shadow: var(--shadow-floating);

              .divider {
                background-color: var(--color-divider-dark);
                border: none;
                color: var(--color-divider-dark);
                height: 1px;
                margin: 0.5rem 0;
              }

              .item {
                align-items: center;
                border-radius: 0.5rem;
                box-sizing: border-box;
                color: inherit;
                display: flex;
                padding: 0.5rem 0.75rem;
                width: 100%;
                outline: none;

                .icon {
                  margin-right: 0.5rem;
                  height: 20px;
                  width: 20px;
                }

                &.router-link-exact-active {
                  color: var(--color-button-text-active);
                  background-color: var(--color-button-bg);
                  outline: 2px solid transparent;

                  &.primary-color {
                    color: var(--color-button-text-active);
                    background-color: var(--color-brand-highlight);
                  }
                }

                &.primary-color {
                  color: var(--color-brand);
                }
              }

              .profile-link {
                .prompt {
                  margin-top: 0.25rem;
                  color: var(--color-text-secondary);
                }
              }
            }

            @media screen and (max-width: 1300px) {
              .content {
                margin-right: 1rem;
              }
            }
          }

          .dropdown:hover .user-icon {
            outline-color: var(--color-brand);
          }

          .dropdown:hover:not(.closed) .content,
          .dropdown:focus:not(.closed) .content,
          .dropdown:focus-within:not(.closed) .content {
            opacity: 1;
            transform: scaleY(1);
            visibility: visible;
          }
        }

        section.auth-prompt {
          display: flex;
          align-items: center;
          height: 100%;
          margin: 0;
          gap: 0.5rem;

          .log-in-button {
            margin: 0 auto;
          }
        }
      }

      @media screen and (max-width: 1095px) {
        display: none;
      }
    }

    .mobile-navigation {
      display: none;

      .nav-menu {
        width: 100%;
        position: fixed;
        bottom: calc(var(--size-mobile-navbar-height) - var(--size-rounded-card));
        padding-bottom: var(--size-rounded-card);
        left: 0;
        background-color: var(--color-raised-bg);
        z-index: 6;
        transform: translateY(100%);
        transition: transform 0.4s cubic-bezier(0.54, 0.84, 0.42, 1);
        border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
        box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0);

        .links,
        .account-container {
          display: grid;
          grid-template-columns: repeat(1, 1fr);
          grid-gap: 1rem;
          justify-content: center;
          padding: 1rem;

          .iconified-button {
            width: 100%;
            max-width: 500px;
            padding: 0.75rem;
            justify-content: center;
            font-weight: 600;
            font-size: 1rem;
            margin: 0 auto;
          }
        }

        .cascade-links {
          @media screen and (min-width: 354px) {
            grid-template-columns: repeat(2, 1fr);
          }
          @media screen and (min-width: 674px) {
            grid-template-columns: repeat(3, 1fr);
          }
        }

        &-browse {
          &.expanded {
            transform: translateY(0);
            box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
          }
        }

        &-mobile {
          .account-container {
            padding-bottom: 0;

            .account-button {
              padding: var(--spacing-card-md);
              display: flex;
              align-items: center;
              justify-content: center;
              gap: 0.5rem;

              .user-icon {
                width: 2.25rem;
                height: 2.25rem;
              }

              .account-text {
                flex-grow: 0;
              }
            }
          }

          &.expanded {
            transform: translateY(0);
            box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
          }
        }
      }

      .mobile-navbar {
        display: flex;
        height: calc(var(--size-mobile-navbar-height) + env(safe-area-inset-bottom));
        border-radius: var(--size-rounded-card) var(--size-rounded-card) 0 0;
        padding-bottom: env(safe-area-inset-bottom);
        position: fixed;
        left: 0;
        bottom: 0;
        background-color: var(--color-raised-bg);
        box-shadow: 0 0 20px 2px rgba(0, 0, 0, 0.3);
        z-index: 7;
        width: 100%;
        align-items: center;
        justify-content: space-between;
        transition: border-radius 0.3s ease-out;
        border-top: 2px solid rgba(0, 0, 0, 0);
        box-sizing: border-box;

        &.expanded {
          box-shadow: none;
          border-radius: 0;
        }

        .tab {
          position: relative;
          background: none;
          display: flex;
          flex-basis: 0;
          justify-content: center;
          align-items: center;
          flex-direction: row;
          gap: 0.25rem;
          font-weight: bold;
          padding: 0;
          transition: color ease-in-out 0.15s;
          color: var(--color-text-inactive);
          text-align: center;

          &.browse {
            svg {
              transform: rotate(180deg);
              transition: transform ease-in-out 0.3s;

              &.closed {
                transform: rotate(0deg);
              }
            }
          }

          &.bubble {
            &::after {
              background-color: var(--color-brand);
              border-radius: var(--size-rounded-max);
              content: "";
              height: 0.5rem;
              position: absolute;
              left: 1.5rem;
              top: 0;
              width: 0.5rem;
            }
          }

          svg {
            height: 1.75rem;
            width: 1.75rem;

            &.smaller {
              width: 1.25rem;
              height: 1.25rem;
            }
          }

          .user-icon {
            width: 2rem;
            height: 2rem;
            transition: border ease-in-out 0.15s;
            border: 0 solid var(--color-brand);
            box-sizing: border-box;

            &.expanded {
              border: 2px solid var(--color-brand);
            }
          }

          &:hover,
          &:focus {
            color: var(--color-text);
          }

          &:first-child {
            margin-left: 2rem;
          }

          &:last-child {
            margin-right: 2rem;
          }

          &.router-link-exact-active:not(&.no-active) {
            svg {
              color: var(--color-brand);
            }

            color: var(--color-brand);
          }
        }
      }

      @media screen and (max-width: 1095px) {
        display: flex;
      }
    }

    div {
      flex-grow: 1;
      justify-content: end;
      align-items: center;
      row-gap: 1rem;
    }

    &.active {
      display: flex;

      @media screen and (min-width: 1095px) {
        display: none;
      }
    }
  }

  main {
    grid-area: main;
  }

  footer {
    margin: 6rem 0 2rem 0;
    text-align: center;
    display: grid;
    grid-template:
      "logo-info  logo-info  logo-info" auto
      "links-1    links-2    links-3" auto
      "buttons    buttons    buttons" auto
      "notice     notice     notice" auto
      / 1fr 1fr 1fr;
    max-width: 1280px;

    .logo-info {
      margin-left: auto;
      margin-right: auto;
      max-width: 15rem;
      margin-bottom: 1rem;
      grid-area: logo-info;

      .text-logo {
        width: 10rem;
        height: auto;
      }
    }

    .links {
      display: flex;
      flex-direction: column;
      margin-bottom: 1rem;

      h4 {
        color: var(--color-text-dark);
        margin: 0 0 1rem 0;
      }

      a {
        margin: 0 0 1rem 0;
      }

      &.links-1 {
        grid-area: links-1;
      }

      &.links-2 {
        grid-area: links-2;
      }

      &.links-3 {
        grid-area: links-3;
      }

      .count-bubble {
        font-size: 1rem;
        border-radius: 5rem;
        background: var(--color-brand);
        color: var(--color-text-inverted);
        padding: 0 0.35rem;
        margin-left: 0.25rem;
      }
    }

    .buttons {
      margin-left: auto;
      margin-right: auto;
      grid-area: buttons;

      button,
      a {
        margin-bottom: 0.5rem;
        margin-left: auto;
        margin-right: auto;
      }
    }

    .not-affiliated-notice {
      grid-area: notice;
      font-size: var(--font-size-xs);
      text-align: center;
      font-weight: 500;
      margin-top: var(--spacing-card-md);
    }

    @media screen and (min-width: 1024px) {
      display: grid;
      margin-inline: auto;
      grid-template:
        "logo-info  links-1 links-2 links-3 buttons" auto
        "notice     notice  notice  notice  notice" auto;
      text-align: unset;

      .logo-info {
        margin-right: 4rem;
      }

      .links {
        margin-right: 4rem;
      }

      .buttons {
        width: unset;
        margin-left: 0;

        button,
        a {
          margin-right: unset;
        }
      }

      .not-affiliated-notice {
        margin-top: 0;
      }
    }
  }
}

@media (min-width: 1024px) {
  .layout {
    main {
      .alpha-alert {
        margin: 1rem;

        .wrapper {
          padding: 1rem 2rem 1rem 1rem;
        }
      }
    }
  }
}

.email-nag {
  background-color: var(--color-raised-bg);
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
}

.site-banner--warning {
  background-color: var(--color-red-bg);
  border-bottom: 2px solid var(--color-red);
  display: grid;
  gap: 0.5rem;
  grid-template: "title actions" "description actions";
  padding-block: var(--gap-xl);
  padding-inline: max(calc((100% - 80rem) / 2 + var(--gap-md)), var(--gap-xl));

  .site-banner__title {
    grid-area: title;
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-weight: bold;
    font-size: var(--font-size-md);
    color: var(--color-contrast);

    svg {
      color: var(--color-red);
      width: 1.5rem;
      height: 1.5rem;
      flex-shrink: 0;
    }
  }

  .site-banner__description {
    grid-area: description;
  }

  .site-banner__actions {
    grid-area: actions;
  }

  a {
    color: var(--color-red);
  }
}

@media (max-width: 1200px) {
  .app-btn {
    display: none;
  }
}
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
