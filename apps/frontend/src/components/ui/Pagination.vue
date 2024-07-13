<template>
  <div v-if="count > 1" class="columns paginates">
    <a
      :class="{ disabled: page === 1 }"
      :tabindex="page === 1 ? -1 : 0"
      class="left-arrow paginate has-icon"
      aria-label="Previous Page"
      :href="linkFunction(page - 1)"
      @click.prevent="page !== 1 ? switchPage(page - 1) : null"
    >
      <LeftArrowIcon />
    </a>
    <div
      v-for="(item, index) in pages"
      :key="'page-' + item + '-' + index"
      :class="{
        'page-number': page !== item,
        shrink: item > 99,
      }"
      class="page-number-container"
    >
      <div v-if="item === '-'" class="has-icon">
        <GapIcon />
      </div>
      <a
        v-else
        :class="{
          'page-number current': page === item,
          shrink: item > 99,
        }"
        :href="linkFunction(item)"
        @click.prevent="page !== item ? switchPage(item) : null"
      >
        {{ item }}
      </a>
    </div>

    <a
      :class="{
        disabled: page === pages[pages.length - 1],
      }"
      :tabindex="page === pages[pages.length - 1] ? -1 : 0"
      class="right-arrow paginate has-icon"
      aria-label="Next Page"
      :href="linkFunction(page + 1)"
      @click.prevent="page !== pages[pages.length - 1] ? switchPage(page + 1) : null"
    >
      <RightArrowIcon />
    </a>
  </div>
</template>

<script>
import GapIcon from "~/assets/images/utils/gap.svg?component";
import LeftArrowIcon from "~/assets/images/utils/left-arrow.svg?component";
import RightArrowIcon from "~/assets/images/utils/right-arrow.svg?component";

export default {
  components: {
    GapIcon,
    LeftArrowIcon,
    RightArrowIcon,
  },
  props: {
    page: {
      type: Number,
      default: 1,
    },
    count: {
      type: Number,
      default: 1,
    },
    linkFunction: {
      type: Function,
      default() {
        return () => "/";
      },
    },
  },
  emits: ["switch-page"],
  computed: {
    pages() {
      let pages = [];

      if (this.count > 7) {
        if (this.page + 3 >= this.count) {
          pages = [
            1,
            "-",
            this.count - 4,
            this.count - 3,
            this.count - 2,
            this.count - 1,
            this.count,
          ];
        } else if (this.page > 5) {
          pages = [1, "-", this.page - 1, this.page, this.page + 1, "-", this.count];
        } else {
          pages = [1, 2, 3, 4, 5, "-", this.count];
        }
      } else {
        pages = Array.from({ length: this.count }, (_, i) => i + 1);
      }

      return pages;
    },
  },
  methods: {
    switchPage(newPage) {
      this.$emit("switch-page", newPage);
      if (newPage !== null && newPage !== "" && !isNaN(newPage)) {
        this.$emit("switch-page", Math.min(Math.max(newPage, 1), this.count));
      }
    },
  },
};
</script>

<style scoped lang="scss">
a {
  position: relative;
  color: var(--color-button-text);
  box-shadow: var(--shadow-raised), var(--shadow-inset);

  padding: 0.5rem 1rem;
  margin: 0;
  border-radius: 2rem;
  background: var(--color-raised-bg);

  transition:
    opacity 0.5s ease-in-out,
    filter 0.2s ease-in-out,
    transform 0.05s ease-in-out,
    outline 0.2s ease-in-out;

  &.page-number.current {
    background: var(--color-brand);
    color: var(--color-brand-inverted);
    cursor: default;
    outline: 2px solid transparent;
  }

  &.paginate.disabled {
    background-color: transparent;
    cursor: not-allowed;
    filter: grayscale(50%);
    opacity: 0.5;
  }
}

.has-icon {
  display: flex;
  align-items: center;
  svg {
    width: 1em;
  }
}

.page-number-container,
a,
.has-icon {
  display: flex;
  justify-content: center;
  align-items: center;
}

.paginates {
  height: 2em;
  margin: 0.5rem 0;
  > div,
  .has-icon {
    margin: 0 0.3em;
  }
}

.left-arrow {
  margin-left: auto !important;
}

.right-arrow {
  margin-right: auto !important;
}

@media screen and (max-width: 400px) {
  .paginates {
    font-size: 80%;
  }
}

@media screen and (max-width: 530px) {
  a {
    width: 2.5rem;
    padding: 0.5rem 0;
  }
}
</style>
