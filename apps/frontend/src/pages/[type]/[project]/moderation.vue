<template>
	<template v-if="canAccess">
		<Admonition
			v-if="userFacingUiVisible && moderationAdmonition"
			:type="moderationAdmonition.type"
			class="mb-4"
			:header="formatMessage(moderationAdmonition.header)"
		>
			<template
				v-for="(section, index) in moderationAdmonition.body"
				:key="`moderation-admonition.${project.status}+${project.requested_status ?? 'none'}.body.${index}`"
			>
				<p
					v-if="section.type === 'paragraph' && section.message"
					class="preserve-lines mb-0 mt-2 leading-tight first:mt-0"
				>
					<IntlFormatted
						:message-id="section.message"
						:values="{
							requestedStatus: project.requested_status ?? 'none',
						}"
					>
						<template #rules-link="{ children }">
							<nuxt-link to="/legal/rules" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
						<template #terms-link="{ children }">
							<nuxt-link to="/legal/terms" class="text-link" target="_blank">
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
						<template #visibility-settings-link="{ children }">
							<router-link :to="`${getProjectLink(project)}/settings#visibility`" class="text-link">
								<component :is="() => normalizeChildren(children)" />
							</router-link>
						</template>
						<template #emphasis="{ children }">
							<span class="font-semibold">
								<component :is="() => normalizeChildren(children)" />
							</span>
						</template>
					</IntlFormatted>
				</p>
				<ul
					v-else-if="section.type === 'bullets'"
					class="mb-0 mt-2 flex list-disc flex-col gap-1 pl-4 leading-normal first:mt-0"
				>
					<li
						v-for="(message, listIndex) in section.items"
						:key="`list-item-${index}-${listIndex}`"
					>
						<IntlFormatted :message-id="message">
							<template #rules-link="{ children }">
								<nuxt-link to="/legal/rules" class="text-link" target="_blank">
									<component :is="() => normalizeChildren(children)" />
								</nuxt-link>
							</template>
							<template #terms-link="{ children }">
								<nuxt-link to="/legal/terms" class="text-link" target="_blank">
									<component :is="() => normalizeChildren(children)" />
								</nuxt-link>
							</template>
						</IntlFormatted>
					</li>
				</ul>
			</template>
		</Admonition>
		<div class="card-shadow mb-6 rounded-2xl border border-solid border-surface-4 bg-surface-3">
			<div class="flex flex-col p-4">
				<div class="flex items-center justify-between">
					<h2 id="messages" class="m-0 text-xl font-semibold text-contrast">
						{{ formatMessage(messages.threadSectionTitle) }}
					</h2>
					<div v-if="staff" class="flex items-center gap-2">
						<Toggle id="moderator-see-user-ui-toggle" v-model="moderatorSeeUserUi" small />
						<label for="moderator-see-user-ui-toggle"> Show member UI </label>
					</div>
				</div>
				<template v-if="userFacingUiVisible">
					<p class="m-0 mt-2 leading-tight">
						{{ formatMessage(messages.threadPrivateDescription) }}
					</p>
					<p class="mb-0 mt-3 leading-tight">
						<IntlFormatted :message-id="messages.threadHelpCenterNote1">
							<template #help-center-link="{ children }">
								<a class="text-link" href="https://support.modrinth.com" target="_blank">
									<component :is="() => normalizeChildren(children)" />
								</a>
							</template>
						</IntlFormatted>
					</p>
					<p class="mb-0 mt-2 leading-tight">
						<IntlFormatted :message-id="messages.threadHelpCenterNote2">
							<template #help-center-link="{ children }">
								<a class="text-link" href="https://support.modrinth.com" target="_blank">
									<component :is="() => normalizeChildren(children)" />
								</a>
							</template>
						</IntlFormatted>
					</p>
					<p
						v-if="isApproved(project)"
						class="mb-0 mt-3 flex items-center gap-2 font-semibold text-orange"
					>
						<IssuesIcon class="shrink-0" />
						{{ formatMessage(messages.threadApprovedWarning) }}
					</p>
				</template>
			</div>
			<ConversationThread
				v-if="prefixedThread"
				:thread="prefixedThread"
				:project="project"
				:set-status="setStatus"
				:review-submission-disabled="reviewSubmissionDisabled"
				:current-member="currentMember ?? undefined"
				:auth="auth"
				class="overflow-clip rounded-b-2xl border-0 border-t border-solid border-surface-4 bg-surface-2"
				@update-thread="updateThread"
			/>
			<div
				v-else
				class="flex items-center justify-center gap-2 rounded-b-2xl border-0 border-t border-solid border-surface-4 bg-surface-2 py-12"
			>
				<template v-if="pending">
					<SpinnerIcon class="size-5 animate-spin" /> Loading messages
				</template>
				<template v-else>
					<p class="m-0 text-red">Failed to load messages</p>
				</template>
			</div>
		</div>
	</template>
</template>
<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { IssuesIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Admonition,
	commonMessages,
	defineMessage,
	defineMessages,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	type MessageDescriptor,
	normalizeChildren,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useQueryClient } from '@tanstack/vue-query'
import dayjs from 'dayjs'
import { computed, watch } from 'vue'

import ConversationThread from '~/components/ui/thread/ConversationThread.vue'
import { canSubmitProjectForReview } from '~/composables/link-network-validation/submission'
import { getProjectLink, isApproved, isRejected, isUnderReview } from '~/helpers/projects.js'

defineEmits(['on-download', 'delete-version'])

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()

type ModerationAdmonitionSection =
	| {
			type: 'paragraph'
			message: MessageDescriptor | null
	  }
	| {
			type: 'bullets'
			items: MessageDescriptor[]
	  }

const messages = defineMessages({
	admonitionRejectedSpamNotice: {
		id: 'project.moderation.admonition.rejected.spam-notice',
		defaultMessage:
			'Repeatedly submitting your project without addressing all moderation concerns first may result in account suspension.',
	},
	threadSectionTitle: {
		id: 'project.moderation.thread.title',
		defaultMessage: 'Moderation messages',
	},
	threadPrivateDescription: {
		id: 'project.moderation.thread.private-description',
		defaultMessage:
			'This is a private conversation thread with the Modrinth moderators. They may message you with issues concerning this project.',
	},
	threadHelpCenterNote1: {
		id: 'project.moderation.thread.help-center-note.1',
		defaultMessage:
			'Content moderators cannot provide support for most issues and messages to this thread do not notify staff.',
	},
	threadHelpCenterNote2: {
		id: 'project.moderation.thread.help-center-note.2',
		defaultMessage:
			'If you need assistance or have additional inquiries, please visit the <help-center-link>Modrinth Help Center</help-center-link> and click the blue bubble to contact support.',
	},
	threadApprovedWarning: {
		id: 'project.moderation.thread.approved-warning',
		defaultMessage:
			'This thread is not actively monitored, but may be reviewed for information about your project if needed.',
	},
	approvedProjectVisibilityMessage: {
		id: 'project.moderation.admonition.approved.body.visibility-message',
		defaultMessage:
			"You can change the visibility of your project in your project's <visibility-settings-link>visibility settings</visibility-settings-link>.",
	},
})

const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
	setProcessing,
	currentMember,
	invalidate,
	allMembers,
	thread,
} = injectProjectPageContext()

const reviewSubmissionDisabled = computed(
	() =>
		!canSubmitProjectForReview(
			projectValidation.value,
			projectValidationLoading.value || projectLinksNetworkValidationLoading.value,
		),
)

const THREADS_RELEASE_DATE = '2023-08-05T12:00:00-07:00'

const prefixedThread = computed(() => {
	const projectDate = project.value?.queued ?? project.value?.approved ?? project.value?.published
	if (thread.value && projectDate && dayjs(projectDate).isBefore(dayjs(THREADS_RELEASE_DATE))) {
		const newThread = JSON.parse(JSON.stringify(thread.value))
		newThread.messages.unshift({
			id: '69',
			author_id: null,
			body: {
				type: 'legacy_project_message',
			},
			created: THREADS_RELEASE_DATE,
			hide_identity: false,
		})
		return newThread
	}
	return thread.value
})

const canAccess = computed(() => !!currentMember.value)
const staff = computed(() => isStaff(currentMember.value?.user))
const userFacingUiVisible = computed(
	() => !!currentMember.value && (!staff.value || moderatorSeeUserUi.value),
)

const approvedAdmonitionMessage = computed<MessageDescriptor | null>(() => {
	switch (project.value?.status) {
		case 'approved':
		case 'archived':
			return defineMessage({
				id: 'project.moderation.admonition.approved.body.public',
				defaultMessage: 'Your project is published and discoverable on Modrinth.',
			})
		case 'unlisted':
			return defineMessage({
				id: 'project.moderation.admonition.approved.body.unlisted',
				defaultMessage:
					'Your project is unlisted, meaning it can only be accessed with a direct link and is not discoverable on Modrinth.',
			})

		case 'private':
			return defineMessage({
				id: 'project.moderation.admonition.approved.body.private',
				defaultMessage:
					'Your project is private, meaning it can only be accessed by you and people you invite.',
			})
		default:
			return null
	}
})

const moderationAdmonition = computed<{
	type: InstanceType<typeof Admonition>['type']
	header: MessageDescriptor
	body: ModerationAdmonitionSection[]
} | null>(() => {
	const currentProject = project.value

	if (currentProject.status === 'draft') {
		return {
			type: 'info',
			header: defineMessage({
				id: 'project.moderation.admonition.draft.header',
				defaultMessage: 'Draft project',
			}),
			body: [
				{
					type: 'paragraph',
					message: defineMessage({
						id: 'project.moderation.admonition.draft.body',
						defaultMessage:
							"This is a draft project that cannot be seen by others until submitted for review and approved by Modrinth's moderation team.",
					}),
				},
				{
					type: 'paragraph',
					message: defineMessage({
						id: 'project.moderation.admonition.draft.submit-for-review',
						defaultMessage:
							"Once you have completed all required steps and ensured your project complies with Modrinth's <rules-link>Content Rules</rules-link> you can submit your project for review.",
					}),
				},
			],
		}
	}

	if (isApproved(currentProject) && approvedAdmonitionMessage.value) {
		return {
			type: 'success',
			header: defineMessage({
				id: 'project.moderation.admonition.approved.header',
				defaultMessage: 'Project approved',
			}),
			body: [
				{
					type: 'paragraph',
					message: approvedAdmonitionMessage.value,
				},
				{
					type: 'paragraph',
					message: messages.approvedProjectVisibilityMessage,
				},
			],
		}
	}

	if (isUnderReview(currentProject)) {
		return {
			type: 'moderation',
			header: defineMessage({
				id: 'project.moderation.admonition.under-review.header',
				defaultMessage: 'Project under review',
			}),
			body: [
				{
					type: 'paragraph',
					message: defineMessage({
						id: 'project.moderation.admonition.under-review.body.1',
						defaultMessage:
							"Your project is in queue to be reviewed by Modrinth's moderation team.",
					}),
				},
				{
					type: 'bullets',
					items: [
						defineMessage({
							id: 'project.moderation.admonition.under-review.body.2',
							defaultMessage:
								"Your project will be scanned and then reviewed by human moderators to ensure it meets Modrinth's <rules-link>Content Rules</rules-link> and <terms-link>Terms of Use</terms-link>.",
						}),
						defineMessage({
							id: 'project.moderation.admonition.under-review.body.3',
							defaultMessage:
								"You can still modify your project, it won't affect your position in the queue.",
						}),
						// temp moved 24-48 hr below to keep old translation for future
						defineMessage({
							id: 'project.moderation.admonition.under-review.body.4.alt-week',
							defaultMessage:
								'We aim to review submissions within a week, but some projects may face delays. This does not reflect an issue with your submission.',
						}),
					],
				},
				{
					type: 'paragraph',
					message: defineMessage({
						id: 'project.moderation.admonition.under-review.body.5',
						defaultMessage:
							'<emphasis>We appreciate your patience while our moderators work hard to keep Modrinth safe, and look forward to helping you share your content! 💚</emphasis>',
					}),
				},
			],
		}
	}

	if (currentProject.status === 'withheld') {
		return {
			type: 'warning',
			header: defineMessage({
				id: 'project.moderation.admonition.withheld.header',
				defaultMessage: 'Unlisted by staff',
			}),
			body: [
				{
					type: 'paragraph',
					message: defineMessage({
						id: 'project.moderation.admonition.withheld.body',
						defaultMessage:
							'Your project will not appear publicly and can only be accessed with a direct link.{requestedStatus, select, unlisted { Based on your selected <visibility-settings-link>visibility settings</visibility-settings-link>, most likely no action is necessary.} other { Please address all moderation concerns, including any issues listed in messages below before resubmitting this project.}}',
					}),
				},
				{
					type: 'paragraph',
					message: messages.admonitionRejectedSpamNotice,
				},
			],
		}
	}

	if (isRejected(currentProject)) {
		return {
			type: 'critical',
			header: defineMessage({
				id: 'project.moderation.admonition.rejected.header',
				defaultMessage: 'Changes requested',
			}),
			body: [
				{
					type: 'paragraph',
					message: defineMessage({
						id: 'project.moderation.admonition.rejected.address-all-concerns',
						defaultMessage:
							'Please address all moderation concerns, including any issues listed in messages below, before resubmitting this project.',
					}),
				},
				{
					type: 'paragraph',
					message: messages.admonitionRejectedSpamNotice,
				},
			],
		}
	}

	return null
})

// unused 24-48hr message still defined here for later
defineMessage({
	id: 'project.moderation.admonition.under-review.body.4',
	defaultMessage:
		'We aim to review submissions in 24–48 hours, but some projects may face delays. This does not reflect an issue with your submission.',
})

const moderatorSeeUserUi = computed<boolean>({
	get() {
		return flags.value.showModeratorProjectMemberUi
	},
	set(value: boolean) {
		flags.value.showModeratorProjectMemberUi = value
		saveFeatureFlags()
	},
})

watch(
	[currentMember, allMembers],
	() => {
		if (allMembers.value.length > 0 && !canAccess.value) {
			showError({
				fatal: true,
				statusCode: 401,
				statusMessage: formatMessage(
					defineMessage({
						id: 'project.moderation.error.unauthorized',
						defaultMessage: 'Unauthorized',
					}),
				),
			})
		}
	},
	{ flush: 'sync', immediate: true },
)

const auth = await useAuth()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const pending = computed(() => thread.value === undefined)

function updateThread(newThread: Labrinth.Threads.v3.Thread | null | undefined) {
	const threadId = newThread?.id ?? project.value?.thread_id
	if (!threadId) return

	queryClient.setQueryData<Labrinth.Threads.v3.Thread | null | undefined>(
		['thread', threadId],
		newThread,
	)
}

async function setStatus(status: Labrinth.Projects.v2.ProjectStatus) {
	if (status === 'processing') {
		await setProcessing()
		return
	}
	startLoading()

	try {
		await client.labrinth.projects_v2.edit(project.value.id, { status })

		project.value.status = status
		await invalidate()
		await queryClient.invalidateQueries({ queryKey: ['thread', project.value?.thread_id] })
	} catch (err) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: getErrorDescription(err),
			type: 'error',
		})
	}

	stopLoading()
}

function getErrorDescription(err: unknown): string {
	if (typeof err === 'object' && err !== null && 'data' in err) {
		const data = (err as { data?: { description?: string } }).data
		if (data?.description) return data.description
	}

	return err instanceof Error ? err.message : String(err)
}
</script>
