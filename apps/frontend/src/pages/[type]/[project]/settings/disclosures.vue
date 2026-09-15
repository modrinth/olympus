<script setup lang="ts">
import {
	commonMessages,
	ConfirmLeaveModal,
	defineMessages,
	EmptyState,
	injectModrinthClient,
	injectProjectPageContext,
	IntlFormatted,
	isDisclosureCompatibleWithProjectTypes,
	type MessageDescriptor,
	normalizeChildren,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, watch } from 'vue'

import {
	AdvertisingDisclosureCard,
	AiDisclosureCard,
	AiFunctionalityDisclosureCard,
	ArchivedDisclosureCard,
	DerivativeDisclosureCard,
	type DisclosureFormIssue,
	type DisclosureLockStatus,
	disclosuresToForm,
	type DisclosureType,
	type DisclosureUpdatedByUser,
	findDisclosureData,
	getDisclosureFormIssues,
	getDisclosureFormSnapshot,
	PaidFeaturesDisclosureCard,
	PhotosensitivityDisclosureCard,
	SystemInteractionsDisclosureCard,
	TelemetryDisclosureCard,
	toModifyRequests,
} from '~/components/ui/project-settings/disclosures'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useAuth } from '~/composables/auth'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'

const DISCLOSURE_QUERY_STALE_TIME = 1000 * 60 * 5

const { formatMessage } = useVIntl()
const { labrinth } = injectModrinthClient()
const {
	projectV2: project,
	projectV3,
	currentMember,
	refreshProjectValidation,
} = injectProjectPageContext()
const queryClient = useQueryClient()
const flags = useFeatureFlags()
const auth = await useAuth()

const projectTypes = computed(() => projectV3.value?.project_types ?? [project.value.project_type])

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)

const canEditDisclosures = computed(
	() => project.value.versions.length > 0 || isServerProject.value,
)

function isDisclosureVisible(type: DisclosureType) {
	return isDisclosureCompatibleWithProjectTypes(type, projectTypes.value)
}

const isActingAsModerator = computed(
	() => isStaff(currentMember.value?.user) && !flags.value.showModeratorProjectMemberUi,
)

const messages = defineMessages({
	title: {
		id: 'project.settings.disclosures.content-disclosures',
		defaultMessage: 'Content disclosures',
	},
	description: {
		id: 'project.settings.disclosures.description',
		defaultMessage: `You must add any applicable content disclosures to your project in compliance with <rules>Modrinth's Content Rules</rules>.`,
	},
	description2: {
		id: 'project.settings.disclosures.description.2',
		defaultMessage: `Unsure how to apply content disclosure to your project? Check out our <faq-link>Content Disclosures FAQ</faq-link>.`,
	},
	noPermission: {
		id: 'project.settings.disclosures.save-blocked.no-permission',
		defaultMessage: `You don't have permission to edit this project's disclosures.`,
	},
	uploadVersionFirstHeading: {
		id: 'project.settings.disclosures.upload-version-first.heading',
		defaultMessage: 'Upload versions before adding disclosures',
	},
})

const issueMessages = defineMessages({
	'advertising-note': {
		id: 'project.settings.disclosures.save-blocked.advertising-note',
		defaultMessage: 'Advertising disclosure requires explanation.',
	},
	'paid-features-empty': {
		id: 'project.settings.disclosures.save-blocked.paid-features-empty',
		defaultMessage: 'Paid features disclosure must list at least one paid feature.',
	},
	'telemetry-empty': {
		id: 'project.settings.disclosures.save-blocked.telemetry-empty',
		defaultMessage: 'Telemetry disclosure must list at least one type of data collected.',
	},
	'derivative-empty': {
		id: 'project.settings.disclosures.save-blocked.derivative-empty',
		defaultMessage: 'Derivative works disclosure must list at least one source work.',
	},
	'derivative-source-label': {
		id: 'project.settings.disclosures.save-blocked.derivative-source-label',
		defaultMessage: 'Derivative work sources must have a name.',
	},
	'photosensitivity-note': {
		id: 'project.settings.disclosures.save-blocked.photosensitivity-note',
		defaultMessage: 'Photosensitivity warning disclosure must include a description.',
	},
	'system-interactions-note': {
		id: 'project.settings.disclosures.save-blocked.system-interactions-note',
		defaultMessage: 'External system interactions disclosure must include a description.',
	},
}) satisfies Record<DisclosureFormIssue, MessageDescriptor>

useProjectSettingsHeadTitle(messages.title)

const disclosuresQueryKey = computed(
	() => ['project', 'disclosures', 'v3', project.value.id] as const,
)

const { data: disclosuresResponse } = useQuery({
	queryKey: disclosuresQueryKey,
	queryFn: () => labrinth.projects_v3.getDisclosures(project.value.id),
	staleTime: DISCLOSURE_QUERY_STALE_TIME,
})

const updaterUserIds = computed(() => {
	const ids = new Set<string>()
	for (const disclosure of disclosuresResponse.value?.disclosures ?? []) {
		if (disclosure.updated_by) {
			ids.add(disclosure.updated_by)
		}
	}
	return [...ids]
})

const { data: updaterUsers } = useQuery({
	queryKey: computed(() => ['users', 'disclosures', updaterUserIds.value] as const),
	queryFn: () => labrinth.users_v2.getMultiple(updaterUserIds.value),
	enabled: computed(() => updaterUserIds.value.length > 0),
	staleTime: DISCLOSURE_QUERY_STALE_TIME,
})

const updaterUsersById = computed(() => {
	const map = new Map<string, NonNullable<typeof updaterUsers.value>[number]>()
	for (const user of updaterUsers.value ?? []) {
		map.set(user.id, user)
	}
	return map
})

const currentUser = computed((): DisclosureUpdatedByUser | null => {
	const user = auth.value?.user
	if (!user?.id || !user.username) {
		return null
	}
	return {
		id: user.id,
		username: user.username,
		avatar_url: user.avatar_url,
	}
})

function resolveUpdatedBy(userId: string | null | undefined): DisclosureUpdatedByUser | null {
	if (!userId) return null
	return (
		updaterUsersById.value.get(userId) ??
		(currentUser.value?.id === userId ? currentUser.value : null)
	)
}

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(
	() =>
		isAdminUser.value ||
		!!((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
)

const {
	saved,
	current,
	saving,
	reset: resetForm,
	save: saveForm,
} = useSavable(
	() => disclosuresToForm(disclosuresResponse.value?.disclosures ?? []),
	async () => {
		if (!hasPermission.value || !canSave.value) {
			throw new Error('Disclosures form is not valid')
		}

		const previous = disclosuresToForm(disclosuresResponse.value?.disclosures ?? [])
		const requests = toModifyRequests(current.value, previous)

		for (const request of requests) {
			await labrinth.projects_v3.modifyDisclosures(project.value.id, request)
		}

		await queryClient.invalidateQueries({ queryKey: disclosuresQueryKey.value })
	},
)

const savedSnapshot = computed(() => getDisclosureFormSnapshot(saved.value))
const currentSnapshot = computed(() => getDisclosureFormSnapshot(current.value))
const hasChanges = computed(
	() => JSON.stringify(savedSnapshot.value) !== JSON.stringify(currentSnapshot.value),
)

const saveValidation = useProjectSaveValidation(() => currentSnapshot.value)

async function save() {
	if (!hasChanges.value || !canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveForm()
		saveValidation.clear()
		await refreshProjectValidation()
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	}
}

function reset() {
	resetForm()
	saveValidation.clear()
}

function disclosureUpdateProps(type: DisclosureType) {
	const disclosure = findDisclosureData(disclosuresResponse.value?.disclosures, type)
	const savedLockStatus = disclosure?.lock_status ?? 'unlocked'
	const lockStatus = current.value.lockStatuses[type] ?? savedLockStatus
	const lockedForAuthor = !isActingAsModerator.value

	return {
		disabled:
			saving.value ||
			!hasPermission.value ||
			(lockedForAuthor && savedLockStatus === 'fully_locked'),
		toggleDisabled: saving.value || (lockedForAuthor && savedLockStatus !== 'unlocked'),
		updatedAt: disclosure?.updated_at,
		updatedBy: resolveUpdatedBy(disclosure?.updated_by),
		setByModerator: !!disclosure?.set_by_moderator,
		lockStatus,
		showLockControls: isActingAsModerator.value,
	}
}

function setDisclosureLockStatus(type: DisclosureType, lockStatus: DisclosureLockStatus) {
	current.value.lockStatuses = {
		...current.value.lockStatuses,
		[type]: lockStatus,
	}
}

watch(
	() => disclosuresResponse.value,
	(value, previous) => {
		if (value && (!previous || !hasChanges.value)) {
			reset()
		}
	},
)

const issues = computed(() => getDisclosureFormIssues(current.value, projectTypes.value))
const disclosureTextValidation = useProjectNagMessages('disclosure-text')
const disclosureValidation = useProjectNagMessages('disclosures')

const canSave = computed(
	() =>
		!saveValidation.hasErrors.value &&
		hasPermission.value &&
		(isAdminUser.value || issues.value.length === 0),
)

const saveDisabledReason = computed(() => {
	if (!hasPermission.value) {
		// should never come up but y'never know
		return formatMessage(messages.noPermission)
	}
	return [...issues.value.map((issue) => formatMessage(issueMessages[issue]))]
})

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)
</script>

<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<h2 class="m-0 text-2xl font-semibold">
			{{ formatMessage(messages.title) }}
		</h2>
		<p class="mb-0 mt-2">
			<IntlFormatted :message-id="messages.description">
				<template #rules="{ children }">
					<nuxt-link to="/legal/rules" target="_blank" class="underline hover:text-contrast">
						<component :is="() => normalizeChildren(children)" />
					</nuxt-link>
				</template>
			</IntlFormatted>
		</p>
		<p class="mb-4 mt-2">
			<IntlFormatted :message-id="messages.description2">
				<template #faq-link="{ children }">
					<a
						href="https://support.modrinth.com/en/articles/16567675#h_29503820b1"
						target="_blank"
						class="underline hover:text-contrast"
					>
						<component :is="() => normalizeChildren(children)" />
					</a>
				</template>
			</IntlFormatted>
		</p>
		<ValidationMessage
			:check="[...disclosureValidation, ...disclosureTextValidation]"
			:project-field="JSON.stringify(savedSnapshot)"
			:current-field="JSON.stringify(currentSnapshot)"
			class="mb-4"
		/>
		<EmptyState
			v-if="!canEditDisclosures"
			type="no-documents"
			:heading="formatMessage(messages.uploadVersionFirstHeading)"
			:description="formatMessage(commonMessages.uploadVersionsEmptyStateDescription)"
		/>
		<template v-else>
			<div class="flex flex-col gap-4">
				<AiDisclosureCard
					v-if="isDisclosureVisible('ai_content')"
					v-model="current.ai"
					v-bind="disclosureUpdateProps('ai_content')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('ai_content', status)
					"
				/>
				<AiFunctionalityDisclosureCard
					v-if="isDisclosureVisible('ai_functionality')"
					v-model="current.aiFunctionality"
					v-bind="disclosureUpdateProps('ai_functionality')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('ai_functionality', status)
					"
				/>
				<AdvertisingDisclosureCard
					v-if="isDisclosureVisible('advertisements')"
					v-model="current.advertising"
					v-bind="disclosureUpdateProps('advertisements')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('advertisements', status)
					"
				/>
				<PaidFeaturesDisclosureCard
					v-if="isDisclosureVisible('paid_features')"
					v-model="current.paidFeatures"
					v-bind="disclosureUpdateProps('paid_features')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('paid_features', status)
					"
				/>
				<TelemetryDisclosureCard
					v-if="isDisclosureVisible('telemetry')"
					v-model="current.telemetry"
					v-bind="disclosureUpdateProps('telemetry')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('telemetry', status)
					"
				/>
				<DerivativeDisclosureCard
					v-if="isDisclosureVisible('derivative_work')"
					v-model="current.derivative"
					v-bind="disclosureUpdateProps('derivative_work')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('derivative_work', status)
					"
				/>
				<PhotosensitivityDisclosureCard
					v-if="isDisclosureVisible('epilepsy_triggers')"
					v-model="current.photosensitivity"
					v-bind="disclosureUpdateProps('epilepsy_triggers')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('epilepsy_triggers', status)
					"
				/>
				<SystemInteractionsDisclosureCard
					v-if="isDisclosureVisible('system_interactions')"
					v-model="current.systemInteractions"
					v-bind="disclosureUpdateProps('system_interactions')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('system_interactions', status)
					"
				/>
				<ArchivedDisclosureCard
					v-if="isDisclosureVisible('archived')"
					v-model="current.archived"
					:project-title="project.title"
					v-bind="disclosureUpdateProps('archived')"
					@set-lock-status="
						(status: DisclosureLockStatus) => setDisclosureLockStatus('archived', status)
					"
				/>
			</div>
			<ValidationMessage :check="saveValidation.messages.value" class="my-4" />
			<UnsavedChangesPopup
				:original="savedSnapshot"
				:modified="currentSnapshot"
				:saving="saving"
				:can-save="canSave"
				:save-disabled-reason="saveDisabledReason"
				@reset="reset"
				@save="save"
			/>
		</template>
	</div>
</template>
