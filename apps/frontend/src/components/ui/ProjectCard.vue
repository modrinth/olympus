<template>
  <article class="project-card base-card padding-bg" :aria-label="name" role="listitem">
    <nuxt-link
      :title="name"
      class="icon"
      tabindex="-1"
      :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`"
    >
      <Avatar :src="iconUrl" :alt="name" size="md" no-shadow loading="lazy" />
    </nuxt-link>
    <nuxt-link
      class="gallery"
      :class="{ 'no-image': !featuredImage }"
      tabindex="-1"
      :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`"
      :style="color ? `background-color: ${toColor};` : ''"
    >
      <img v-if="featuredImage" :src="featuredImage" alt="gallery image" loading="lazy" />
    </nuxt-link>
    <div class="title">
      <nuxt-link :to="`/${$getProjectTypeForUrl(type, categories)}/${id}`">
        <h2 class="name !text-2xl">
          {{ name }}
        </h2>
      </nuxt-link>
      <p v-if="author" class="author">
        by
        <nuxt-link class="title-link" :to="'/user/' + author">
          {{ author }}
        </nuxt-link>
      </p>
      <Badge v-if="status && status !== 'approved'" :type="status" class="status" />
    </div>
    <p class="description">
      {{ description }}
    </p>
    <Categories
      :categories="
        categories.filter((x) => !hideLoaders || !tags.loaders.find((y) => y.name === x))
      "
      :type="type"
      class="tags"
    >
      <EnvironmentIndicator
        v-if="clientSide && serverSide"
        :type-only="moderation"
        :client-side="clientSide"
        :server-side="serverSide"
        :type="projectTypeDisplay"
        :search="search"
        :categories="categories"
      />
    </Categories>
    <div class="stats">
      <div v-if="downloads" class="stat">
        <DownloadIcon aria-hidden="true" />
        <p>
          <strong>{{ $formatNumber(downloads) }}</strong
          ><span class="stat-label"> download<span v-if="downloads !== '1'">s</span></span>
        </p>
      </div>
      <div v-if="follows" class="stat">
        <HeartIcon aria-hidden="true" />
        <p>
          <strong>{{ $formatNumber(follows) }}</strong
          ><span class="stat-label"> follower<span v-if="follows !== '1'">s</span></span>
        </p>
      </div>
      <div class="buttons">
        <slot />
      </div>
      <div
        v-if="showUpdatedDate"
        v-tooltip="$dayjs(updatedAt).format('MMMM D, YYYY [at] h:mm A')"
        class="stat date"
      >
        <EditIcon aria-hidden="true" />
        <span class="date-label">Updated </span>{{ fromNow(updatedAt) }}
      </div>
      <div
        v-else-if="showCreatedDate"
        v-tooltip="$dayjs(createdAt).format('MMMM D, YYYY [at] h:mm A')"
        class="stat date"
      >
        <CalendarIcon aria-hidden="true" />
        <span class="date-label">Published </span>{{ fromNow(createdAt) }}
      </div>
    </div>
  </article>
</template>

<script>
import Categories from "~/components/ui/search/Categories.vue";
import { Badge } from "@modrinth/ui";
import { EnvironmentIndicator } from "@modrinth/ui";

import CalendarIcon from "~/assets/images/utils/calendar.svg?component";
import EditIcon from "~/assets/images/utils/updated.svg?component";
import DownloadIcon from "~/assets/images/utils/download.svg?component";
import HeartIcon from "~/assets/images/utils/heart.svg?component";
import { Avatar } from "@modrinth/ui";

export default {
  components: {
    EnvironmentIndicator,
    Avatar,
    Categories,
    Badge,
    CalendarIcon,
    EditIcon,
    DownloadIcon,
    HeartIcon,
  },
  props: {
    id: {
      type: String,
      default: "modrinth-0",
    },
    type: {
      type: String,
      default: "mod",
    },
    name: {
      type: String,
      default: "Project Name",
    },
    author: {
      type: String,
      default: null,
    },
    description: {
      type: String,
      default: "A _type description",
    },
    iconUrl: {
      type: String,
      default: "#",
      required: false,
    },
    downloads: {
      type: String,
      default: null,
      required: false,
    },
    follows: {
      type: String,
      default: null,
      required: false,
    },
    createdAt: {
      type: String,
      default: "0000-00-00",
    },
    updatedAt: {
      type: String,
      default: null,
    },
    categories: {
      type: Array,
      default() {
        return [];
      },
    },
    status: {
      type: String,
      default: null,
    },
    hasModMessage: {
      type: Boolean,
      default: false,
    },
    serverSide: {
      type: String,
      required: false,
      default: "",
    },
    clientSide: {
      type: String,
      required: false,
      default: "",
    },
    moderation: {
      type: Boolean,
      required: false,
      default: false,
    },
    search: {
      type: Boolean,
      required: false,
      default: false,
    },
    featuredImage: {
      type: String,
      required: false,
      default: null,
    },
    showUpdatedDate: {
      type: Boolean,
      required: false,
      default: true,
    },
    showCreatedDate: {
      type: Boolean,
      required: false,
      default: true,
    },
    hideLoaders: {
      type: Boolean,
      required: false,
      default: false,
    },
    color: {
      type: Number,
      required: false,
      default: null,
    },
  },
  setup() {
    const tags = useTags();

    return { tags };
  },
  computed: {
    projectTypeDisplay() {
      return this.$getProjectTypeForDisplay(this.type, this.categories);
    },
    toColor() {
      let color = this.color;

      color >>>= 0;
      const b = color & 0xff;
      const g = (color & 0xff00) >>> 8;
      const r = (color & 0xff0000) >>> 16;
      return "rgba(" + [r, g, b, 1].join(",") + ")";
    },
  },
};
</script>

<style lang="scss" scoped>
.project-card {
  display: inline-grid;
  box-sizing: border-box;
  overflow: hidden;
  margin: 0;
}

.display-mode--list .project-card {
  grid-template:
    "icon title stats"
    "icon description stats"
    "icon tags stats";
  grid-template-columns: min-content 1fr auto;
  grid-template-rows: min-content 1fr min-content;
  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-sm);
  width: 100%;

  @media screen and (max-width: 750px) {
    grid-template:
      "icon title"
      "icon description"
      "icon tags"
      "stats stats";
    grid-template-columns: min-content auto;
    grid-template-rows: min-content 1fr min-content min-content;
  }

  @media screen and (max-width: 550px) {
    grid-template:
      "icon title"
      "icon description"
      "tags tags"
      "stats stats";
    grid-template-columns: min-content auto;
    grid-template-rows: min-content 1fr min-content min-content;
  }
}

.display-mode--gallery .project-card,
.display-mode--grid .project-card {
  padding: 0 0 var(--spacing-card-bg) 0;
  grid-template: "gallery gallery" "icon title" "description  description" "tags tags" "stats stats";
  grid-template-columns: min-content 1fr;
  grid-template-rows: min-content min-content 1fr min-content min-content;
  row-gap: var(--spacing-card-sm);

  .gallery {
    display: inline-block;
    width: 100%;
    height: 10rem;
    background-color: var(--color-button-bg-active);

    &.no-image {
      filter: brightness(0.7);
    }

    img {
      box-shadow: none;
      width: 100%;
      height: 10rem;
      object-fit: cover;
    }
  }

  .icon {
    margin-left: var(--spacing-card-bg);
    margin-top: -3rem;
    z-index: 1;

    img,
    svg {
      border-radius: var(--size-rounded-lg);
      box-shadow:
        -2px -2px 0 2px var(--color-raised-bg),
        2px -2px 0 2px var(--color-raised-bg);
    }
  }

  .title {
    margin-left: var(--spacing-card-md);
    margin-right: var(--spacing-card-bg);
    flex-direction: column;

    .name {
      font-size: 1.25rem;
    }

    .status {
      margin-top: var(--spacing-card-xs);
    }
  }

  .description {
    margin-inline: var(--spacing-card-bg);
  }

  .tags {
    margin-inline: var(--spacing-card-bg);
  }

  .stats {
    margin-inline: var(--spacing-card-bg);
    flex-direction: row;
    align-items: center;

    .stat-label {
      display: none;
    }

    .buttons {
      flex-direction: row;
      gap: var(--spacing-card-sm);
      align-items: center;

      > :first-child {
        margin-left: auto;
      }

      &:first-child > :last-child {
        margin-right: auto;
      }
    }

    .buttons:not(:empty) + .date {
      flex-basis: 100%;
    }
  }
}

.display-mode--grid .project-card {
  .gallery {
    display: none;
  }

  .icon {
    margin-top: calc(var(--spacing-card-bg) - var(--spacing-card-sm));

    img,
    svg {
      border: none;
    }
  }

  .title {
    margin-top: calc(var(--spacing-card-bg) - var(--spacing-card-sm));
  }
}

.icon {
  grid-area: icon;
  display: flex;
  align-items: center;
}

.gallery {
  display: none;
  height: 10rem;
  grid-area: gallery;
}

.title {
  grid-area: title;
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: baseline;
  column-gap: var(--spacing-card-sm);
  row-gap: 0;
  word-wrap: anywhere;

  h2,
  p {
    margin: 0;
  }

  svg {
    width: auto;
    color: var(--color-orange);
    height: 1.5rem;
    margin-bottom: -0.25rem;
  }
}

.stats {
  grid-area: stats;
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: var(--spacing-card-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--spacing-card-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      height: var(--stat-strong-size);
      width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }

  @media screen and (max-width: 750px) {
    flex-direction: row;
    column-gap: var(--spacing-card-md);
    margin-top: var(--spacing-card-xs);
  }

  @media screen and (max-width: 600px) {
    margin-top: 0;

    .stat-label {
      display: none;
    }
  }
}

.environment {
  color: var(--color-text) !important;
  font-weight: bold;
}

.description {
  grid-area: description;
  margin-block: 0;
  display: flex;
  justify-content: flex-start;
}

.tags {
  grid-area: tags;
  display: flex;
  flex-direction: row;

  @media screen and (max-width: 550px) {
    margin-top: var(--spacing-card-xs);
  }
}

.buttons {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-sm);
  align-items: flex-end;
  flex-grow: 1;
}

.small-mode {
  @media screen and (min-width: 750px) {
    grid-template:
      "icon title"
      "icon description"
      "icon tags"
      "stats stats" !important;
    grid-template-columns: min-content auto !important;
    grid-template-rows: min-content 1fr min-content min-content !important;

    .tags {
      margin-top: var(--spacing-card-xs) !important;
    }

    .stats {
      flex-direction: row;
      column-gap: var(--spacing-card-md) !important;
      margin-top: var(--spacing-card-xs) !important;

      .stat-label {
        display: none !important;
      }
    }
  }
}
</style>
