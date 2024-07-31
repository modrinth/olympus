<template>
  <Checkbox
    class="filter"
    :model-value="activeFilters.includes(facetName)"
    :description="displayName"
    @update:model-value="toggle()"
  >
    <div class="filter-text">
      <div v-if="icon" aria-hidden="true" class="icon" v-html="icon" />
      <div v-else class="icon">
        <slot />
      </div>
      <span aria-hidden="true"> {{ displayName }}</span>
    </div>
  </Checkbox>
</template>

<script>
import { Checkbox } from "@modrinth/ui";

export default {
  components: {
    Checkbox,
  },
  props: {
    facetName: {
      type: String,
      default: "",
    },
    displayName: {
      type: String,
      default: "",
    },
    icon: {
      type: String,
      default: "",
    },
    activeFilters: {
      type: Array,
      default() {
        return [];
      },
    },
  },
  emits: ["toggle"],
  methods: {
    toggle() {
      this.$emit("toggle", this.facetName);
    },
  },
};
</script>

<style lang="scss" scoped>
.filter {
  margin-bottom: 0.5rem;

  :deep(.filter-text) {
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
</style>
