<template>
	<div>
		<NewModal
			ref="modalSubmit"
			:header="
				formatMessage(
					isRejected(project)
						? messages.resubmitModalHeaderResubmitting
						: messages.resubmitModalHeaderSubmitting,
				)
			"
		>
			<div class="flex max-w-[35rem] flex-col gap-3">
				<p class="m-0">
					<IntlFormatted
						:message-id="messages.resubmitModalDescription"
						:values="{ projectTitle: project.title }"
					>
						<template #project-title="{ children }">
							<span class="font-semibold text-contrast">
								<component :is="() => children" />
							</span>
						</template>
					</IntlFormatted>
				</p>
				<p class="m-0">{{ formatMessage(messages.resubmitModalReminder) }}</p>
				<p class="m-0 font-semibold text-red">
					{{ formatMessage(messages.resubmitModalWarning) }}
				</p>
				<Checkbox
					v-model="submissionConfirmation"
					:description="formatMessage(messages.resubmitModalConfirmationDescription)"
				>
					{{ formatMessage(messages.resubmitModalConfirmationLabel) }}
				</Checkbox>
				<div class="flex flex-wrap items-center justify-end gap-2">
					<Button type="outlined" @click="modalSubmit.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="orange"
						:disabled="!submissionConfirmation || isLoading || reviewSubmissionDisabled"
						@click="runBlockingAction('resubmit-modal', resubmit)"
					>
						<SpinnerIcon
							v-if="loadingAction === 'resubmit-modal'"
							class="animate-spin"
							aria-hidden="true"
						/>
						<ScaleIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.actionResubmitForReview) }}
					</Button>
				</div>
			</div>
		</NewModal>
		<NewModal ref="modalReply" :header="formatMessage(messages.replyModalHeader)">
			<div class="flex max-w-[45rem] flex-col gap-3">
				<p class="m-0">{{ formatMessage(messages.replyModalDescription) }}</p>
				<p class="m-0">
					<IntlFormatted :message-id="messages.replyModalHelpCenterNote">
						<template #help-center-link="{ children }">
							<a class="text-link" href="https://support.modrinth.com" target="_blank">
								<component :is="() => children" />
							</a>
						</template>
					</IntlFormatted>
				</p>
				<Checkbox
					v-model="replyConfirmation"
					:description="formatMessage(messages.replyModalConfirmationDescription)"
				>
					{{ formatMessage(messages.replyModalConfirmationLabel) }}
				</Checkbox>
				<div class="flex flex-wrap items-center justify-end gap-2">
					<Button type="outlined" @click="modalReply.hide()">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</Button>
					<Button
						type="colored"
						color="brand"
						:disabled="!replyConfirmation || isLoading"
						@click="runBlockingAction('reply-modal', () => sendReplyFromModal())"
					>
						<SpinnerIcon
							v-if="loadingAction === 'reply-modal'"
							class="animate-spin"
							aria-hidden="true"
						/>
						<ReplyIcon v-else aria-hidden="true" />
						{{ formatMessage(messages.actionReplyToThread) }}
					</Button>
				</div>
			</div>
		</NewModal>
		<div v-if="flags.showThreadIds" class="mx-4 mb-3 font-semibold">
			Thread ID:
			<CopyCode :text="thread.id" />
		</div>
		<div v-bind="$attrs" class="flex flex-col">
			<div v-if="sortedMessages.length > 0" class="flex flex-col pt-2">
				<ThreadMessage
					v-for="message in sortedMessages"
					:key="'message-' + message.id"
					:thread="thread"
					:message="message"
					:members="members"
					:report="report"
					:auth="auth"
					raised
					@update-thread="() => updateThreadLocal()"
				/>
			</div>
			<div v-if="report && report.closed" class="m-4 mt-2 flex flex-col gap-4">
				<p class="m-0">{{ formatMessage(messages.closedThreadDescription) }}</p>
				<Button
					v-if="isStaff(auth.user)"
					:disabled="isLoading"
					class="w-fit"
					@click="runBlockingAction('reopen', () => reopenReport())"
				>
					<SpinnerIcon v-if="loadingAction === 'reopen'" class="animate-spin" aria-hidden="true" />
					<CheckCircleIcon v-else aria-hidden="true" />
					{{ formatMessage(messages.actionReopenThread) }}
				</Button>
			</div>
			<template v-else-if="!report || !report.closed">
				<div class="mx-4 mb-2 mt-2">
					<MarkdownEditor
						v-model="replyBody"
						:placeholder="
							formatMessage(
								sortedMessages.length > 0
									? messages.replyEditorPlaceholderReply
									: messages.replyEditorPlaceholderSend,
							)
						"
						:on-image-upload="onUploadImage"
					/>
				</div>
				<div class="m-4 mt-3 flex flex-wrap items-center justify-between gap-4">
					<div class="flex flex-wrap items-center gap-2">
						<Button
							v-if="sortedMessages.length > 0"
							type="colored"
							color="brand"
							:disabled="!replyBody || isLoading"
							@click="
								isApproved(project) && !isStaff(auth.user)
									? openReplyModal()
									: runBlockingAction('reply', () => sendReply())
							"
						>
							<SpinnerIcon
								v-if="loadingAction === 'reply'"
								class="animate-spin"
								aria-hidden="true"
							/>
							<ReplyIcon v-else aria-hidden="true" />
							{{ formatMessage(messages.actionReply) }}
						</Button>
						<Button
							v-else
							:disabled="!replyBody || isLoading"
							@click="
								isApproved(project) && !isStaff(auth.user)
									? openReplyModal()
									: runBlockingAction('send', () => sendReply())
							"
						>
							<SpinnerIcon
								v-if="loadingAction === 'send'"
								class="animate-spin"
								aria-hidden="true"
							/>
							<SendIcon v-else aria-hidden="true" />
							{{ formatMessage(messages.actionSend) }}
						</Button>
						<Button
							v-if="isStaff(auth.user)"
							:disabled="!replyBody || isLoading"
							@click="runBlockingAction('private-note', () => sendReply(null, true))"
						>
							<SpinnerIcon
								v-if="loadingAction === 'private-note'"
								class="animate-spin"
								aria-hidden="true"
							/>
							<StickyNotePlusIcon v-else aria-hidden="true" />
							{{ formatMessage(messages.actionAddPrivateNote) }}
						</Button>
						<template v-if="currentMember && !currentMember.staffOnly">
							<template v-if="isRejected(project)">
								<Button
									v-if="replyBody"
									type="colored"
									color="orange"
									:disabled="isLoading || reviewSubmissionDisabled"
									@click="openResubmitModal(true)"
								>
									<ScaleIcon aria-hidden="true" />
									{{ formatMessage(messages.actionResubmitForReviewWithReply) }}
								</Button>
								<Button v-else :disabled="isLoading || reviewSubmissionDisabled" @click="openResubmitModal(false)">
									<ScaleIcon aria-hidden="true" />
									{{ formatMessage(messages.actionResubmitForReview) }}
								</Button>
							</template>
						</template>
					</div>
					<div class="flex flex-wrap items-center gap-2">
						<template v-if="report">
							<Button
								v-if="isStaff(auth.user) && replyBody"
								type="colored"
								color="red"
								:disabled="isLoading"
								@click="runBlockingAction('close-with-reply', () => closeReport(true))"
							>
								<SpinnerIcon
									v-if="loadingAction === 'close-with-reply'"
									class="animate-spin"
									aria-hidden="true"
								/>
								<CheckCircleIcon v-else aria-hidden="true" />
								{{ formatMessage(messages.actionCloseWithReply) }}
							</Button>
							<Button
								v-else
								:disabled="isLoading"
								@click="runBlockingAction('close', () => closeReport())"
							>
								<SpinnerIcon
									v-if="loadingAction === 'close'"
									class="animate-spin"
									aria-hidden="true"
								/>
								<CheckCircleIcon v-else aria-hidden="true" />
								{{ formatMessage(messages.actionCloseThread) }}
							</Button>
						</template>
						<template v-if="project">
							<template v-if="isStaff(auth.user)">
								<Button
									v-if="replyBody"
									type="colored"
									color="green"
									:disabled="isApproved(project) || isLoading"
									@click="runBlockingAction('approve-with-reply', () => sendReply(requestedStatus))"
								>
									<SpinnerIcon
										v-if="loadingAction === 'approve-with-reply'"
										class="animate-spin"
										aria-hidden="true"
									/>
									<CheckIcon v-else aria-hidden="true" />
									{{ formatMessage(messages.actionApproveWithReply) }}
								</Button>
								<Button
									v-else
									type="colored"
									color="green"
									:disabled="isApproved(project) || isLoading"
									@click="runBlockingAction('approve', () => setStatus(requestedStatus))"
								>
									<SpinnerIcon
										v-if="loadingAction === 'approve'"
										class="animate-spin"
										aria-hidden="true"
									/>
									<CheckIcon v-else aria-hidden="true" />
									{{ formatMessage(messages.actionApprove) }}
								</Button>
								<SplitButton
									type="colored"
									color="red"
									:menu-label="formatMessage(commonMessages.moreOptionsButton)"
									:disabled="isLoading"
									:primary-disabled="project.status === 'rejected'"
									:options="
										replyBody
											? [
													{
														id: 'withhold-reply',
														label: formatMessage(messages.actionWithholdWithReply),
														tone: 'orange',
														hoverFilled: true,
														action: () =>
															runBlockingAction('withhold-reply', () => sendReply('withheld')),
														disabled: project.status === 'withheld' || isLoading,
													},
													{
														id: 'set-to-draft-reply',
														label: formatMessage(messages.actionSetToDraftWithReply),
														tone: 'orange',
														hoverFilled: true,
														action: () =>
															runBlockingAction('set-to-draft-reply', () => sendReply('draft')),
														disabled: project.status === 'draft' || isLoading,
													},
													{
														id: 'send-to-review-reply',
														label: formatMessage(messages.actionSendToReviewWithReply),
														tone: 'orange',
														hoverFilled: true,
														action: () =>
															runBlockingAction('send-to-review-reply', () =>
																sendReply('processing', true),
															),
														disabled: project.status === 'processing' || isLoading || reviewSubmissionDisabled,
													},
												]
											: [
													{
														id: 'withhold',
														label: formatMessage(messages.actionWithhold),
														tone: 'orange',
														hoverFilled: true,
														action: () =>
															runBlockingAction('withhold', () => setStatus('withheld')),
														disabled: project.status === 'withheld' || isLoading,
													},
													{
														id: 'set-to-draft',
														label: formatMessage(messages.actionSetToDraft),
														tone: 'orange',
														hoverFilled: true,
														action: () =>
															runBlockingAction('set-to-draft', () => setStatus('draft')),
														disabled: project.status === 'draft' || isLoading,
													},
													{
														id: 'send-to-review',
														label: formatMessage(messages.actionSendToReview),
														tone: 'orange',
														hoverFilled: true,
														action: () =>
															runBlockingAction('send-to-review', () => setStatus('processing')),
														disabled: project.status === 'processing' || isLoading || reviewSubmissionDisabled,
													},
												]
									"
									@click="
										replyBody
											? runBlockingAction('reject-with-reply', () => sendReply('rejected'))
											: runBlockingAction('reject', () => setStatus('rejected'))
									"
								>
									<SpinnerIcon
										v-if="loadingAction === 'reject-with-reply' || loadingAction === 'reject'"
										class="animate-spin"
										aria-hidden="true"
									/>
									<XIcon v-else aria-hidden="true" />
									{{
										formatMessage(
											replyBody ? messages.actionRejectWithReply : messages.actionReject,
										)
									}}
									<template #withhold-reply>
										<EyeOffIcon aria-hidden="true" />
										{{ formatMessage(messages.actionWithholdWithReply) }}
									</template>
									<template #withhold>
										<EyeOffIcon aria-hidden="true" />
										{{ formatMessage(messages.actionWithhold) }}
									</template>
									<template #set-to-draft-reply>
										<FileTextIcon aria-hidden="true" />
										{{ formatMessage(messages.actionSetToDraftWithReply) }}
									</template>
									<template #set-to-draft>
										<FileTextIcon aria-hidden="true" />
										{{ formatMessage(messages.actionSetToDraft) }}
									</template>
									<template #send-to-review-reply>
										<ScaleIcon aria-hidden="true" />
										{{ formatMessage(messages.actionSendToReviewWithReply) }}
									</template>
									<template #send-to-review>
										<ScaleIcon aria-hidden="true" />
										{{ formatMessage(messages.actionSendToReview) }}
									</template>
								</SplitButton>
							</template>
						</template>
					</div>
				</div>
			</template>
		</div>
	</div>
</template>

<script setup>
import {
	CheckCircleIcon,
	CheckIcon,
	EyeOffIcon,
	FileTextIcon,
	ReplyIcon,
	ScaleIcon,
	SendIcon,
	SpinnerIcon,
	StickyNotePlusIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Button,
	Checkbox,
	commonMessages,
	CopyCode,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	MarkdownEditor,
	NewModal,
	SplitButton,
	useVIntl,
} from '@modrinth/ui'

import ThreadMessage from '~/components/ui/thread/ThreadMessage.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { isApproved, isRejected } from '~/helpers/projects.js'
import { isStaff } from '~/helpers/users.js'

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	resubmitModalHeaderResubmitting: {
		id: 'conversation-thread.resubmit-modal.header.resubmitting',
		defaultMessage: 'Resubmitting for review',
	},
	resubmitModalHeaderSubmitting: {
		id: 'conversation-thread.resubmit-modal.header.submitting',
		defaultMessage: 'Submitting for review',
	},
	resubmitModalDescription: {
		id: 'conversation-thread.resubmit-modal.description',
		defaultMessage:
			"You're submitting <project-title>{projectTitle}</project-title> to be reviewed again by the moderators.",
	},
	resubmitModalReminder: {
		id: 'conversation-thread.resubmit-modal.reminder',
		defaultMessage: 'Make sure you have addressed all the comments from the moderation team.',
	},
	resubmitModalWarning: {
		id: 'conversation-thread.resubmit-modal.warning',
		defaultMessage:
			"Repeated submissions without addressing the moderators' comments may result in an account suspension.",
	},
	resubmitModalConfirmationDescription: {
		id: 'conversation-thread.resubmit-modal.confirmation.description',
		defaultMessage: 'Confirm I have addressed the messages from the moderators',
	},
	resubmitModalConfirmationLabel: {
		id: 'conversation-thread.resubmit-modal.confirmation.label',
		defaultMessage: "I confirm that I have properly addressed the moderators' comments.",
	},
	replyModalHeader: {
		id: 'conversation-thread.reply-modal.header',
		defaultMessage: 'Reply to thread',
	},
	replyModalDescription: {
		id: 'conversation-thread.reply-modal.description',
		defaultMessage:
			'Your project is already approved. As such, the moderation team does not actively monitor this thread. However, they may still see your message if there is a problem with your project.',
	},
	replyModalHelpCenterNote: {
		id: 'conversation-thread.reply-modal.help-center-note',
		defaultMessage:
			'If you need to get in contact with the moderation team, please use the <help-center-link>Modrinth Help Center</help-center-link> and click the blue bubble in the bottom right corner to contact support.',
	},
	replyModalConfirmationDescription: {
		id: 'conversation-thread.reply-modal.confirmation.description',
		defaultMessage: 'Confirm moderators do not actively monitor this',
	},
	replyModalConfirmationLabel: {
		id: 'conversation-thread.reply-modal.confirmation.label',
		defaultMessage: 'I acknowledge that the moderators do not actively monitor the thread.',
	},
	closedThreadDescription: {
		id: 'conversation-thread.closed-thread.description',
		defaultMessage: 'This thread is closed and new messages cannot be sent to it.',
	},
	replyEditorPlaceholderReply: {
		id: 'conversation-thread.reply-editor.placeholder.reply',
		defaultMessage: 'Reply to thread...',
	},
	replyEditorPlaceholderSend: {
		id: 'conversation-thread.reply-editor.placeholder.send',
		defaultMessage: 'Send a message...',
	},
	actionResubmitForReview: {
		id: 'conversation-thread.action.resubmit-for-review',
		defaultMessage: 'Resubmit for review',
	},
	actionReplyToThread: {
		id: 'conversation-thread.action.reply-to-thread',
		defaultMessage: 'Reply to thread',
	},
	actionReopenThread: {
		id: 'conversation-thread.action.reopen-thread',
		defaultMessage: 'Reopen thread',
	},
	actionReply: {
		id: 'conversation-thread.action.reply',
		defaultMessage: 'Reply',
	},
	actionSend: {
		id: 'conversation-thread.action.send',
		defaultMessage: 'Send',
	},
	actionAddPrivateNote: {
		id: 'conversation-thread.action.add-private-note',
		defaultMessage: 'Add private note',
	},
	actionResubmitForReviewWithReply: {
		id: 'conversation-thread.action.resubmit-for-review-with-reply',
		defaultMessage: 'Resubmit for review with reply',
	},
	actionCloseWithReply: {
		id: 'conversation-thread.action.close-with-reply',
		defaultMessage: 'Close with reply',
	},
	actionCloseThread: {
		id: 'conversation-thread.action.close-report',
		defaultMessage: 'Close report',
	},
	actionApproveWithReply: {
		id: 'conversation-thread.action.approve-with-reply',
		defaultMessage: 'Approve with reply',
	},
	actionApprove: {
		id: 'conversation-thread.action.approve',
		defaultMessage: 'Approve',
	},
	actionRejectWithReply: {
		id: 'conversation-thread.action.reject-with-reply',
		defaultMessage: 'Reject with reply',
	},
	actionReject: {
		id: 'conversation-thread.action.reject',
		defaultMessage: 'Reject',
	},
	actionWithholdWithReply: {
		id: 'conversation-thread.action.withhold-with-reply',
		defaultMessage: 'Withhold with reply',
	},
	actionWithhold: {
		id: 'conversation-thread.action.withhold',
		defaultMessage: 'Withhold',
	},
	actionSetToDraftWithReply: {
		id: 'conversation-thread.action.set-to-draft-with-reply',
		defaultMessage: 'Set to draft with reply',
	},
	actionSetToDraft: {
		id: 'conversation-thread.action.set-to-draft',
		defaultMessage: 'Set to draft',
	},
	actionSendToReviewWithReply: {
		id: 'conversation-thread.action.send-to-review-with-reply',
		defaultMessage: 'Send to review with reply',
	},
	actionSendToReview: {
		id: 'conversation-thread.action.send-to-review',
		defaultMessage: 'Send to review',
	},
	errorSendingMessage: {
		id: 'conversation-thread.error.sending-message',
		defaultMessage: 'Error sending message',
	},
	errorClosingReport: {
		id: 'conversation-thread.error.closing-report',
		defaultMessage: 'Error closing report',
	},
	errorReopeningReport: {
		id: 'conversation-thread.error.reopening-report',
		defaultMessage: 'Error reopening report',
	},
})

const props = defineProps({
	reviewSubmissionDisabled: {
		type: Boolean,
		default: false,
	},
	thread: {
		type: Object,
		required: true,
	},
	report: {
		type: Object,
		required: false,
		default: null,
	},
	project: {
		type: Object,
		required: false,
		default: null,
	},
	setStatus: {
		type: Function,
		required: false,
		default: () => {},
	},
	currentMember: {
		type: Object,
		default() {
			return null
		},
	},
	auth: {
		type: Object,
		required: true,
	},
})

const emit = defineEmits(['update-thread'])

const app = useNuxtApp()
const flags = useFeatureFlags()

const members = computed(() => {
	const members = {}
	for (const member of props.thread.members) {
		members[member.id] = member
	}
	return members
})

const replyBody = ref('')

const sortedMessages = computed(() => {
	if (props.thread !== null) {
		return props.thread.messages
			.slice()
			.sort((a, b) => app.$dayjs(a.created) - app.$dayjs(b.created))
	}
	return []
})

const modalSubmit = ref(null)
const modalReply = ref(null)

const loadingAction = ref(null)
const isLoading = computed(() => loadingAction.value !== null)

async function runBlockingAction(actionId, action) {
	if (loadingAction.value !== null) {
		return
	}
	loadingAction.value = actionId
	try {
		await action()
	} finally {
		loadingAction.value = null
	}
}

async function updateThreadLocal() {
	let threadId = null
	if (props.project) {
		threadId = props.project.thread_id
	} else if (props.report) {
		threadId = props.report.thread_id
	}
	let thread = null
	if (threadId) {
		thread = await useBaseFetch(`thread/${threadId}`)
	}
	emit('update-thread', thread)
}

const imageIDs = ref([])

async function onUploadImage(file) {
	const response = await useImageUpload(file, { context: 'thread_message' })

	imageIDs.value.push(response.id)
	// Keep the last 10 entries of image IDs
	imageIDs.value = imageIDs.value.slice(-10)

	return response.url
}

async function sendReplyFromModal(status = null, privateMessage = false) {
	await sendReply(status, privateMessage)
	modalReply.value.hide()
}

async function sendReply(status = null, privateMessage = false) {
	if (status === 'processing' && props.reviewSubmissionDisabled) return
	try {
		const body = {
			body: {
				type: 'text',
				body: replyBody.value,
				private: privateMessage,
			},
		}

		if (imageIDs.value.length > 0) {
			body.body = {
				...body.body,
				uploaded_images: imageIDs.value,
			}
		}

		await useBaseFetch(`thread/${props.thread.id}`, {
			method: 'POST',
			body,
		})

		replyBody.value = ''

		await updateThreadLocal()
		if (status !== null) {
			await props.setStatus(status)
		}
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorSendingMessage),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

async function closeReport(reply) {
	if (reply) {
		await sendReply()
	}

	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: true,
			},
		})
		await updateThreadLocal()
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorClosingReport),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

async function reopenReport() {
	try {
		await useBaseFetch(`report/${props.report.id}`, {
			method: 'PATCH',
			body: {
				closed: false,
			},
		})
		await updateThreadLocal()
	} catch (err) {
		addNotification({
			title: formatMessage(messages.errorReopeningReport),
			text: err.data ? err.data.description : err,
			type: 'error',
		})
	}
}

const replyWithSubmission = ref(false)
const submissionConfirmation = ref(false)
const replyConfirmation = ref(false)

function openResubmitModal(reply) {
	submissionConfirmation.value = false
	replyWithSubmission.value = reply
	modalSubmit.value.show()
}

function openReplyModal() {
	replyConfirmation.value = false
	modalReply.value.show()
}

async function resubmit() {
	if (props.reviewSubmissionDisabled) return
	if (replyWithSubmission.value) {
		await sendReply('processing')
	} else {
		await props.setStatus('processing')
	}
	modalSubmit.value.hide()
}

const requestedStatus = computed(() => props.project.requested_status ?? 'approved')

defineOptions({
	inheritAttrs: false,
})
</script>
