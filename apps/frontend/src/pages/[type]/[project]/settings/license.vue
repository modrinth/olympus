<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<section class="universal-card flex flex-col gap-6">
			<div class="flex flex-col gap-2">
				<h2 class="m-0">
					{{ formatMessage(messages.title) }}
				</h2>
				<p class="m-0 text-base text-secondary">
					<IntlFormatted
						:message-id="messages.intro"
						:values="{ type: formatProjectType(project.project_type).toLowerCase() }"
					>
						<template #guide="{ children }">
							<NuxtLink
								to="/news/article/licensing-guide/"
								target="_blank"
								rel="noopener"
								class="text-link"
								><component :is="() => children"
							/></NuxtLink>
						</template>
					</IntlFormatted>
				</p>
			</div>

			<div class="flex min-w-0 flex-col gap-2">
				<label for="license-multiselect" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(messages.select) }}
				</label>

				<div class="flex min-w-0 flex-col gap-2">
					<Combobox
						v-model="selectedLicense"
						:options="licenseOptions"
						:display-value="licenseDisplayName(current.license) || undefined"
						:placeholder="formatMessage(messages.selectPlaceholder)"
						:disabled="saving || !hasPermission"
						trigger-type="base"
						class="w-full max-w-[600px]"
					/>
					<p class="m-0 text-base text-secondary">
						{{ formatMessage(messages.selectDescription) }}
					</p>
					<ValidationMessage
						:check="licenseSelectionValidation"
						:project-field="project.license.id"
						:current-field="licenseId"
						class="mt-2"
					/>
					<ValidationMessage
						:check="[
							...saveValidation.forField('license'),
							...saveValidation.forField('source-availability', 'source'),
						]"
						class="mt-2"
					/>
				</div>
			</div>

			<div v-if="current.license.requiresOnlyOrLater" class="flex min-w-0 flex-col gap-2">
				<label for="or-later-checkbox" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(messages.later) }}
				</label>

				<Checkbox
					id="or-later-checkbox"
					v-model="current.allowOrLater"
					:disabled="saving || !hasPermission"
					:description="formatMessage(messages.allowLater)"
				>
					{{ formatMessage(messages.allowLater) }}
				</Checkbox>
				<p class="m-0 text-base text-secondary">
					{{ formatMessage(messages.laterDescription) }}
				</p>
			</div>

			<div v-if="current.license.friendly" class="flex min-w-0 flex-col gap-2">
				<label for="license-url" class="w-fit text-lg font-semibold text-contrast">
					{{ formatMessage(messages.url) }}
				</label>

				<div class="flex min-w-0 flex-col gap-2">
					<Input
						id="license-url"
						v-model="current.licenseUrl"
						type="url"
						:maxlength="2048"
						:placeholder="
							formatMessage(
								current.license.friendly === 'Custom' ? messages.url : messages.optionalUrl,
							)
						"
						:disabled="saving || !hasPermission || licenseId === 'LicenseRef-Unknown'"
						wrapper-class="w-full"
					/>
					<p class="m-0 text-base text-secondary">
						{{
							formatMessage(
								current.license.friendly === 'Custom'
									? messages.customUrlDescription
									: messages.urlDescription,
							)
						}}
					</p>
					<ValidationMessage :check="customUrlMessage" />
					<ValidationMessage
						:check="
							saveValidation
								.forField('custom-license', 'license')
								.filter((message) => message.values?.missingUrl)
						"
					/>
					<ValidationMessage
						:check="customLicenseValidation"
						:project-field="project.license.id"
						:current-field="licenseId"
					/>
					<ValidationMessage
						:check="effectiveLicenseCheck"
						:project-field="project.license.url ?? ''"
						:current-field="current.licenseUrl"
					/>
					<ValidationMessage
						:check="saveValidation.forField('license-url', 'license')"
						class="mt-2"
					/>
				</div>
			</div>

			<div v-if="current.license?.friendly === 'Custom'" class="flex min-w-0 flex-col gap-2">
				<label
					:for="current.hasSpdxLicense ? 'license-spdx' : 'license-name'"
					class="w-fit text-lg font-semibold text-contrast"
				>
					{{ formatMessage(current.hasSpdxLicense ? messages.spdx : messages.name) }}
				</label>

				<div class="flex min-w-0 flex-col gap-2">
					<Input
						v-if="current.hasSpdxLicense"
						id="license-spdx"
						v-model="current.license.short"
						wrapper-class="w-full"
						:maxlength="128"
						:placeholder="formatMessage(messages.spdxPlaceholder)"
						:disabled="saving || !hasPermission"
					/>
					<Input
						v-else
						id="license-name"
						v-model="current.license.short"
						wrapper-class="w-full"
						:maxlength="128"
						:placeholder="formatMessage(messages.namePlaceholder)"
						:disabled="saving || !hasPermission"
					/>

					<Checkbox
						v-model="current.hasSpdxLicense"
						:disabled="saving || !hasPermission"
						:description="formatMessage(messages.hasSpdx)"
					>
						{{ formatMessage(messages.hasSpdx) }}
					</Checkbox>

					<p class="m-0 text-base text-secondary">
						<IntlFormatted
							:message-id="
								current.hasSpdxLicense ? messages.spdxDescription : messages.nameDescription
							"
						>
							<template #spdx="{ children }">
								<a
									href="https://spdx.org/licenses/"
									target="_blank"
									rel="noopener"
									class="text-link"
								>
									<component :is="() => children" />
								</a>
							</template>
						</IntlFormatted>
					</p>
					<ValidationMessage :check="customNameMessage" />
					<ValidationMessage
						:check="
							saveValidation
								.forField('custom-license', 'license')
								.filter((message) => message.values?.missingName)
						"
					/>
				</div>
			</div>
		</section>
		<ValidationMessage
			:check="
				saveValidation.withoutFields([
					'license',
					['source-availability', 'source'],
					['license-url', 'license'],
					['custom-license', 'license'],
				])
			"
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
	</div>
</template>

<script setup lang="ts">
import {
	Checkbox,
	Combobox,
	type ComboboxOption,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	IntlFormatted,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { builtinLicenses, formatProjectType, isAdmin, TeamMemberPermission } from '@modrinth/utils'
import { computed } from 'vue'

import ValidationMessage from '@/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { normalizeProjectUrl } from '~/helpers/project-url'

const { projectV2: project, currentMember, patchProjectV3 } = injectProjectPageContext()

const { addNotification } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	custom: { id: 'project.settings.license.custom', defaultMessage: 'Custom' },
	allRights: {
		id: 'project.settings.license.all-rights',
		defaultMessage: 'All Rights Reserved/No License',
	},
	title: { id: 'project.settings.license.title', defaultMessage: 'License' },
	intro: {
		id: 'project.settings.license.intro',
		defaultMessage:
			'It is important to choose a proper license for your {type}. You may choose one from our list or provide a custom license. You may also provide a custom URL to your chosen license; otherwise, the license text will be displayed. See our <guide>licensing guide</guide> for more information.',
	},
	select: {
		id: 'project.settings.license.select',
		defaultMessage: 'Select a license',
	},
	selectDescription: {
		id: 'project.settings.license.select-description',
		defaultMessage: "How users are and aren't allowed to use your project.",
	},
	selectPlaceholder: {
		id: 'project.settings.license.select-placeholder',
		defaultMessage: 'Select license...',
	},
	later: {
		id: 'project.settings.license.later',
		defaultMessage: 'Later editions',
	},
	laterDescription: {
		id: 'project.settings.license.later-description',
		defaultMessage:
			'The license you selected has an "or later" clause. If you check this box, users may use your project under later editions of the license.',
	},
	allowLater: {
		id: 'project.settings.license.allow-later',
		defaultMessage: 'Allow later editions',
	},
	url: { id: 'project.settings.license.url', defaultMessage: 'License URL' },
	optionalUrl: {
		id: 'project.settings.license.optional-url',
		defaultMessage: 'License URL (optional)',
	},
	urlDescription: {
		id: 'project.settings.license.url-description',
		defaultMessage:
			"The web location of the full license text. If you don't provide a link, the license text will be displayed instead.",
	},
	customUrlDescription: {
		id: 'project.settings.license.custom-url-description',
		defaultMessage:
			'The web location of the full license text. You have to provide a link since this is a custom license.',
	},
	spdx: {
		id: 'project.settings.license.spdx',
		defaultMessage: 'SPDX identifier',
	},
	spdxPlaceholder: {
		id: 'project.settings.license.spdx-placeholder',
		defaultMessage: 'SPDX identifier',
	},
	spdxDescription: {
		id: 'project.settings.license.spdx-description',
		defaultMessage:
			'If your license does not have an official <spdx>SPDX license identifier</spdx>, uncheck the box and enter the name of the license instead.',
	},
	name: { id: 'project.settings.license.name', defaultMessage: 'License name' },
	namePlaceholder: {
		id: 'project.settings.license.name-placeholder',
		defaultMessage: 'License name',
	},
	nameDescription: {
		id: 'project.settings.license.name-description',
		defaultMessage:
			'The full name of the license. If the license has a SPDX identifier, please check the checkbox and use the identifier instead.',
	},
	hasSpdx: {
		id: 'project.settings.license.has-spdx',
		defaultMessage: 'Use SPDX identifier',
	},
	missingName: {
		id: 'project.settings.license.missing-name',
		defaultMessage: 'Enter a name or SPDX identifier for your custom license.',
	},
	missingUrl: {
		id: 'project.settings.license.missing-url',
		defaultMessage: 'Enter a URL to the full text of your custom license.',
	},
	updated: {
		id: 'project.settings.license.updated',
		defaultMessage: 'License updated',
	},
	updatedText: {
		id: 'project.settings.license.updated-text',
		defaultMessage: 'Your license has been updated.',
	},
})
useProjectSettingsHeadTitle(commonProjectSettingsMessages.license)

function licenseDisplayName(license: { short: string; friendly: string }) {
	if (license.friendly === 'Custom') return formatMessage(messages.custom)
	if (license.short === 'All-Rights-Reserved') return formatMessage(messages.allRights)
	return license.friendly
}

const licenseOptions = computed<ComboboxOption<string>[]>(() =>
	builtinLicenses.map((license) => ({
		value: license.short,
		label: licenseDisplayName(license),
	})),
)

function getInitialLicense() {
	const oldLicenseId = project.value.license.id
	const trimmedLicenseId = oldLicenseId
		.replaceAll('-only', '')
		.replaceAll('-or-later', '')
		.replaceAll('LicenseRef-', '')

	if (oldLicenseId === 'LicenseRef-Unknown') {
		return {
			friendly: '',
			short: oldLicenseId.replaceAll('LicenseRef-', ''),
			requiresOnlyOrLater: false,
		}
	}

	return (
		builtinLicenses.find((x) => x.short === trimmedLicenseId) ?? {
			friendly: 'Custom',
			short: oldLicenseId.replaceAll('LicenseRef-', ''),
			requiresOnlyOrLater: oldLicenseId.includes('-or-later'),
		}
	)
}

const {
	saved,
	current,
	saving,
	hasChanges,
	reset,
	save: saveLicense,
} = useSavable(
	() => ({
		license: getInitialLicense(),
		licenseUrl: project.value.license.url ?? '',
		allowOrLater: project.value.license.id.includes('-or-later'),
		hasSpdxLicense: !project.value.license.id.includes('LicenseRef-'),
	}),
	async () => {
		const payload: {
			license_id?: string
			license_url?: string | null
		} = {}

		if (licenseId.value !== project.value.license.id) {
			payload.license_id = licenseId.value
		}

		if (current.value.licenseUrl !== project.value.license.url) {
			payload.license_url = normalizeProjectUrl(current.value.licenseUrl) || null
		}

		await patchProjectV3(payload, true, true)
	},
)

const licenseSelectionValidation = useProjectNagMessages('license')
const customLicenseValidation = useProjectNagMessages('custom-license')
const effectiveLicenseCheck = useProjectNagMessages('license-url', 'license')

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const selectedLicense = computed({
	get: () => (current.value.license.friendly === 'Custom' ? '' : current.value.license.short),
	set: (short: string) => {
		const license = builtinLicenses.find((option) => option.short === short)
		if (license) current.value.license = license
	},
})

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(
	() =>
		isAdminUser.value ||
		Boolean((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
)

const saveValidation = useProjectSaveValidation(() => current.value)
const missingCustomName = computed(
	() => current.value.license.friendly === 'Custom' && !current.value.license.short.trim(),
)
const missingCustomUrl = computed(
	() => current.value.license.friendly === 'Custom' && !current.value.licenseUrl.trim(),
)
const customNameMessage = computed(() =>
	project.value.status === 'processing' && missingCustomName.value
		? { severity: 'error' as const, message: messages.missingName }
		: null,
)
const customUrlMessage = computed(() =>
	project.value.status === 'processing' && missingCustomUrl.value
		? { severity: 'error' as const, message: messages.missingUrl }
		: null,
)
const canSave = computed(
	() =>
		hasPermission.value &&
		!missingCustomName.value &&
		!missingCustomUrl.value &&
		!saveValidation.messages.value.some((message) => message.severity === 'error'),
)

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	try {
		await saveLicense()
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

const licenseId = computed(() => {
	let id = ''

	if (
		(!current.value.hasSpdxLicense && current.value.license.friendly === 'Custom') ||
		current.value.license.short === 'All-Rights-Reserved' ||
		current.value.license.short === 'Unknown'
	) {
		id += 'LicenseRef-'
	}

	id += current.value.license.short
	if (current.value.license.requiresOnlyOrLater) {
		id += current.value.allowOrLater ? '-or-later' : '-only'
	}

	if (!current.value.hasSpdxLicense && current.value.license.friendly === 'Custom') {
		id = id.replaceAll(' ', '-')
	}

	return id
})
</script>
