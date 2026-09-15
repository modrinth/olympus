<template>
	<div>
		<AiImageWarningModal ref="aiImageWarningModal" />
		<ConfirmModal
			ref="modal_confirm"
			:title="formatMessage(messages.deleteConfirmationTitle)"
			:description="formatMessage(messages.deleteConfirmationDescription)"
			:has-to-type="true"
			:confirmation-text="project.name"
			:proceed-label="formatMessage(messages.deleteConfirmationProceedText)"
			@proceed="deleteProject"
		/>
		<section class="universal-card">
			<div class="flex max-w-[600px] flex-col gap-6">
				<div class="label">
					<h3>
						<span class="label__title size-card-header">Project information</span>
					</h3>
				</div>

				<div>
					<label for="project-name" class="block w-fit">
						<span class="label__title">Name</span>
					</label>
					<Input
						id="project-name"
						v-model="name"
						:maxlength="2048"
						wrapper-class="w-full max-w-72"
						:disabled="!hasPermission"
					/>
					<ValidationMessage
						:check="nameValidation"
						:project-field="project.name"
						:current-field="name"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('name')" class="mt-2" />
				</div>

				<div @focusin="onSlugSuggestionFocusIn" @focusout="onSlugSuggestionFocusOut">
					<label for="project-slug" class="block w-fit">
						<span class="label__title">URL</span>
					</label>
					<Input
						id="project-slug"
						v-model="slug"
						:maxlength="64"
						autocomplete="off"
						:disabled="!hasPermission"
						wrapper-class="w-full"
					>
						<template #prefix>
							<span class="whitespace-nowrap">
								<span class="hidden sm:inline">https://modrinth.com</span>/{{ projectTypeForUrl }}/
							</span>
						</template>
					</Input>
					<ValidationMessage :check="saveValidation.forField('slug')" class="mt-2" />
					<SlugSuggestions
						:selected="slug"
						:suggestions="slugSuggestions"
						:visible="hasPermission && showSlugSuggestions"
						@select="slug = $event"
					/>
				</div>

				<div>
					<label for="project-summary" class="block w-fit">
						<span class="label__title">Summary</span>
					</label>
					<Textarea
						id="project-summary"
						v-model="summary"
						:maxlength="256"
						:disabled="!hasPermission"
						resize="vertical"
					/>
					<ValidationMessage
						:check="summaryValidation"
						:project-field="project.summary"
						:current-field="summary"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('summary')" class="mt-2" />
				</div>

				<div>
					<label for="project-icon" class="block w-fit">
						<span class="label__title"
							>Icon <span class="font-normal text-secondary">(optional)</span></span
						>
					</label>

					<div class="input-group">
						<Avatar
							:src="deletedIcon ? null : previewImage ? previewImage : project.icon_url"
							:alt="project.name"
							size="md"
							class="project__icon"
						/>
						<div class="flex flex-col gap-2">
							<FileButton
								id="project-icon"
								:max-size="262144000"
								accept="image/png,image/jpeg,image/gif,image/webp"
								class="button-like choose-image"
								prompt="Upload icon"
								aria-label="Upload icon"
								:disabled="!hasPermission"
								@change="showPreviewImage"
							>
								<UploadIcon aria-hidden="true" />
							</FileButton>
							<Button
								v-if="!deletedIcon && (previewImage || project.icon_url)"
								:disabled="!hasPermission"
								@click="markIconForDeletion"
							>
								<TrashIcon aria-hidden="true" />
								Remove icon
							</Button>
						</div>
					</div>
					<ValidationMessage
						:check="iconValidation"
						:project-field="false"
						:current-field="Boolean(icon || deletedIcon)"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('icon')" class="mt-2" />
				</div>

				<!-- Server Project Settings -->
				<template v-if="isServerProject">
					<!-- Banner -->
					<div>
						<label class="block w-fit">
							<span class="label__title"
								>Banner <span class="font-normal text-secondary">(optional)</span></span
							>
						</label>
						<div class="mt-2">
							<label
								class="flex cursor-pointer flex-col items-center justify-center rounded-2xl border-dashed border-surface-5 transition-colors"
								:class="
									!deletedBanner && (bannerPreview || bannerGalleryImage?.url)
										? 'border-none'
										: 'aspect-[468/60] border-2 bg-surface-2'
								"
							>
								<div
									v-if="!deletedBanner && (bannerPreview || bannerGalleryImage?.url)"
									class="relative h-full w-full overflow-hidden rounded-2xl"
								>
									<img
										:src="bannerPreview || bannerGalleryImage?.url"
										alt="Banner preview"
										class="h-full w-full object-cover"
									/>
								</div>
								<ImageIcon v-else aria-hidden="true" class="h-8 w-8 text-secondary" />
								<input
									type="file"
									accept="image/png,image/jpeg,image/gif,image/webp"
									class="hidden"
									:disabled="!hasPermission"
									@change="
										(e) => {
											const input = e.target
											if (input.files?.length) {
												if (
													fileIsValid(
														input.files[0],
														{ maxSize: 524288000, alertOnInvalid: true },
														formatBytes,
													)
												)
													showBannerPreview(Array.from(input.files))
											}
										}
									"
								/>
							</label>
						</div>
						<div class="mt-2 flex items-center gap-2">
							<FileButton
								:max-size="524288"
								accept="image/png,image/jpeg,image/gif,image/webp"
								class="button-like"
								prompt="Upload banner"
								:disabled="!hasPermission"
								@change="showBannerPreview"
							>
								<UploadIcon aria-hidden="true" />
							</FileButton>
							<Button
								v-if="!deletedBanner && (bannerPreview || bannerGalleryImage?.url)"
								:disabled="!hasPermission"
								@click="markBannerForDeletion"
							>
								<TrashIcon aria-hidden="true" />
								Remove banner
							</Button>
						</div>
						<div class="mt-2 text-secondary">Gif, 468×60px recommended.</div>
					</div>
				</template>

				<div id="visibility">
					<label class="block w-fit">
						<span class="label__title">Visibility</span>
					</label>
					<div class="flex flex-col gap-2.5">
						<Combobox
							v-model="visibility"
							:options="visibilityOptions"
							placeholder="Select one"
							:disabled="!hasPermission"
							:max-height="500"
						/>
					</div>
				</div>
			</div>
		</section>

		<div class="mt-6 flex flex-col gap-4">
			<h2 class="m-0 text-2xl font-semibold">
				{{ formatMessage(messages.dangerZone) }}
			</h2>
			<SettingsToggleCard
				v-if="!isServerProject"
				v-model="monetizationEnabled"
				:disabled="monetizationToggleDisabled"
				:title="formatMessage(messages.monetizationTitle)"
			>
				<p>
					<IntlFormatted :message-id="messages.monetizationDescription">
						<template #rewards-program-link="{ children }">
							<nuxt-link
								to="/legal/cmp-info"
								target="_blank"
								class="smart-clickable:allow-pointer-events text-link"
							>
								<component :is="() => normalizeChildren(children)" />
							</nuxt-link>
						</template>
					</IntlFormatted>
				</p>
				<div v-if="isForceDemonetized" class="mt-2">
					<SettingsInlineWarning>
						<IntlFormatted :message-id="messages.monetizationDisabledDescription">
							<template #contact-support-link="{ children }">
								<a
									class="smart-clickable:allow-pointer-events text-orange underline hover:brightness-110"
									href="https://support.modrinth.com"
									target="_blank"
									rel="noopener noreferrer"
								>
									<component :is="() => normalizeChildren(children)" />
								</a>
							</template>
						</IntlFormatted>
					</SettingsInlineWarning>
				</div>
				<div v-if="isStaff" class="smart-clickable:allow-pointer-events mt-2">
					<Button
						v-if="!isForceDemonetized"
						type="colored"
						color="orange"
						:disabled="loadingModeratorMonetization"
						@click="updateMonetizationStatus('force-demonetized')"
					>
						<ScaleIcon aria-hidden="true" />
						Disable monetization
					</Button>
					<Button
						v-else
						:disabled="loadingModeratorMonetization"
						@click="updateMonetizationStatus('monetized')"
					>
						<ScaleIcon aria-hidden="true" />
						Allow monetization
					</Button>
				</div>
			</SettingsToggleCard>
			<SettingsOptionCard :title="formatMessage(messages.deleteProjectTitle)">
				<p>
					<IntlFormatted :message-id="messages.deleteProjectDescription1">
						<template #emphasis="{ children }">
							<span class="font-medium text-red"
								><component :is="() => normalizeChildren(children)"
							/></span>
						</template>
					</IntlFormatted>
				</p>
				<p>
					{{ formatMessage(messages.deleteProjectDescription2) }}
				</p>
				<template #actions>
					<Button
						type="colored"
						color="red"
						:disabled="!hasDeletePermission"
						@click="$refs.modal_confirm.show()"
					>
						<TrashIcon aria-hidden="true" />
						{{ formatMessage(messages.deleteProjectButton) }}
					</Button>
				</template>
			</SettingsOptionCard>
		</div>
		<ValidationMessage
			:check="saveValidation.withoutFields(['name', 'summary', 'icon', 'slug'])"
			class="my-4"
		/>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="canSave"
			@reset="resetChanges"
			@save="handleSave"
		/>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
	</div>
</template>

<script setup>
import { ImageIcon, ScaleIcon, TrashIcon, UploadIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	Combobox,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	ConfirmModal,
	defineMessages,
	FileButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	IntlFormatted,
	normalizeChildren,
	SettingsInlineWarning,
	SettingsOptionCard,
	SettingsToggleCard,
	Textarea,
	UnsavedChangesPopup,
	useFormatBytes,
	usePageLeaveSafety,
	useVIntl,
} from '@modrinth/ui'
import { fileIsValid, formatProjectStatus, isAdmin } from '@modrinth/utils'

import AiImageWarningModal from '~/components/ui/AiImageWarningModal.vue'
import SlugSuggestions from '~/components/ui/SlugSuggestions.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useAuth } from '~/composables/auth.js'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import {
	useProjectSlugSuggestions,
	useSlugSuggestionVisibility,
} from '~/composables/project-slug-suggestions'
import { fileDeclaresAi } from '~/helpers/c2pa'
import { getProjectTypeForUrl } from '~/helpers/projects.js'

const auth = await useAuth()
const { formatMessage } = useVIntl()

const { addNotification } = injectNotificationManager()
const {
	projectV3: project,
	currentMember,
	allMembers,
	patchProjectV3,
	patchIcon,
	invalidate,
} = injectProjectPageContext()
const { labrinth } = injectModrinthClient()
const aiImageWarningModal = useTemplateRef('aiImageWarningModal')

useProjectSettingsHeadTitle(commonProjectSettingsMessages.general)

const tags = useGeneratedState()
const router = useNativeRouter()

const formatBytes = useFormatBytes()

const name = ref(project.value.name)
const slug = ref(project.value.slug ?? '')
const summary = ref(project.value.summary)
const {
	onFocusIn: onSlugSuggestionFocusIn,
	onFocusOut: onSlugSuggestionFocusOut,
	visible: showSlugSuggestions,
} = useSlugSuggestionVisibility()
const ownerUsername = computed(
	() => (allMembers.value.find((member) => member.is_owner) ?? allMembers.value[0])?.user.username,
)
const { suggestions: slugSuggestions } = useProjectSlugSuggestions({
	title: name,
	username: ownerUsername,
	currentProjectId: () => project.value.id,
	enabled: showSlugSuggestions,
})
const icon = ref(null)
const previewImage = ref(null)
const deletedIcon = ref(false)
const visibility = ref(
	tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status,
)

const monetizationEnabled = ref(project.value.monetization_status === 'monetized')
const loadingModeratorMonetization = ref(false)

watch(
	() => project.value.monetization_status,
	() => {
		monetizationEnabled.value = project.value.monetization_status === 'monetized'
	},
)

const isStaff = computed(
	() => !!auth.value.user && tags.value.staffRoles.includes(auth.value.user.role),
)

const isForceDemonetized = computed(() => project.value.monetization_status === 'force-demonetized')

// Server project specific refs
const MC_SERVER_BANNER_NAME = '__mc_server_banner__'
const isServerProject = computed(() => project.value?.minecraft_server != null)
const projectTypeForUrl = computed(() => {
	if (isServerProject.value) return 'server'
	const type = project.value.project_types?.[0] ?? 'mod'
	return getProjectTypeForUrl(type, project.value.loaders)
})
const bannerPreview = ref(null)
const deletedBanner = ref(false)
const bannerFile = ref(null)
const bannerGalleryImage = computed(() =>
	project.value.gallery?.find((img) => img.name === MC_SERVER_BANNER_NAME),
)
const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (
		isAdminUser.value || ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
	)
})

const nameValidation = useProjectNagMessages('name')
const summaryValidation = useProjectNagMessages('summary')
const iconValidation = useProjectNagMessages('icon')
const canSave = computed(() => hasPermission.value && !saveValidation.hasErrors.value)

const monetizationToggleDisabled = computed(() => !hasPermission.value || isForceDemonetized.value)

const hasDeletePermission = computed(() => {
	const DELETE_PROJECT = 1 << 7
	return ((currentMember.value?.permissions ?? 0) & DELETE_PROJECT) === DELETE_PROJECT
})

const visibilityOptions = computed(() =>
	tags.value.approvedStatuses
		.filter((status) => status !== 'archived')
		.map((status) => {
			const subLabel = () => {
				switch (status) {
					case 'approved':
						return 'Visible via URL, on your profile, and in search.'
					case 'unlisted':
						return 'Visible via URL only. Not shown on your profile or in search.'
					case 'private':
						return 'Not publicly visible. Only accessible to project members.'
					default:
						return ''
				}
			}
			return {
				value: status,
				label: formatProjectStatus(status),
				subLabel: subLabel(),
			}
		}),
)

const basePatchData = computed(() => {
	const data = {}

	if (name.value !== project.value.name) {
		data.name = name.value.trim()
	}
	if (slug.value !== (project.value.slug ?? '')) {
		data.slug = slug.value.trim()
	}
	if (summary.value !== project.value.summary) {
		data.summary = summary.value.trim()
	}
	if (tags.value.approvedStatuses.includes(project.value.status)) {
		if (visibility.value !== project.value.status) {
			data.status = visibility.value
		}
	} else if (visibility.value !== project.value.requested_status) {
		data.requested_status = visibility.value
	}

	if (project.value.monetization_status !== 'force-demonetized') {
		const wasMonetized = project.value.monetization_status === 'monetized'
		if (monetizationEnabled.value !== wasMonetized) {
			data.monetization_status = monetizationEnabled.value ? 'monetized' : 'demonetized'
		}
	}

	return data
})

const saving = ref(false)

const original = computed(() => ({
	name: project.value.name,
	slug: project.value.slug ?? '',
	summary: project.value.summary,
	visibility: tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status,
	icon: null,
	deletedIcon: false,
	bannerFile: null,
	deletedBanner: false,
	monetizationEnabled: project.value.monetization_status === 'monetized',
}))

const modified = computed(() => ({
	name: name.value,
	slug: slug.value,
	summary: summary.value,
	visibility: visibility.value,
	icon: icon.value,
	deletedIcon: deletedIcon.value,
	bannerFile: bannerFile.value,
	deletedBanner: deletedBanner.value,
	monetizationEnabled: monetizationEnabled.value,
}))

const hasChanges = computed(() =>
	Object.keys(modified.value).some((key) => original.value[key] !== modified.value[key]),
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

function resetChanges() {
	saveValidation.clear()
	name.value = project.value.name
	slug.value = project.value.slug ?? ''
	summary.value = project.value.summary
	visibility.value = tags.value.approvedStatuses.includes(project.value.status)
		? project.value.status
		: project.value.requested_status
	icon.value = null
	previewImage.value = null
	deletedIcon.value = false
	bannerFile.value = null
	bannerPreview.value = null
	deletedBanner.value = false
	monetizationEnabled.value = project.value.monetization_status === 'monetized'
}

async function updateMonetizationStatus(status) {
	loadingModeratorMonetization.value = true
	try {
		await patchProjectV3({ monetization_status: status })
	} finally {
		loadingModeratorMonetization.value = false
	}
}

const saveValidation = useProjectSaveValidation(() => modified.value)

async function handleSave() {
	if (!canSave.value || saving.value) return
	summary.value = summary.value.trim()
	const submittedState = saveValidation.snapshot()
	saving.value = true
	try {
		const hasPatchChanges = Object.keys(basePatchData.value).length > 0

		if (hasPatchChanges) {
			await patchProjectV3(basePatchData.value, false, true)
		}

		if (deletedIcon.value) {
			await deleteIcon()
			deletedIcon.value = false
		} else if (icon.value) {
			await patchIcon(icon.value)
			icon.value = null
		}

		if (deletedBanner.value) {
			await deleteBanner()
			deletedBanner.value = false
		} else if (bannerFile.value) {
			await uploadBanner()
			bannerFile.value = null
			bannerPreview.value = null
		}
		saveValidation.clear()
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	} finally {
		saving.value = false
	}
}

const showPreviewImage = async (files) => {
	const file = files[0]
	if (!file) {
		return
	}
	if (await fileDeclaresAi(file)) {
		aiImageWarningModal.value?.show()
		return
	}
	const reader = new FileReader()
	icon.value = file
	deletedIcon.value = false
	reader.readAsDataURL(icon.value)
	reader.onload = (event) => {
		previewImage.value = event.target?.result
	}
}

const showBannerPreview = (files) => {
	const file = files[0]
	if (file) {
		bannerFile.value = file
		const reader = new FileReader()
		reader.onload = (e) => {
			bannerPreview.value = e.target.result
		}
		reader.readAsDataURL(file)
		deletedBanner.value = false
	}
}

const markBannerForDeletion = () => {
	bannerPreview.value = null
	bannerFile.value = null
	deletedBanner.value = true
}

const uploadBanner = async () => {
	if (!bannerFile.value) return

	try {
		const existingBanner = project.value.gallery?.find((img) => img.name === MC_SERVER_BANNER_NAME)
		if (existingBanner) {
			await labrinth.projects_v3.deleteGalleryImage(project.value.id, existingBanner.url)
		}

		const ext = bannerFile.value.type.split('/').pop() ?? 'png'
		await labrinth.projects_v3.createGalleryImage(project.value.id, bannerFile.value, {
			ext,
			featured: false,
			name: MC_SERVER_BANNER_NAME,
		})

		await invalidate()
		addNotification({
			title: 'Banner updated',
			text: 'Your project banner has been updated.',
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'Failed to update banner',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	}
}

const deleteBanner = async () => {
	try {
		const bannerImage = project.value.gallery?.find((img) => img.name === MC_SERVER_BANNER_NAME)
		if (bannerImage) {
			await labrinth.projects_v3.deleteGalleryImage(project.value.id, bannerImage.url)
			await invalidate()
			addNotification({
				title: 'Banner removed',
				text: 'Your project banner has been removed.',
				type: 'success',
			})
		}
	} catch (err) {
		addNotification({
			title: 'Failed to remove banner',
			text: err.data?.description ?? String(err),
			type: 'error',
		})
	}
}

const deleteProject = async () => {
	await labrinth.projects_v3.deleteProject(project.value.id)
	await initUserProjects()
	await router.push('/dashboard/projects')
	addNotification({
		title: 'Project deleted',
		text: 'Your project has been deleted.',
		type: 'success',
	})
}

const markIconForDeletion = () => {
	deletedIcon.value = true
	icon.value = null
	previewImage.value = null
}

const deleteIcon = async () => {
	await labrinth.projects_v3.deleteIcon(project.value.id)
	await invalidate()
	addNotification({
		title: 'Project icon removed',
		text: "Your project's icon has been removed.",
		type: 'success',
	})
}

const messages = defineMessages({
	dangerZone: {
		id: 'project.settings.danger-zone',
		defaultMessage: 'Danger zone',
	},
	monetizationTitle: {
		id: 'project.settings.monetization.title',
		defaultMessage: 'Monetization',
	},
	monetizationDescription: {
		id: 'project.settings.monetization.description',
		defaultMessage: `Projects on Modrinth are automatically enrolled in the <rewards-program-link>Rewards Program</rewards-program-link>. If you don't want to (or can't for legal reasons) earn revenue from this project, you can turn it off here.`,
	},
	monetizationDisabledDescription: {
		id: 'project.settings.monetization.disabled-description',
		defaultMessage: `This project is not eligible for monetization. If you think this is a mistake, please <contact-support-link>contact support</contact-support-link>.`,
	},
	deleteProjectTitle: {
		id: 'project.settings.delete-project.title',
		defaultMessage: 'Delete project',
	},
	deleteProjectButton: {
		id: 'project.settings.delete-project.button',
		defaultMessage: 'Delete project',
	},
	deleteProjectDescription1: {
		id: 'project.settings.delete-project.description.1',
		defaultMessage:
			'Permanently deletes this project from Modrinth. Deleted projects <emphasis>cannot be recovered</emphasis> by Modrinth staff or support.',
	},
	deleteProjectDescription2: {
		id: 'project.settings.delete-project.description.2',
		defaultMessage:
			'Files uploaded to this project that are actively used in Modpacks hosted on Modrinth may continue to exist.',
	},
	deleteConfirmationTitle: {
		id: 'project.settings.delete-project.confirmation.title',
		defaultMessage: 'Are you sure you want to delete this project?',
	},
	deleteConfirmationDescription: {
		id: 'project.settings.delete-project.confirmation.description',
		defaultMessage: `If you proceed, all of this project's information and all of its versions will be immediately deleted from our database. None of it is recoverable later if you change your mind. Consider just setting your project to be Private for a less permanent option.`,
	},
	deleteConfirmationProceedText: {
		id: 'project.settings.delete-project.confirmation.proceed-text',
		defaultMessage: 'Permanently delete project',
	},
})
</script>

<style lang="scss" scoped>
.visibility-info {
	padding: 0;
	list-style: none;
	display: flex;
	flex-direction: column;
	gap: var(--spacing-card-xs);

	li {
		display: flex;
		align-items: center;
		gap: var(--spacing-card-xs);
	}
}

svg {
	&.good {
		color: var(--color-green);
	}

	&.bad {
		color: var(--color-red);
	}

	&.warn {
		color: var(--color-orange);
	}
}
</style>
