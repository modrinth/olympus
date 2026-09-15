<template>
	<div v-if="visibleNags.length > 0" class="universal-card my-4">
		<div class="flex max-w-full flex-wrap items-center gap-x-6 gap-y-4">
			<div class="flex flex-auto flex-wrap items-center gap-x-6 gap-y-4">
				<h2 class="my-0 mr-auto">
					{{ getFormattedMessage(messages.publishingChecklist) }}
				</h2>
				<div class="flex flex-row gap-2">
					<div class="flex items-center gap-1">
						<AsteriskIcon class="size-4 shrink-0 text-red" />
						<span class="text-secondary">{{ getFormattedMessage(messages.required) }}</span>
					</div>
					|
					<div class="flex items-center gap-1">
						<TriangleAlertIcon class="size-4 shrink-0 text-orange" />
						<span class="text-secondary">{{ getFormattedMessage(messages.warning) }}</span>
					</div>
					|
					<div class="flex items-center gap-1">
						<LightBulbIcon class="size-4 shrink-0 text-purple" />
						<span class="text-secondary">{{ getFormattedMessage(messages.suggestion) }}</span>
					</div>
				</div>
			</div>
			<div class="input-group">
				<IconButton
					label="Toggle details"
					:aria-expanded="!collapsed"
					:class="{ '[&>svg]:rotate-180': !collapsed }"
					@click="$emit('toggleCollapsed')"
				>
					<DropdownIcon class="transition-transform duration-300 ease-in-out" />
				</IconButton>
			</div>
		</div>
		<Accordion :open-by-default="!collapsed" content-class="mt-4">
			<div class="relative">
				<div
					v-if="!disableHorizontalScroll"
					class="nag-scroll-shadow-left pointer-events-none absolute bottom-0 left-0 top-0 z-10 w-8 bg-surface-3 transition-opacity duration-200"
					:class="showLeftNagShadow ? 'opacity-100' : 'opacity-0'"
				/>
				<div
					ref="nagScroller"
					class="flex w-full gap-2 pb-2"
					:class="{
						'grid grid-cols-4': disableHorizontalScroll,
						'overflow-x-auto overflow-y-hidden': !disableHorizontalScroll,
						'cursor-grab select-none': canScrollNags,
						'is-dragging': draggingNags,
					}"
					@pointerdown="onNagPointerDown"
					@pointermove="onNagPointerMove"
					@pointerup="finishNagDrag"
					@pointercancel="finishNagDrag"
					@click.capture="onNagClick"
					@wheel="onNagWheel"
					@scroll="updateNagScrollShadows"
				>
					<div
						v-for="nag in visibleNags"
						:key="nag.id"
						class="flex w-[268px] shrink-0 flex-col gap-2.5 rounded-2xl border border-solid border-surface-5 bg-surface-2 p-4"
					>
						<span class="flex items-start gap-2 font-medium text-contrast">
							<component
								:is="nag.icon || getDefaultIcon(nag.status)"
								v-tooltip="getStatusTooltip(nag.status)"
								:class="[
									'mt-0.5 size-4 min-w-4',
									nag.status === 'required' && 'text-red',
									nag.status === 'warning' && 'text-orange',
									nag.status === 'suggestion' && 'text-purple',
								]"
							/>
							{{ getFormattedMessage(nag.title) }}
						</span>
						<span>
							<span
								v-for="(segment, index) in getNagDescriptionSegments(nag)"
								:key="index"
								:class="{ 'break-all': segment.isUrl }"
								v-text="segment.text"
							/>
						</span>
						<NuxtLink
							v-if="nag.link && shouldShowLink(nag)"
							:to="`/${project.project_type}/${project.slug ? project.slug : project.id}/${
								nag.link.path
							}`"
							class="goto-link mt-auto"
						>
							{{ getFormattedMessage(nag.link.title) }}
							<ChevronRightIcon aria-hidden="true" class="featured-header-chevron" />
						</NuxtLink>
						<Button
							v-if="nag.status === 'special-submit-action' && nag.id === 'submit-for-review'"
							v-tooltip="
								!canSubmitForReview
									? getFormattedMessage(messages.submitChecklistTooltip)
									: undefined
							"
							type="colored"
							color="orange"
							:disabled="!canSubmitForReview"
							@click="submitForReview"
						>
							<SendIcon />
							{{ getFormattedMessage(messages.submitForReviewButton) }}
						</Button>
					</div>
				</div>
				<div
					v-if="!disableHorizontalScroll"
					class="nag-scroll-shadow-right pointer-events-none absolute bottom-0 right-0 top-0 z-10 w-8 bg-surface-3 transition-opacity duration-200"
					:class="showRightNagShadow ? 'opacity-100' : 'opacity-0'"
				/>
			</div>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AsteriskIcon,
	ChevronRightIcon,
	DropdownIcon,
	LightBulbIcon,
	ScaleIcon,
	SendIcon,
	TriangleAlertIcon,
} from '@modrinth/assets'
import type { Nag, NagContext, NagStatus } from '@modrinth/moderation'
import { nagDestinations, normalizeProjectNagKind, toProjectNag } from '@modrinth/moderation'
import { Accordion, Button, IconButton } from '@modrinth/ui'
import {
	commonMessages,
	defineMessages,
	injectNotificationManager,
	type MessageDescriptor,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import type { Component } from 'vue'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

interface Tags {
	categories?: Labrinth.Tags.v2.Category[]
	rejectedStatuses: string[]
	gameVersions: { version: string }[]
	loaders: { name: string }[]
}

interface Props {
	project: Labrinth.Projects.v2.Project
	projectV3: Labrinth.Projects.v3.Project
	versions?: Labrinth.Versions.v3.Version[]
	nags?: Nag[]
	validationNags?: Labrinth.Projects.v3.ProjectNag[]
	validationLoading?: boolean
	validationAvailable?: boolean
	refreshValidation?: () => Promise<Labrinth.Projects.v3.ProjectValidationResponse | null>
	currentMember?: Labrinth.Projects.v3.TeamMember | null
	collapsed?: boolean
	disableHorizontalScroll?: boolean
	routeName?: string
	tags: Tags
}

const messages = defineMessages({
	publishingChecklist: {
		id: 'project-moderation-nags.publishing-checklist',
		defaultMessage: 'Publishing checklist',
	},
	submitForReview: {
		id: 'project-moderation-nags.submit-for-review',
		defaultMessage: 'Submit for review',
	},
	submitForReviewDesc: {
		id: 'project-moderation-nags.submit-for-review-desc',
		defaultMessage:
			'Your project is only viewable by members of the project. It must be reviewed by moderators in order to be published.',
	},
	submitForReviewButton: {
		id: 'project-moderation-nags.submit-for-review-button',
		defaultMessage: 'Submit for review',
	},
	resubmitForReview: {
		id: 'project-moderation-nags.resubmit-for-review',
		defaultMessage: 'Resubmit for review',
	},
	resubmitForReviewDesc: {
		id: 'project-moderation-nags.resubmit-for-review-desc',
		defaultMessage:
			"Your project has been {status, select, rejected {rejected} withheld {withheld} other {{status}}} by Modrinth's staff. In most cases, you can resubmit for review after addressing the staff's message.",
	},
	visitModerationPage: {
		id: 'project-moderation-nags.visit-moderation-page',
		defaultMessage: 'Visit moderation page',
	},
	submitChecklistTooltip: {
		id: 'project-moderation-nags.submit-checklist-tooltip',
		defaultMessage: 'You must complete the required steps in the publishing checklist!',
	},
	required: {
		id: 'project-moderation-nags.required',
		defaultMessage: 'Required',
	},
	warning: {
		id: 'project-moderation-nags.warning',
		defaultMessage: 'Warning',
	},
	suggestion: {
		id: 'project-moderation-nags.suggestion',
		defaultMessage: 'Suggestion',
	},
	projectSubmittedForReview: {
		id: 'project-moderation-nags.project-submitted-for-review',
		defaultMessage: 'Your project has been submitted for review!',
	},
})

const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const flags = useFeatureFlags()

const props = withDefaults(defineProps<Props>(), {
	versions: () => [],
	currentMember: null,
	collapsed: false,
	disableHorizontalScroll: false,
	routeName: '',
	validationNags: () => [],
	validationLoading: false,
	validationAvailable: true,
})

const emit = defineEmits<{
	toggleCollapsed: []
	setProcessing: [processing: boolean]
}>()

const isProcessing = computed(() => props.project.status === 'processing')

const nagScroller = ref<HTMLElement | null>(null)
const canScrollNags = ref(false)
const showLeftNagShadow = ref(false)
const showRightNagShadow = ref(false)
const draggingNags = ref(false)

let nagScrollerResizeObserver: ResizeObserver | null = null
let nagDragPointerId: number | null = null
let nagDragCaptureTarget: Element | null = null
let nagDragStartX = 0
let nagDragStartScrollLeft = 0
let suppressNagClick = false
let suppressNagClickTimeout: ReturnType<typeof setTimeout> | null = null

function updateNagScrollShadows() {
	const el = nagScroller.value
	if (!el || props.disableHorizontalScroll) {
		canScrollNags.value = false
		showLeftNagShadow.value = false
		showRightNagShadow.value = false
		return
	}

	canScrollNags.value = el.scrollWidth > el.clientWidth + 1
	showLeftNagShadow.value = canScrollNags.value && el.scrollLeft > 0
	showRightNagShadow.value =
		canScrollNags.value && el.scrollLeft < el.scrollWidth - el.clientWidth - 1
}

function onNagWheel(event: WheelEvent) {
	const el = nagScroller.value
	if (props.disableHorizontalScroll || !el || el.scrollWidth <= el.clientWidth) return

	const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY
	event.preventDefault()
	el.scrollLeft += delta
}

function onNagPointerDown(event: PointerEvent) {
	const el = nagScroller.value
	if (
		props.disableHorizontalScroll ||
		!el ||
		el.scrollWidth <= el.clientWidth + 1 ||
		event.pointerType === 'touch' ||
		event.button !== 0
	)
		return

	nagDragPointerId = event.pointerId
	nagDragStartX = event.clientX
	nagDragStartScrollLeft = el.scrollLeft
	suppressNagClick = false
	nagDragCaptureTarget =
		event.target instanceof Element ? (event.target.closest('a, button') ?? el) : el
	nagDragCaptureTarget.setPointerCapture(event.pointerId)
}

function onNagPointerMove(event: PointerEvent) {
	const el = nagScroller.value
	if (!el || event.pointerId !== nagDragPointerId) return

	const distance = event.clientX - nagDragStartX
	if (!draggingNags.value && Math.abs(distance) < 4) return

	draggingNags.value = true
	suppressNagClick = true
	event.preventDefault()
	el.scrollLeft = nagDragStartScrollLeft - distance
}

function finishNagDrag(event: PointerEvent) {
	if (event.pointerId !== nagDragPointerId) return

	if (nagDragCaptureTarget?.hasPointerCapture(event.pointerId)) {
		nagDragCaptureTarget.releasePointerCapture(event.pointerId)
	}
	nagDragPointerId = null
	nagDragCaptureTarget = null
	draggingNags.value = false

	if (suppressNagClick) {
		if (suppressNagClickTimeout) clearTimeout(suppressNagClickTimeout)
		suppressNagClickTimeout = setTimeout(() => {
			suppressNagClick = false
			suppressNagClickTimeout = null
		}, 0)
	}
}

function onNagClick(event: MouseEvent) {
	if (!suppressNagClick) return

	event.preventDefault()
	event.stopPropagation()
	suppressNagClick = false
	if (suppressNagClickTimeout) clearTimeout(suppressNagClickTimeout)
	suppressNagClickTimeout = null
}

onMounted(() => {
	nagScrollerResizeObserver = new ResizeObserver(updateNagScrollShadows)
	if (nagScroller.value) nagScrollerResizeObserver.observe(nagScroller.value)
	nextTick(updateNagScrollShadows)
})

onBeforeUnmount(() => {
	nagScrollerResizeObserver?.disconnect()
	if (suppressNagClickTimeout) clearTimeout(suppressNagClickTimeout)
})

watch(nagScroller, (el, previousEl) => {
	if (previousEl) nagScrollerResizeObserver?.unobserve(previousEl)
	if (el) nagScrollerResizeObserver?.observe(el)
	nextTick(updateNagScrollShadows)
})

watch(
	() => props.disableHorizontalScroll,
	() => nextTick(updateNagScrollShadows),
)

const nagContext = computed<NagContext>(() => ({
	project: props.project,
	projectV3: props.projectV3,
	versions: props.versions,
	currentMember: props.currentMember?.user,
	currentRoute: props.routeName,
	tags: props.tags,
}))

const canSubmitForReview = computed(() => {
	return (
		!props.validationLoading &&
		props.validationAvailable &&
		!props.validationNags.some((nag) => nag.severity === 'required')
	)
})

async function submitForReview() {
	if (!canSubmitForReview.value) return
	const validation = await props.refreshValidation?.()
	if (!validation || validation.nags.some((nag) => nag.severity === 'required')) return
	if (!props.collapsed) emit('toggleCollapsed')
	emit('setProcessing', true)
	await navigateTo(
		`/${props.project.project_type}/${props.project.slug ?? props.project.id}/${nagDestinations.moderation.path}`,
	)
	addNotification({
		type: 'success',
		title: formatMessage(commonMessages.successLabel),
		text: formatMessage(messages.projectSubmittedForReview),
	})
}

const applicableNags = computed<Nag[]>(() => {
	if (props.nags) return props.nags

	const nagsByKind = new Map<
		Labrinth.Projects.v3.NormalizedProjectNagKind,
		Labrinth.Projects.v3.ProjectNag
	>()
	for (const nag of props.validationNags) {
		const kind = normalizeProjectNagKind(nag.kind)
		if (kind && !nagsByKind.has(kind)) nagsByKind.set(kind, nag)
	}

	return [...nagsByKind.values()].map((nag) => toProjectNag(nag, props.project.project_type))
})

function isNagComplete(nag: Nag): boolean {
	const context = nagContext.value
	return !nag.shouldShow(context)
}

const visibleNags = computed<Nag[]>(() => {
	const finalNags = applicableNags.value.filter((nag) => !isNagComplete(nag))

	if (
		isProcessing.value &&
		isStaff(props.currentMember?.user) &&
		!flags.value.alwaysShowPublishingChecklistForStaff &&
		!finalNags.some((nag) => nag.status === 'required')
	) {
		return []
	}

	if (props.project.status === 'draft') {
		finalNags.push({
			id: 'submit-for-review',
			title: messages.submitForReview,
			description: () => formatMessage(messages.submitForReviewDesc),
			status: 'special-submit-action',
			shouldShow: (ctx) => ctx.project.status === 'draft',
		})
	}

	if (props.tags.rejectedStatuses.includes(props.project.status)) {
		finalNags.push({
			id: 'resubmit-for-review',
			title: messages.resubmitForReview,
			description: (ctx) =>
				formatMessage(messages.resubmitForReviewDesc, { status: ctx.project.status }),
			status: 'special-submit-action',
			shouldShow: (ctx) => ctx.tags.rejectedStatuses.includes(ctx.project.status),
			link: {
				...nagDestinations.moderation,
				title: messages.visitModerationPage,
			},
		})
	}

	finalNags.sort((a, b) => {
		const statusOrder = { required: 0, warning: 1, suggestion: 2, 'special-submit-action': 3 }
		return statusOrder[a.status] - statusOrder[b.status]
	})

	return finalNags
})

watch(visibleNags, () => nextTick(updateNagScrollShadows))

watch(isProcessing, (processing) => {
	if (processing && !props.collapsed) emit('toggleCollapsed')
})

let validationProjectId = props.project.id
let previousActionableNagKeys: Set<string> | null = null

watch(
	[
		() => props.project.id,
		() => props.validationLoading,
		() => props.validationAvailable,
		() => props.validationNags,
	],
	([projectId, validationLoading, validationAvailable, validationNags]) => {
		if (projectId !== validationProjectId) {
			validationProjectId = projectId
			previousActionableNagKeys = null
		}

		if (validationLoading || !validationAvailable) return

		const actionableNagKeys = new Set(
			validationNags
				.filter((nag) => nag.severity === 'required' || nag.severity === 'warning')
				.map((nag) => `${nag.severity}:${nag.kind}`),
		)
		const previousNagKeys = previousActionableNagKeys
		const hasNewActionableNag =
			previousNagKeys !== null && [...actionableNagKeys].some((key) => !previousNagKeys.has(key))

		previousActionableNagKeys = actionableNagKeys

		if (isProcessing.value && props.collapsed && hasNewActionableNag) {
			emit('toggleCollapsed')
		}
	},
	{ deep: true, immediate: true },
)

function shouldShowLink(nag: Nag): boolean {
	return nag.link?.shouldShow(nagContext.value) ?? false
}

function getDefaultIcon(status: NagStatus): Component {
	switch (status) {
		case 'required':
			return AsteriskIcon
		case 'warning':
			return TriangleAlertIcon
		case 'suggestion':
			return LightBulbIcon
		case 'special-submit-action':
			return ScaleIcon
		default:
			return AsteriskIcon
	}
}

function getStatusTooltip(status: NagStatus): string {
	switch (status) {
		case 'required':
			return formatMessage(messages.required)
		case 'warning':
			return formatMessage(messages.warning)
		case 'suggestion':
			return formatMessage(messages.suggestion)
		default:
			return formatMessage(messages.required)
	}
}

function getNagDescription(nag: Nag): string {
	if (typeof nag.description === 'function') {
		return nag.description(nagContext.value)
	}
	return formatMessage(nag.description, nag.values)
}

function getNagDescriptionSegments(nag: Nag): { text: string; isUrl: boolean }[] {
	return getNagDescription(nag)
		.split(/(https?:\/\/[^\s"'<>“”]+)/gi)
		.filter(Boolean)
		.map((text) => ({ text, isUrl: /^https?:\/\//i.test(text) }))
}

function getFormattedMessage(message: string | MessageDescriptor): string {
	if (typeof message === 'string') {
		return message
	}
	return formatMessage(message)
}
</script>

<style lang="scss" scoped>
.is-dragging,
.is-dragging * {
	cursor: grabbing !important;
}

.nag-scroll-shadow-left {
	-webkit-mask-image: linear-gradient(to right, black, transparent);
	mask-image: linear-gradient(to right, black, transparent);
}

.nag-scroll-shadow-right {
	-webkit-mask-image: linear-gradient(to left, black, transparent);
	mask-image: linear-gradient(to left, black, transparent);
}
</style>
