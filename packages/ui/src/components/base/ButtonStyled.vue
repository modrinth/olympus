<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    color?: 'standard' | 'brand' | 'red' | 'orange' | 'green' | 'blue' | 'purple'
    size?: 'standard' | 'large'
    circular?: boolean
    type?: 'standard' | 'outlined' | 'transparent'
    colorFill?: 'auto' | 'background' | 'text' | 'none'
    hoverColorFill?: 'auto' | 'background' | 'text' | 'none'
  }>(),
  {
    color: 'standard',
    size: 'standard',
    circular: false,
    type: 'standard',
    colorFill: 'auto',
    hoverColorFill: 'auto',
  },
)

const colorVar = computed(() => {
  switch (props.color) {
    case 'brand':
      return 'var(--color-brand)'
    case 'red':
      return 'var(--color-red)'
    case 'orange':
      return 'var(--color-orange)'
    case 'green':
      return 'var(--color-green)'
    case 'blue':
      return 'var(--color-blue)'
    case 'purple':
      return 'var(--color-purple)'
    case 'standard':
    default:
      return null
  }
})

const height = computed(() => {
  if (props.size === 'large') {
    return '3rem'
  }
  return '2.25rem'
})

const width = computed(() => {
  if (props.size === 'large') {
    return props.circular ? '3rem' : 'auto'
  }
  return props.circular ? '2.25rem' : 'auto'
})

const paddingX = computed(() => {
  let padding = props.circular ? '0.5rem' : '0.75rem'
  if (props.size === 'large') {
    padding = props.circular ? '0.75rem' : '1rem'
  }
  return `calc(${padding} - 0.125rem)`
})

const paddingY = computed(() => {
  if (props.size === 'large') {
    return '0.75rem'
  }
  return '0.5rem'
})

const gap = computed(() => {
  if (props.size === 'large') {
    return '0.5rem'
  }
  return '0.375rem'
})

const fontWeight = computed(() => {
  if (props.size === 'large') {
    return '800'
  }
  return '600'
})

const radius = computed(() => {
  if (props.circular) {
    return '99999px'
  }

  if (props.size === 'large') {
    return '1rem'
  }
  return '0.75rem'
})

const iconSize = computed(() => {
  if (props.size === 'large') {
    return '1.5rem'
  }
  return '1.25rem'
})

function setColorFill(
  colors: { bg: string; text: string },
  fill: 'background' | 'text' | 'none',
): { bg: string; text: string } {
  if (colorVar.value) {
    if (fill === 'background') {
      colors.bg = colorVar.value
      colors.text = 'var(--color-accent-contrast)'
    } else if (fill === 'text') {
      colors.text = colorVar.value
    }
  }
  return colors
}

const colorVariables = computed(() => {
  let colors = {
    bg: 'var(--color-button-bg)',
    text: 'var(--color-base)',
  }
  let hoverColors = JSON.parse(JSON.stringify(colors))

  if (props.type === 'outlined') {
    hoverColors.bg = 'transparent'
  }

  if (props.type === 'outlined' || props.type === 'transparent') {
    colors.bg = 'transparent'
    colors = setColorFill(colors, props.colorFill === 'auto' ? 'text' : props.colorFill)
    hoverColors = setColorFill(
      hoverColors,
      props.hoverColorFill === 'auto' ? 'text' : props.hoverColorFill,
    )
  } else {
    colors = setColorFill(colors, props.colorFill === 'auto' ? 'background' : props.colorFill)
    hoverColors = setColorFill(
      hoverColors,
      props.hoverColorFill === 'auto' ? 'background' : props.hoverColorFill,
    )
  }

  return `--_bg: ${colors.bg}; --_text: ${colors.text}; --_hover-bg: ${hoverColors.bg}; --_hover-text: ${hoverColors.text};`
})
</script>

<template>
  <div
    class="btn-wrapper"
    :class="{ outline: type === 'outlined' }"
    :style="`${colorVariables}--_height:${height};--_width:${width};--_radius: ${radius};--_padding-x:${paddingX};--_padding-y:${paddingY};--_gap:${gap};--_font-weight:${fontWeight};--_icon-size:${iconSize};`"
  >
    <slot />
  </div>
</template>

<style scoped>
.btn-wrapper {
  display: contents;
}

/* Searches up to 4 children deep for valid button */
.btn-wrapper :slotted(:is(button, a):first-child),
.btn-wrapper :slotted(*) > :is(button, a):first-child,
.btn-wrapper :slotted(*) > *:first-child > :is(button, a):first-child,
.btn-wrapper :slotted(*) > *:first-child > *:first-child > :is(button, a):first-child {
  @apply flex flex-row items-center justify-center border-solid border-2 border-transparent active:scale-95 hover:brightness-125 focus-visible:brightness-125 bg-[--_bg] text-[--_text] hover:bg-[--_hover-bg] hover:text-[--_hover-text] focus-visible:bg-[--_hover-bg] focus-visible:text-[--_hover-text] h-[--_height] min-w-[--_width] rounded-[--_radius] px-[--_padding-x] py-[--_padding-y] gap-[--_gap] font-[--_font-weight];
  transition:
    scale 0.125s ease-in-out,
    background-color 0.25s ease-in-out;
}

.btn-wrapper.outline :slotted(:is(button, a):first-child),
.btn-wrapper.outline :slotted(*) > :is(button, a):first-child,
.btn-wrapper.outline :slotted(*) > *:first-child > :is(button, a):first-child,
.btn-wrapper.outline :slotted(*) > *:first-child > *:first-child > :is(button, a):first-child {
  @apply border-current;
}

/*noinspection CssUnresolvedCustomProperty*/
.btn-wrapper :slotted(:is(button, a):first-child) > svg:first-child,
.btn-wrapper :slotted(*) > :is(button, a):first-child > svg:first-child,
.btn-wrapper :slotted(*) > *:first-child > :is(button, a):first-child > svg:first-child,
.btn-wrapper
  :slotted(*)
  > *:first-child
  > *:first-child
  > :is(button, a):first-child
  > svg:first-child {
  min-width: var(--_icon-size, 1rem);
  min-height: var(--_icon-size, 1rem);
}

/* guys, I know this is nuts, I know */
</style>
