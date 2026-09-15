<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<div class="universal-card">
			<div class="markdown-disclaimer">
				<h2>{{ formatMessage(messages.title) }}</h2>
				<span class="label__description">
					<IntlFormatted :message-id="messages.intro">
						<template #rules="{ children }">
							<NuxtLink class="text-link" target="_blank" to="/legal/rules"
								><component :is="() => children"
							/></NuxtLink>
						</template>
					</IntlFormatted>
				</span>
			</div>
			<MarkdownEditor
				v-model="current.description"
				:disabled="saving || !hasPermission"
				:on-image-upload="onUploadHandler"
			/>
			<ValidationMessage
				:check="descriptionValidation"
				:project-field="saved.description"
				:current-field="current.description"
				class="mt-2"
			/>
			<ValidationMessage :check="saveValidation.forField('description')" class="mt-2" />
		</div>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-save="canSave"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script lang="ts" setup>
import {
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	injectNotificationManager,
	injectProjectPageContext,
	IntlFormatted,
	MarkdownEditor,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { computed, useTemplateRef } from 'vue'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useImageUpload } from '~/composables/image-upload.ts'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { fileDeclaresAi } from '~/helpers/c2pa'

const { projectV2: project, currentMember, patchProjectV3 } = injectProjectPageContext()
const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: { id: 'project.settings.description.title', defaultMessage: 'Description' },
	intro: {
		id: 'project.settings.description.intro',
		defaultMessage:
			'You can type an extended description of your project here. The description must clearly and honestly describe the purpose and function of the project. See section 2.1 of the <rules>Content Rules</rules> for the full requirements.',
	},
	updated: { id: 'project.settings.description.updated', defaultMessage: 'Description updated' },
	updatedText: {
		id: 'project.settings.description.updated-text',
		defaultMessage: 'Your description has been updated.',
	},
})
const aiImageWarningModal = useTemplateRef('aiImageWarningModal')

useProjectSettingsHeadTitle(commonProjectSettingsMessages.description)

const {
	saved,
	current,
	saving,
	hasChanges,
	reset,
	save: saveForm,
} = useSavable(
	() => ({ description: project.value.body }),
	async ({ description }) => {
		await patchProjectV3({ description }, true, true)
	},
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(
	() =>
		isAdminUser.value ||
		(!!currentMember.value &&
			(currentMember.value.permissions & TeamMemberPermission.EDIT_BODY) ===
				TeamMemberPermission.EDIT_BODY),
)
const descriptionValidation = useProjectNagMessages('description', 'description')
const saveValidation = useProjectSaveValidation(() => current.value)
const canSave = computed(
	() =>
		hasPermission.value &&
		!saveValidation.messages.value.some((message) => message.severity === 'error'),
)

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveForm()
		saveValidation.clear()
		addNotification({
			title: formatMessage(messages.updated),
			text: formatMessage(messages.updatedText),
			type: 'success',
		})
	} catch (error) {
		saveValidation.capture(error, submittedState)
	}
}

async function onUploadHandler(file: File) {
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
	const response = await useImageUpload(file, {
		context: 'project',
		projectID: project.value.id,
	})

	return response.url
}
</script>

<style scoped>
.markdown-disclaimer {
	margin-block: 1rem;
}
</style>
