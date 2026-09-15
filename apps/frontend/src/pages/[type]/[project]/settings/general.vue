<script setup lang="ts">
import {
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	IconSelect,
	injectProjectPageContext,
	Input,
	type MessageDescriptor,
	SettingsLabel,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'

import SlugSuggestions from '~/components/ui/SlugSuggestions.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import {
	useProjectSlugSuggestions,
	useSlugSuggestionVisibility,
} from '~/composables/project-slug-suggestions'

const { formatMessage } = useVIntl()

const { allMembers, projectV2: project, patchProject } = injectProjectPageContext()

useProjectSettingsHeadTitle(commonProjectSettingsMessages.general)

const {
	saved,
	current,
	saving,
	hasChanges,
	reset: resetForm,
	save: saveForm,
} = useSavable(
	() => ({
		title: project.value.title,
		tagline: project.value.description,
		url: project.value.slug,
		icon: project.value.icon_url,
	}),
	async ({ title, tagline, url }) => {
		await patchProject(
			{
				...(title !== undefined && { title }),
				...(tagline !== undefined && { description: tagline }),
				...(url !== undefined && { slug: url }),
			},
			false,
			true,
		)
	},
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const titleValidation = useProjectNagMessages('name')
const taglineValidation = useProjectNagMessages('summary')
const iconValidation = useProjectNagMessages('icon')
const saveValidation = useProjectSaveValidation(() => current.value)
const canSave = computed(() => !saveValidation.hasErrors.value)
const {
	onFocusIn: onSlugSuggestionFocusIn,
	onFocusOut: onSlugSuggestionFocusOut,
	visible: showSlugSuggestions,
} = useSlugSuggestionVisibility()
const ownerUsername = computed(
	() => (allMembers.value.find((member) => member.is_owner) ?? allMembers.value[0])?.user.username,
)
const { suggestions: slugSuggestions } = useProjectSlugSuggestions({
	title: () => current.value.title,
	username: ownerUsername,
	currentProjectId: () => project.value.id,
	enabled: showSlugSuggestions,
})

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveForm()
		saveValidation.clear()
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	}
}

function reset() {
	resetForm()
	saveValidation.clear()
}

const messages = defineMessages({
	nameTitle: {
		id: 'project.settings.general.name.title',
		defaultMessage: 'Name',
	},
	nameDescription: {
		id: 'project.settings.general.name.description',
		defaultMessage:
			"Avoid prefixes, suffixes, parentheticals, or added descriptions—just the project's actual name.",
	},
	taglineTitle: {
		id: 'project.settings.general.tagline.title',
		defaultMessage: 'Tagline',
	},
	taglineDescription: {
		id: 'project.settings.general.tagline.description',
		defaultMessage: 'Summarize your project in no more than one sentence.',
	},
	urlTitle: {
		id: 'project.settings.general.url.title',
		defaultMessage: 'URL',
	},
})

const placeholders: { name: MessageDescriptor; tagline: MessageDescriptor }[] = [
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.1',
			defaultMessage: 'e.g. Nether Overhaul 2',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.1',
			defaultMessage: 'e.g. Overhauls game progression to revolve around the Nether.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.2',
			defaultMessage: 'e.g. Construction Equipment',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.2',
			defaultMessage: 'e.g. Adds wearable construction gear.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.3',
			defaultMessage: 'e.g. Better than Caving',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.3',
			defaultMessage: 'e.g. Adds realistic mineshaft-building mechanics.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.4',
			defaultMessage: 'e.g. Enhanced Portals',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.4',
			defaultMessage: 'e.g. Improves how Nether portals link to each other.',
		},
	}),
	defineMessages({
		name: {
			id: 'project.settings.general.name.placeholder.5',
			defaultMessage: 'e.g. Dangerous Mobs',
		},
		tagline: {
			id: 'project.settings.general.tagline.placeholder.5',
			defaultMessage:
				'e.g. Adds powerful boss versions of the normal mobs to encounter in the night.',
		},
	}),
]

const placeholderIndex = useState<number>('project-settings-random-placeholder', () =>
	Math.floor(Math.random() * (placeholders.length + 1)),
)

const placeholder = computed(() => placeholders[placeholderIndex.value] ?? placeholders[0])
</script>
<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<ValidationMessage
			:check="saveValidation.withoutFields(['name', 'summary', 'icon', 'slug'])"
			class="my-4"
		/>
		<UnsavedChangesPopup
			:original="saved"
			:modified="current"
			:saving="saving"
			:can-save="canSave"
			@reset="reset"
			@save="save"
		/>
		<div class="base-card block">
			<div class="group relative float-end ml-4">
				<IconSelect v-model="current.icon" />
				<ValidationMessage
					:check="iconValidation"
					:project-field="saved.icon"
					:current-field="current.icon"
					class="mt-2"
				/>
				<ValidationMessage :check="saveValidation.forField('icon')" class="mt-2" />
			</div>
			<div>
				<SettingsLabel
					id="project-name"
					:title="messages.nameTitle"
					:description="messages.nameDescription"
				/>
				<div class="flex">
					<Input
						id="project-name"
						v-model="current.title"
						:placeholder="formatMessage(placeholder.name)"
						autocomplete="off"
						:maxlength="50"
						wrapper-class="flex-grow"
					/>
				</div>
				<ValidationMessage
					:check="titleValidation"
					:project-field="saved.title"
					:current-field="current.title"
					class="mt-2"
				/>
				<ValidationMessage :check="saveValidation.forField('name')" class="mt-2" />
			</div>
			<div class="mt-4">
				<SettingsLabel
					id="project-tagline"
					:title="messages.taglineTitle"
					:description="messages.taglineDescription"
				/>
				<Input
					id="project-tagline"
					v-model="current.tagline"
					:placeholder="formatMessage(placeholder.tagline)"
					autocomplete="off"
					:maxlength="120"
					wrapper-class="w-full"
				/>
				<ValidationMessage
					:check="taglineValidation"
					:project-field="saved.tagline"
					:current-field="current.tagline"
					class="mt-2"
				/>
				<ValidationMessage :check="saveValidation.forField('summary')" class="mt-2" />
			</div>
			<div class="mt-4" @focusin="onSlugSuggestionFocusIn" @focusout="onSlugSuggestionFocusOut">
				<SettingsLabel id="project-url" :title="messages.urlTitle" />
				<Input
					id="project-url"
					v-model="current.url"
					:maxlength="64"
					autocomplete="off"
					wrapper-class="w-full"
				>
					<template #prefix>
						<span class="whitespace-nowrap">https://modrinth.com/project/</span>
					</template>
				</Input>
				<ValidationMessage :check="saveValidation.forField('slug')" class="mt-2" />
				<SlugSuggestions
					:selected="current.url"
					:suggestions="slugSuggestions"
					:visible="showSlugSuggestions"
					@select="current.url = $event"
				/>
			</div>
		</div>
	</div>
</template>
