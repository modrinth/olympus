<template>
  <div>
    <section class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Tags</span>
        </h3>
      </div>
      <p>
        Accurate tagging is important to help people find your
        {{ $formatProjectType(project.project_type).toLowerCase() }}. Make sure to select all tags
        that apply.
      </p>
      <p v-if="project.versions.length === 0" class="known-errors">
        Please upload a version first in order to select tags!
      </p>
      <template v-else>
        <template v-for="header in Object.keys(categoryLists)" :key="`categories-${header}`">
          <div class="label">
            <h4>
              <span class="label__title">{{ $formatCategoryHeader(header) }}</span>
            </h4>
            <span class="label__description">
              <template v-if="header === 'categories'">
                Select all categories that reflect the themes or function of your
                {{ $formatProjectType(project.project_type).toLowerCase() }}.
              </template>
              <template v-else-if="header === 'features'">
                Select all of the features that your
                {{ $formatProjectType(project.project_type).toLowerCase() }} makes use of.
              </template>
              <template v-else-if="header === 'resolutions'">
                Select the resolution(s) of textures in your
                {{ $formatProjectType(project.project_type).toLowerCase() }}.
              </template>
              <template v-else-if="header === 'performance impact'">
                Select the realistic performance impact of your
                {{ $formatProjectType(project.project_type).toLowerCase() }}. Select multiple if the
                {{ $formatProjectType(project.project_type).toLowerCase() }} is configurable to
                different levels of performance impact.
              </template>
            </span>
          </div>
          <div class="category-list input-div">
            <Checkbox
              v-for="category in categoryLists[header]"
              :key="`category-${header}-${category.name}`"
              :model-value="selectedTags.includes(category)"
              :description="$formatCategory(category.name)"
              class="category-selector"
              @update:model-value="toggleCategory(category)"
            >
              <div class="category-selector__label">
                <div
                  v-if="header !== 'resolutions' && category.icon"
                  aria-hidden="true"
                  class="icon"
                  v-html="category.icon"
                />
                <span aria-hidden="true"> {{ $formatCategory(category.name) }}</span>
              </div>
            </Checkbox>
          </div>
        </template>
        <div class="label">
          <h4>
            <span class="label__title"><StarIcon /> Featured tags</span>
          </h4>
          <span class="label__description">
            You can feature up to 3 of your most relevant tags. Other tags may be promoted to
            featured if you do not select all 3.
          </span>
        </div>
        <p v-if="selectedTags.length < 1">
          Select at least one category in order to feature a category.
        </p>
        <div class="category-list input-div">
          <Checkbox
            v-for="category in selectedTags"
            :key="`featured-category-${category.name}`"
            class="category-selector"
            :model-value="featuredTags.includes(category)"
            :description="$formatCategory(category.name)"
            :disabled="featuredTags.length >= 3 && !featuredTags.includes(category)"
            @update:model-value="toggleFeaturedCategory(category)"
          >
            <div class="category-selector__label">
              <div
                v-if="category.header !== 'resolutions' && category.icon"
                aria-hidden="true"
                class="icon"
                v-html="category.icon"
              />
              <span aria-hidden="true"> {{ $formatCategory(category.name) }}</span>
            </div>
          </Checkbox>
        </div>
      </template>

      <div class="button-group">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="!hasChanges"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </section>
  </div>
</template>

<script>
import Checkbox from "~/components/ui/Checkbox.vue";
import StarIcon from "~/assets/images/utils/star.svg?component";
import SaveIcon from "~/assets/images/utils/save.svg?component";

export default defineNuxtComponent({
  components: {
    Checkbox,
    SaveIcon,
    StarIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {};
      },
    },
    allMembers: {
      type: Array,
      default() {
        return [];
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null;
      },
    },
    patchProject: {
      type: Function,
      default() {
        return () => {
          this.$notify({
            group: "main",
            title: "An error occurred",
            text: "Patch project function not found",
            type: "error",
          });
        };
      },
    },
  },
  data() {
    return {
      selectedTags: this.$sortedCategories().filter(
        (x) =>
          x.project_type === this.project.actualProjectType &&
          (this.project.categories.includes(x.name) ||
            this.project.additional_categories.includes(x.name)),
      ),
      featuredTags: this.$sortedCategories().filter(
        (x) =>
          x.project_type === this.project.actualProjectType &&
          this.project.categories.includes(x.name),
      ),
    };
  },
  computed: {
    categoryLists() {
      const lists = {};
      this.$sortedCategories().forEach((x) => {
        if (x.project_type === this.project.actualProjectType) {
          const header = x.header;
          if (!lists[header]) {
            lists[header] = [];
          }
          lists[header].push(x);
        }
      });
      return lists;
    },
    patchData() {
      const data = {};
      // Promote selected categories to featured if there are less than 3 featured
      const newFeaturedTags = this.featuredTags.slice();
      if (newFeaturedTags.length < 1 && this.selectedTags.length > newFeaturedTags.length) {
        const nonFeaturedCategories = this.selectedTags.filter((x) => !newFeaturedTags.includes(x));

        nonFeaturedCategories
          .slice(0, Math.min(nonFeaturedCategories.length, 3 - newFeaturedTags.length))
          .forEach((x) => newFeaturedTags.push(x));
      }
      // Convert selected and featured categories to backend-usable arrays
      const categories = newFeaturedTags.map((x) => x.name);
      const additionalCategories = this.selectedTags
        .filter((x) => !newFeaturedTags.includes(x))
        .map((x) => x.name);

      if (
        categories.length !== this.project.categories.length ||
        categories.some((value) => !this.project.categories.includes(value))
      ) {
        data.categories = categories;
      }

      if (
        additionalCategories.length !== this.project.additional_categories.length ||
        additionalCategories.some((value) => !this.project.additional_categories.includes(value))
      ) {
        data.additional_categories = additionalCategories;
      }

      return data;
    },
    hasChanges() {
      return Object.keys(this.patchData).length > 0;
    },
  },
  methods: {
    toggleCategory(category) {
      if (this.selectedTags.includes(category)) {
        this.selectedTags = this.selectedTags.filter((x) => x !== category);
        if (this.featuredTags.includes(category)) {
          this.featuredTags = this.featuredTags.filter((x) => x !== category);
        }
      } else {
        this.selectedTags.push(category);
      }
    },
    toggleFeaturedCategory(category) {
      if (this.featuredTags.includes(category)) {
        this.featuredTags = this.featuredTags.filter((x) => x !== category);
      } else {
        this.featuredTags.push(category);
      }
    },
    saveChanges() {
      if (this.hasChanges) {
        this.patchProject(this.patchData);
      }
    },
  },
});
</script>
<style lang="scss" scoped>
.label__title {
  display: flex;
  align-items: center;
  gap: var(--spacing-card-xs);
  margin-top: var(--spacing-card-bg);

  svg {
    vertical-align: top;
  }
}

.category-list {
  column-count: 4;
  column-gap: var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);

  :deep(.category-selector) {
    margin-bottom: 0.5rem;
    .category-selector__label {
      display: flex;
      align-items: center;
      .icon {
        height: 1rem;
        svg {
          margin-right: 0.25rem;
          width: 1rem;
          height: 1rem;
        }
      }
    }
    span {
      user-select: none;
    }
  }

  @media only screen and (max-width: 1250px) {
    column-count: 3;
  }
  @media only screen and (max-width: 1024px) {
    column-count: 4;
  }
  @media only screen and (max-width: 960px) {
    column-count: 3;
  }
  @media only screen and (max-width: 750px) {
    column-count: 2;
  }
  @media only screen and (max-width: 530px) {
    column-count: 1;
  }
}
</style>
