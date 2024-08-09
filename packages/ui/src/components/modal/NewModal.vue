<template>
  <div v-if="open">
    <div
      :class="{ shown: visible }"
      class="tauri-overlay"
      data-tauri-drag-region
      @click="() => (closable ? hide() : {})"
    />
    <div
      :class="{
        shown: visible,
        noblur: props.noblur,
      }"
      class="modal-overlay"
      @click="() => (closable ? hide() : {})"
    />
    <div class="modal-container" :class="{ shown: visible }">
      <div class="modal-body flex flex-col bg-bg-raised rounded-2xl p-6">
        <div class="flex items-center pb-6 border-b-[1px] border-button-bg">
          <div class="flex flex-grow items-center gap-3">
            <slot name="title" />
          </div>
          <ButtonStyled v-if="closable" circular>
            <button @click="hide">
              <XIcon />
            </button>
          </ButtonStyled>
        </div>
        <div class="overflow-y-auto">
          <slot> You just lost the game. </slot>
        </div>
      </div>
    </div>
  </div>
  <div v-else></div>
</template>

<script setup>
import { XIcon } from '@modrinth/assets'
import { ref } from 'vue'
import ButtonStyled from '../base/ButtonStyled.vue'

const props = defineProps({
  noblur: {
    type: Boolean,
    default: false,
  },
  closable: {
    type: Boolean,
    default: true,
  },
})

const open = ref(false)
const visible = ref(false)

function show() {
  open.value = true
  setTimeout(() => {
    visible.value = true
  }, 50)
}

function hide() {
  visible.value = false
  setTimeout(() => {
    open.value = false
  }, 300)
}

defineExpose({
  show,
  hide,
})
</script>

<style lang="scss" scoped>
.tauri-overlay {
  position: fixed;
  visibility: hidden;
  top: 0;
  left: 0;
  width: 100%;
  height: 100px;
  z-index: 20;

  &.shown {
    opacity: 1;
    visibility: visible;
  }
}

.modal-overlay {
  visibility: hidden;
  position: fixed;
  inset: -5rem;
  z-index: 19;
  opacity: 0;
  transition: all 0.2s ease-out;
  background: linear-gradient(to bottom, rgba(27, 48, 42, 0.52) 0%, rgba(13, 21, 26, 0.95) 100%);
  transform: translateY(2rem) scale(0.8);
  border-radius: 120px;
  filter: blur(5px);

  @media (prefers-reduced-motion) {
    transition: none !important;
  }

  &.shown {
    opacity: 1;
    visibility: visible;
    backdrop-filter: blur(5px);
    transform: translateY(0) scale(1);
    border-radius: 0px;
  }

  &.noblur {
    backdrop-filter: none;
    filter: none;
  }
}

.modal-container {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 21;
  visibility: hidden;
  pointer-events: none;

  &.shown {
    visibility: visible;
    .modal-body {
      opacity: 1;
      visibility: visible;
      transform: translateY(0);
      scale: 1;
    }
  }

  .modal-body {
    position: fixed;
    box-shadow: 4px 4px 26px 10px rgba(0, 0, 0, 0.08);
    max-height: calc(100% - 2 * var(--gap-lg));
    max-width: min(var(--_max-width, 60rem), calc(100% - 2 * var(--gap-lg)));
    overflow-y: auto;
    width: fit-content;
    pointer-events: auto;
    scale: 0.97;

    transform: translateY(1rem);
    visibility: hidden;
    opacity: 0;
    transition: all 0.2s ease-in-out;

    @media (prefers-reduced-motion) {
      transition: none !important;
    }

    @media screen and (max-width: 650px) {
      width: calc(100% - 2 * var(--gap-lg));
    }
  }
}
</style>
