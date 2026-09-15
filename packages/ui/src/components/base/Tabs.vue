<template>
	<div
		v-if="tabs.length > 0"
		class="inline-flex w-fit items-center overflow-x-auto rounded-xl border border-solid border-surface-5 p-0.5 shadow-sm gap-1 h-[38px]"
		role="tablist"
	>
		<button
			v-for="(tab, index) in tabs"
			:key="tab.value"
			ref="tabButtons"
			type="button"
			class="flex min-h-6 shrink-0 cursor-pointer items-center justify-center gap-2 rounded-[10px] border border-solid px-2.5 h-full text-sm font-medium outline-none transition-all active:scale-[0.97] focus-visible:ring-4 focus-visible:ring-brand-shadow disabled:cursor-not-allowed disabled:opacity-50"
			:class="
				tab.value === value
					? color === 'gray'
						? 'border-surface-5 bg-surface-4 text-contrast'
						: 'border-green bg-highlight-green text-green'
					: 'border-transparent bg-transparent text-primary hover:bg-surface-4'
			"
			role="tab"
			:disabled="disabled"
			:aria-selected="tab.value === value"
			:tabindex="tab.value === value || (!hasSelectedTab && index === 0) ? 0 : -1"
			@click="selectTab(tab)"
			@keydown="onTabKeydown($event, index)"
		>
			<component
				:is="tab.icon"
				v-if="tab.icon"
				class="size-5 shrink-0"
				:class="
					tab.value === value
						? color === 'gray'
							? 'text-contrast'
							: 'text-green'
						: 'text-secondary'
				"
			/>
			<span v-if="tab.label" class="text-nowrap">{{ tab.label }}</span>
		</button>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, ref } from 'vue'

export type TabsValue = string | number

export interface TabsTab {
	value: TabsValue
	label: string
	icon?: Component
}

const props = defineProps<{
	value: TabsValue
	tabs: TabsTab[]
	color?: 'green' | 'gray'
	disabled?: boolean
}>()

const emit = defineEmits<{
	'update:value': [value: TabsValue]
	change: [tab: TabsTab]
}>()

const tabButtons = ref<HTMLButtonElement[]>()

const hasSelectedTab = computed(() => props.tabs.some((tab) => tab.value === props.value))

function selectTab(tab: TabsTab) {
	if (props.disabled) return
	emit('update:value', tab.value)
	emit('change', tab)
}

function selectTabAtIndex(index: number) {
	const tab = props.tabs[index]
	if (!tab) return

	selectTab(tab)
	requestAnimationFrame(() => {
		tabButtons.value?.[index]?.focus()
	})
}

function onTabKeydown(event: KeyboardEvent, index: number) {
	if (props.disabled || props.tabs.length === 0) return

	const lastIndex = props.tabs.length - 1
	let nextIndex: number | undefined

	if (event.key === 'ArrowRight') {
		nextIndex = index === lastIndex ? 0 : index + 1
	} else if (event.key === 'ArrowLeft') {
		nextIndex = index === 0 ? lastIndex : index - 1
	} else if (event.key === 'Home') {
		nextIndex = 0
	} else if (event.key === 'End') {
		nextIndex = lastIndex
	}

	if (nextIndex === undefined) return

	event.preventDefault()
	selectTabAtIndex(nextIndex)
}
</script>
