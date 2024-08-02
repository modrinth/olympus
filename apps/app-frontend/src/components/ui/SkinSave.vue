<script setup>
import { Card } from '@modrinth/ui'

const props = defineProps({
  data: {
    type: Object,
    default() {
      return {}
    },
  },
})

const emit = defineEmits(['setSkin'])
</script>

<template>
  <div class="instance">
    <Card class="instance-card-item button-base" @click="emit('setSkin', props.data)">
      <img
        :src="props.data.model"
        alt="Mod card"
        class="mod-image"
        draggable="false"
      />
      <div class="project-info">
        <p class="title">{{ props.data.name }}</p>
        <p class="description">{{ props.data.arms }}, {{ props.data.cape }}</p>
      </div>
    </Card>
  </div>
</template>

<style lang="scss" scoped>
.instance {
  position: relative;

  &:hover {
    .cta {
      opacity: 1;
      bottom: calc(var(--gap-md) + 4.25rem);
    }
  }
}

.instance-card-item {
  display: block;
  flex: none;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .mod-image {
    --size: 100%;
    width: 144px;
    height: 144px;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      width: 144px;
      margin: 0;
      font-weight: 600;
      font-size: 1rem;
      line-height: 110%;
      display: inline-block;
    }

    .description {
      color: var(--color-base);
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      font-weight: 500;
      font-size: 0.775rem;
      line-height: 125%;
      margin: 0.25rem 0 0;
      text-transform: capitalize;
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
}
</style>
