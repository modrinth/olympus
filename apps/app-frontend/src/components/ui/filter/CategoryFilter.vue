<script setup>
import { computed, ref } from 'vue'
import { get_categories, sortByNameOrNumber } from '@/helpers/tags.js'
import { handleError } from '@/store/notifications.js'
import { SearchFilter } from '@modrinth/ui'
import { formatCategory, formatCategoryHeader } from '@modrinth/utils'

const props = defineProps({
  projectType: {
    type: String,
    required: true,
  },
  facets: {
    type: Object,
    required: true,
  },
})

const emits = defineEmits(['toggleFacet'])

function onToggleFacet(elementName) {
  emits('toggleFacet', elementName)
}

const sortedCategories = computed(() => {
  const values = new Map()

  outer: for (const category of categories.value.filter((cat) => {
    return (
      cat.project_type === (props.projectType === 'datapack' ? 'mod' : props.projectType) ||
      props.projectType === 'all'
    )
  })) {
    if (!values.has(category.header)) {
      values.set(category.header, [])
    }

    for (let existingCategory of values.get(category.header)) {
      if (existingCategory.name === category.name) {
        continue outer
      }
    }

    values.get(category.header).push(category)
  }
  return values
})

const categories = await get_categories()
  .catch(handleError)
  .then((s) => sortByNameOrNumber(s, ['header', 'name']))
  .then(ref)
</script>

<template>
  <div
    v-for="categoryList in Array.from(sortedCategories)"
    :key="categoryList[0]"
    class="categories"
  >
    <h2>{{ formatCategoryHeader(categoryList[0]) }}</h2>
    <div v-for="category in categoryList[1]" :key="category.name">
      <SearchFilter
        :active-filters="facets"
        :icon="category.icon"
        :display-name="formatCategory(category.name)"
        :facet-name="`categories:${encodeURIComponent(category.name)}`"
        class="filter-checkbox"
        @toggle="onToggleFacet"
      />
    </div>
  </div>
</template>

<style scoped lang="scss"></style>
