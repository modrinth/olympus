<script setup lang="ts">
import {
	Checkbox,
	commonMessages,
	ConfirmLeaveModal,
	defineMessages,
	EmptyState,
	formatCategory,
	formatCategoryHeader,
	formatProjectTypeSentence,
	FormattedTag,
	injectProjectPageContext,
	type MessageDescriptor,
	sortProjectTypes,
	TagItem,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString, sortedCategories } from '@modrinth/utils'
import { computed } from 'vue'

import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'

interface Category {
	name: string
	header: string
	icon?: string
	project_type: string
}

interface CategoryGroup {
	id: string
	title: string
	description?: string
	categories: Category[]
}

const MAX_FEATURED_TAGS = 3

const SHARED_CATEGORY_PROJECT_TYPES: Record<string, string> = {
	plugin: 'mod',
	datapack: 'mod',
}

const tags = useGeneratedState()
const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'project.settings.tags.title',
		defaultMessage: 'Tags',
	},
	uploadVersionFirstHeading: {
		id: 'project.settings.tags.upload-version-first.heading',
		defaultMessage: 'Upload versions before adding tags',
	},
	categoriesDescription: {
		id: 'project.settings.tags.categories-description',
		defaultMessage: 'Select all categories that reflect the themes or function of your {type}.',
	},
	featuresDescription: {
		id: 'project.settings.tags.features-description',
		defaultMessage: 'Select all of the features that your {type} makes use of.',
	},
	resolutionsDescription: {
		id: 'project.settings.tags.resolutions-description',
		defaultMessage: 'Select the resolution(s) of textures in your {type}.',
	},
	performanceImpactDescription: {
		id: 'project.settings.tags.performance-impact-description',
		defaultMessage:
			'Select the realistic performance impact of your {type}. Select multiple if the {type} is configurable to different levels of performance impact.',
	},
	featuredTags: {
		id: 'project.settings.tags.featured-tags',
		defaultMessage: 'Featured tags',
	},
	featuredTagsDescription: {
		id: 'project.settings.tags.featured-tags-select-description',
		defaultMessage:
			'Select your most relevant tags. These are displayed before the rest of your tags.',
	},
	selectAtLeastOneCategory: {
		id: 'project.settings.tags.select-at-least-one-category',
		defaultMessage: 'Select at least one category in order to feature a category.',
	},
	featuredTagsRequired: {
		id: 'project.settings.tags.featured-tags-required',
		defaultMessage: 'You must have at least one featured tag.',
	},
	tooManyTagsServerHardWarning: {
		id: 'project.settings.tags.too-many-tags-server-hard-warning',
		defaultMessage:
			"You've selected {count} tags. Please reduce to 18 or fewer to keep your server focused and easier to discover.",
	},
	tooManyTagsServerSoftWarning: {
		id: 'project.settings.tags.too-many-tags-server-soft-warning',
		defaultMessage:
			"You've selected {count} tags. Consider reducing to 12 or fewer to keep your server focused and easier to discover.",
	},
	tooManyTagsProjectWarning: {
		id: 'project.settings.tags.too-many-tags-project-warning',
		defaultMessage:
			"You've selected {count} tags. Consider reducing to 8 or fewer to keep your project focused and easier to discover.",
	},
	multipleResolutionTagsWarning: {
		id: 'project.settings.tags.multiple-resolution-tags-warning',
		defaultMessage:
			"You've selected {count} resolution tags ({tags}). Resource packs should typically only have one resolution tag.",
	},
	allTagsSelectedWarning: {
		id: 'project.settings.tags.all-tags-selected-warning',
		defaultMessage:
			"You've selected all {count} available tags. Please select only the tags that truly apply to your project.",
	},
})

const groupTitleMessages: Record<string, MessageDescriptor> = defineMessages({
	categories: {
		id: 'project.settings.tags.group-title.categories',
		defaultMessage: '{showType, select, yes {{types} categories} other {Categories}}',
	},
	features: {
		id: 'project.settings.tags.group-title.features',
		defaultMessage: '{showType, select, yes {{types} features} other {Features}}',
	},
	resolutions: {
		id: 'project.settings.tags.group-title.resolutions',
		defaultMessage: '{showType, select, yes {{types} resolutions} other {Resolutions}}',
	},
	'performance impact': {
		id: 'project.settings.tags.group-title.performance-impact',
		defaultMessage:
			'{showType, select, yes {{types} performance impact} other {Performance impact}}',
	},
})

const groupDescriptionMessages: Record<string, MessageDescriptor> = {
	categories: messages.categoriesDescription,
	features: messages.featuresDescription,
	resolutions: messages.resolutionsDescription,
	'performance impact': messages.performanceImpactDescription,
}

const { projectV2: project, projectV3, patchProject } = injectProjectPageContext()

useProjectSettingsHeadTitle(messages.title)

const formatCategoryName = (categoryName: string) => formatCategory(formatMessage, categoryName)

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)

const canSelectTags = computed(() => project.value.versions.length > 0 || isServerProject.value)

const projectTypes = computed(() => {
	if (isServerProject.value) {
		return ['minecraft_java_server']
	}

	const types = projectV3.value?.project_types?.length
		? projectV3.value.project_types
		: [project.value.actualProjectType]

	return sortProjectTypes(new Set(types))
})

const allCategories = computed(
	() => sortedCategories(tags.value, formatCategoryName, locale.value) as Category[],
)

const projectTypesByCategoryList = computed(() => {
	const lists = new Map<string, string[]>()
	for (const projectType of projectTypes.value) {
		const source = SHARED_CATEGORY_PROJECT_TYPES[projectType] ?? projectType
		lists.set(source, [...(lists.get(source) ?? []), projectType])
	}
	return lists
})

const projectTypeListFormatter = computed(
	() => new Intl.ListFormat(locale.value, { style: 'long', type: 'conjunction' }),
)

function formatProjectTypeName(type: string) {
	return formatProjectTypeSentence(
		formatMessage,
		type === 'minecraft_java_server' ? 'server' : type,
	)
}

function formatGroupTitle(header: string, types: string[]) {
	const message = groupTitleMessages[header]
	if (!message) {
		return formatCategoryHeader(formatMessage, header)
	}

	const showType = projectTypes.value.length > 1
	return formatMessage(message, {
		showType: showType ? 'yes' : 'other',
		types: showType
			? capitalizeString(projectTypeListFormatter.value.format(types.map(formatProjectTypeName)))
			: '',
	})
}

function formatGroupDescription(header: string, types: string[]) {
	const message = groupDescriptionMessages[header]
	if (!message) {
		return undefined
	}

	// listing out every type gets unwieldy, fallback to the generic term when there's more than one
	return formatMessage(message, {
		type: formatProjectTypeName(types.length > 1 ? 'project' : types[0]),
	})
}

const categorySections = computed(() => {
	const sections: {
		categoryList: string
		header: string
		types: string[]
		categories: Category[]
	}[] = []

	for (const [categoryList, types] of projectTypesByCategoryList.value) {
		const byHeader = new Map<string, Category[]>()
		for (const category of allCategories.value) {
			if (category.project_type !== categoryList) continue
			byHeader.set(category.header, [...(byHeader.get(category.header) ?? []), category])
		}

		for (const [header, categories] of byHeader) {
			sections.push({
				categoryList,
				header,
				types,
				categories:
					header === 'minecraft_server_features' ? withPokemonFirst(categories) : categories,
			})
		}
	}

	return sections
})

const categoryGroups = computed<CategoryGroup[]>(() =>
	categorySections.value.map((section) => ({
		id: `${section.categoryList}-${section.header}`,
		title: formatGroupTitle(section.header, section.types),
		description: formatGroupDescription(section.header, section.types),
		categories: section.categories,
	})),
)

function withPokemonFirst(categories: Category[]) {
	return categories.slice().sort((a, b) => {
		if (a.name === 'pokemon') return -1
		if (b.name === 'pokemon') return 1
		return 0
	})
}

const availableTags = computed(() => [
	...new Set(categorySections.value.flatMap((section) => section.categories.map((x) => x.name))),
])

function hasSameTags(a: string[], b: string[]) {
	return a.length === b.length && a.every((tag) => b.includes(tag))
}

const {
	saved,
	current,
	saving,
	hasChanges,
	reset: resetForm,
	save: saveForm,
} = useSavable(
	() => ({
		selectedTags: availableTags.value.filter(
			(tag) =>
				project.value.categories.includes(tag) || project.value.additional_categories.includes(tag),
		),
		featuredTags: availableTags.value.filter((tag) => project.value.categories.includes(tag)),
	}),
	async () => {
		const featuredTags = current.value.featuredTags
		const additionalCategories = current.value.selectedTags.filter(
			(tag) => !featuredTags.includes(tag),
		)

		const data: Record<string, string[]> = {}

		if (!hasSameTags(featuredTags, project.value.categories)) {
			data.categories = featuredTags
		}

		if (!hasSameTags(additionalCategories, project.value.additional_categories)) {
			data.additional_categories = additionalCategories
		}

		await patchProject(data, false, true)
	},
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const saveValidation = useProjectSaveValidation(() => current.value)
const canSave = computed(() => !saveValidation.hasErrors.value)

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

const tagValidation = useProjectNagMessages('tags')

function toggleTagRaw(selection: string[], tag: string) {
	if (selection.includes(tag)) {
		return selection.filter((x) => x !== tag)
	}
	return availableTags.value.filter((x) => x === tag || selection.includes(x))
}

const toggleTag = (tag: string) => {
	current.value.selectedTags = toggleTagRaw(current.value.selectedTags, tag)
	if (!current.value.selectedTags.includes(tag)) {
		current.value.featuredTags = current.value.featuredTags.filter((x) => x !== tag)
	}
}

const toggleFeatured = (tag: string) => {
	current.value.featuredTags = toggleTagRaw(current.value.featuredTags, tag)
}
</script>
<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<h2 class="mb-4 mt-0 text-2xl font-semibold">
			{{ formatMessage(messages.title) }}
		</h2>
		<EmptyState
			v-if="!canSelectTags"
			type="no-documents"
			:heading="formatMessage(messages.uploadVersionFirstHeading)"
			:description="formatMessage(commonMessages.uploadVersionsEmptyStateDescription)"
		/>
		<div v-else class="flex flex-col gap-4">
			<div
				v-for="group in categoryGroups"
				:key="group.id"
				class="rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4"
			>
				<h3 class="mb-1 mt-0 text-lg font-semibold text-contrast">
					{{ group.title }}
				</h3>
				<p v-if="group.description" class="mb-3 mt-0">
					{{ group.description }}
				</p>
				<div class="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4">
					<Checkbox
						v-for="category in group.categories"
						:key="`${group.id}-${category.name}`"
						:model-value="current.selectedTags.includes(category.name)"
						:description="formatCategoryName(category.name)"
						@update:model-value="toggleTag(category.name)"
					>
						<span aria-hidden="true">
							<FormattedTag :tag="category.name" enforce-type="category" />
						</span>
					</Checkbox>
				</div>
			</div>

			<div class="rounded-2xl border border-solid border-surface-4 bg-surface-3 p-4">
				<div class="mb-1 flex items-center gap-2">
					<h3 class="m-0 text-lg font-semibold text-contrast">
						{{ formatMessage(messages.featuredTags) }}
					</h3>
					<TagItem
						v-tooltip="
							current.featuredTags.length === 0
								? formatMessage(messages.featuredTagsRequired)
								: undefined
						"
					>
						{{ current.featuredTags.length }}/{{ MAX_FEATURED_TAGS }}
					</TagItem>
				</div>
				<p class="mb-3 mt-0">
					{{ formatMessage(messages.featuredTagsDescription) }}
				</p>
				<p v-if="current.selectedTags.length < 1" class="m-0 text-secondary">
					{{ formatMessage(messages.selectAtLeastOneCategory) }}
				</p>
				<div v-else class="grid grid-cols-2 gap-x-4 gap-y-2 sm:grid-cols-4">
					<Checkbox
						v-for="name in current.selectedTags"
						:key="`featured-${name}`"
						:model-value="current.featuredTags.includes(name)"
						:description="formatCategoryName(name)"
						@update:model-value="toggleFeatured(name)"
					>
						<span aria-hidden="true">
							<FormattedTag :tag="name" enforce-type="category" />
						</span>
					</Checkbox>
				</div>
			</div>
			<ValidationMessage
				:check="tagValidation"
				:project-field="JSON.stringify(saved)"
				:current-field="JSON.stringify(current)"
			/>
			<ValidationMessage :check="saveValidation.forField('tags')" />
		</div>
		<ValidationMessage :check="saveValidation.withoutFields(['tags'])" class="my-4" />
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
