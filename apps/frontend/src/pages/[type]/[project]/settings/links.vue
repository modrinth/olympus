<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<div class="flex min-w-0 flex-col gap-8">
			<section v-for="section in linkSections" :key="section.id" class="min-w-0">
				<div class="mb-2.5 flex flex-wrap items-center justify-between gap-3">
					<h2 class="m-0 text-2xl font-semibold">{{ section.title }}</h2>
					<TeleportOverflowMenu
						v-if="section.id === 'donations' && section.rows.length > 0"
						:options="donationPlatformOptions"
						:label="formatMessage(messages.addLink)"
						:disabled="saving || !hasPermission || donationPlatformOptions.length === 0"
						:icon-only="false"
						type="outlined"
						placement="bottom-end"
					>
						<PlusIcon aria-hidden="true" />
						{{ formatMessage(messages.addLink) }}
					</TeleportOverflowMenu>
				</div>
				<Table
					:columns="section.columns"
					:data="section.rows"
					row-key="id"
					:table-min-width="section.rows.length ? '36rem' : undefined"
				>
					<template #header-url="{ column }">
						<span class="ml-3.5">{{ column.label }}</span>
					</template>
					<template #empty-state>
						<EmptyState :description="formatMessage(messages.donationsDescription)" class="my-4">
							<template #actions>
								<TeleportOverflowMenu
									:options="donationPlatformOptions"
									:label="formatMessage(messages.addLink)"
									:disabled="saving || !hasPermission || donationPlatformOptions.length === 0"
									:icon-only="false"
									type="outlined"
									placement="bottom-start"
									class="-mt-4"
								>
									<PlusIcon aria-hidden="true" />
									{{ formatMessage(messages.addLink) }}
								</TeleportOverflowMenu>
							</template>
						</EmptyState>
					</template>
					<template #cell-name="{ row }">
						<label v-tooltip="row.description" :for="row.inputId" class="font-medium">
							{{ row.name }}
						</label>
					</template>
					<template #cell-url="{ row }">
						<div class="flex min-w-0 items-center gap-2">
							<Input
								:id="row.inputId"
								:model-value="row.value"
								:type="row.donation?.mode === 'username' ? 'text' : 'url'"
								:placeholder="
									row.donation
										? donationPlaceholder(row.donation)
										: formatMessage(messages.urlPlaceholder)
								"
								:maxlength="2048"
								:disabled="saving || !hasPermission"
								:aria-describedby="row.inputId + '-help'"
								autocapitalize="none"
								autocomplete="off"
								:spellcheck="false"
								wrapper-class="min-w-0 flex-1 !bg-surface-3 !border-surface-4 !text-contrast"
								input-class="!text-contrast"
								@update:model-value="changeLink(row, $event)"
							/>
							<Tabs
								v-if="row.donation?.id && usernamePrefixes[row.donation.id]"
								:value="row.donation.mode"
								:tabs="donationModeTabs"
								:disabled="saving || !hasPermission"
								color="gray"
								class="shrink-0"
								@update:value="changeDonationMode(row.donation, $event)"
							/>
							<IconButton
								v-tooltip="
									visitUrl(row.url)
										? formatMessage(messages.visitLink, { url: visitUrl(row.url) })
										: null
								"
								type="base"
								class="shrink-0"
								:label="formatMessage(messages.visitLink, { url: visitUrl(row.url) })"
								:disabled="!visitUrl(row.url)"
								@mousedown.stop
								@click.stop="visitLink(row.url)"
							>
								<ExternalIcon />
							</IconButton>
							<IconButton
								v-if="row.donation"
								v-tooltip="formatMessage(messages.removeDonation)"
								type="base"
								class="shrink-0"
								:label="formatMessage(messages.removeDonation)"
								:disabled="saving || !hasPermission"
								@click.stop="removeDonation(row.donation)"
							>
								<XIcon />
							</IconButton>
						</div>
						<span :id="row.inputId + '-help'" class="sr-only">{{ row.description }}</span>
						<div class="mt-2.5 empty:hidden">
							<template v-if="row.field">
								<ValidationMessage
									:check="savedFieldMessages(row.field)"
									:project-field="saved[row.field]"
									:current-field="current[row.field]"
								/>
								<ValidationMessage :check="saveValidation.forField(row.field)" />
							</template>
							<ValidationMessage v-else-if="row.donation" :check="donationMessages(row.donation)" />
						</div>
					</template>
				</Table>
			</section>
		</div>
		<ValidationMessage :check="otherSaveMessages" class="my-4" />
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="canSave"
			@reset="reset"
			@save="save"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	EmptyState,
	IconButton,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	Table,
	type TableColumn,
	Tabs,
	type TabsValue,
	TeleportOverflowMenu,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'

import ValidationMessage from '@/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import {
	type DonationInput,
	donationInput,
	donationUsernamePrefixes as usernamePrefixes,
	setDonationInput,
	toggleDonationInput,
} from '~/helpers/donation-links'
import { normalizeProjectUrl } from '~/helpers/project-url'

type EditableLinkField = 'discord' | 'issues' | 'site' | 'source' | 'store' | 'wiki'
type EditableLinks = Partial<Record<EditableLinkField, string>>
type ProjectLinkUrls = Labrinth.Projects.v3.Project['link_urls']

interface DonationRow extends DonationInput {
	key: number
}

type LinkTableRow = {
	id: string
	inputId: string
	name: string
	description: string
	value: string
	url: string
	field?: EditableLinkField
	donation?: DonationRow
}

const messages = defineMessages({
	title: { id: 'project.settings.links.title', defaultMessage: 'Links' },
	issues: { id: 'project.settings.links.issues', defaultMessage: 'Issue tracker' },
	issuesDescription: {
		id: 'project.settings.links.issues-description',
		defaultMessage: 'A place for users to report bugs, issues, and concerns about your project.',
	},
	source: { id: 'project.settings.links.source', defaultMessage: 'Source code' },
	sourceDescription: {
		id: 'project.settings.links.source-description',
		defaultMessage: 'A page/repository containing the source code for your project',
	},
	wiki: { id: 'project.settings.links.wiki', defaultMessage: 'Wiki page' },
	wikiDescription: {
		id: 'project.settings.links.wiki-description',
		defaultMessage: 'A page containing information, documentation, and help for the project.',
	},
	discord: { id: 'project.settings.links.discord', defaultMessage: 'Discord invite' },
	discordDescription: {
		id: 'project.settings.links.discord-description',
		defaultMessage: 'An invitation link to your Discord server.',
	},
	site: { id: 'project.settings.links.site', defaultMessage: 'Website' },
	siteDescription: {
		id: 'project.settings.links.site-description',
		defaultMessage: "Your server's official website.",
	},
	store: { id: 'project.settings.links.store', defaultMessage: 'Store' },
	storeDescription: {
		id: 'project.settings.links.store-description',
		defaultMessage: "A link to your server's store or shop.",
	},
	linkType: { id: 'project.settings.links.link-type', defaultMessage: 'Link type' },
	donationPlatform: { id: 'project.settings.links.donation-platform', defaultMessage: 'Platform' },
	donationLink: { id: 'project.settings.links.donation-link', defaultMessage: 'Link' },
	donations: { id: 'project.settings.links.donations', defaultMessage: 'Donation links' },
	addLink: { id: 'project.settings.links.add-link', defaultMessage: 'Add link' },
	removeDonation: {
		id: 'project.settings.links.remove-donation-link',
		defaultMessage: 'Remove donation link',
	},
	donationsDescription: {
		id: 'project.settings.links.donations-description',
		defaultMessage: 'Add donation links for users to support you directly.',
	},
	urlPlaceholder: {
		id: 'project.settings.links.url-placeholder',
		defaultMessage: 'Enter a valid URL',
	},
	donationUsernamePlaceholder: {
		id: 'project.settings.links.donation-username-placeholder',
		defaultMessage: 'Enter your {platform} username',
	},
	visitLink: {
		id: 'project.settings.links.visit-link',
		defaultMessage: 'Visit {url}',
	},
	donationUrl: {
		id: 'project.settings.links.donation-url',
		defaultMessage: 'URL',
	},
	donationUsername: {
		id: 'project.settings.links.donation-username',
		defaultMessage: 'Username',
	},
	updatedTitle: { id: 'project.settings.links.updated-title', defaultMessage: 'Links updated' },
	updated: {
		id: 'project.settings.links.updated',
		defaultMessage: 'Your links have been updated.',
	},
	serverDiscord: { id: 'project.settings.links.server-discord', defaultMessage: 'Discord' },
	serverWikiDescription: {
		id: 'project.settings.links.server-wiki-description',
		defaultMessage: 'A page containing information, documentation, and help for the server.',
	},
	serverUpdated: {
		id: 'project.settings.links.server-updated',
		defaultMessage: 'Your server links have been updated.',
	},
})

const fieldMessages = computed(() => ({
	issues: { title: messages.issues, description: messages.issuesDescription },
	source: { title: messages.source, description: messages.sourceDescription },
	wiki: {
		title: messages.wiki,
		description: isServerProject.value ? messages.serverWikiDescription : messages.wikiDescription,
	},
	discord: {
		title: isServerProject.value ? messages.serverDiscord : messages.discord,
		description: messages.discordDescription,
	},
	site: { title: messages.site, description: messages.siteDescription },
	store: { title: messages.store, description: messages.storeDescription },
}))

const { formatMessage } = useVIntl()
const tags = useGeneratedState()
const { projectV3: project, currentMember, patchProjectV3 } = injectProjectPageContext()
const { addNotification } = injectNotificationManager()
useProjectSettingsHeadTitle(commonProjectSettingsMessages.links)

const isServerProject = computed(() => project.value.minecraft_server != null)
const visibleFields = computed<EditableLinkField[]>(() =>
	isServerProject.value
		? ['site', 'store', 'wiki', 'discord']
		: ['issues', 'source', 'wiki', 'discord'],
)
const hasPermission = computed(
	() =>
		isAdmin(currentMember.value?.user) ||
		((currentMember.value?.permissions ?? 0) & (1 << 2)) !== 0,
)
const {
	saved,
	current,
	reset: resetFields,
} = useSavable<EditableLinks>(
	() =>
		Object.fromEntries(
			visibleFields.value.map((field) => [field, project.value.link_urls?.[field]?.url ?? '']),
		),
	() => {},
)

let nextRowKey = 0

function makeRow(id?: string, url = ''): DonationRow {
	return { key: nextRowKey++, ...donationInput(id, url) }
}

function donationRowsFromLinks(links?: ProjectLinkUrls): DonationRow[] {
	return tags.value.donationPlatforms.flatMap((platform) =>
		links?.[platform.short]?.url ? [makeRow(platform.short, links[platform.short].url)] : [],
	)
}

const donationLinks = ref(donationRowsFromLinks(project.value.link_urls))
const donationPlatformOptions = computed(() =>
	tags.value.donationPlatforms
		.filter((platform) => !donationLinks.value.some((row) => row.id === platform.short))
		.map((platform) => ({
			id: platform.short,
			label: platform.name,
			action: () => addDonation(platform.short),
		})),
)
const donationModeTabs = computed(() => [
	{ value: 'username', label: formatMessage(messages.donationUsername) },
	{ value: 'url', label: formatMessage(messages.donationUrl) },
])
const linkColumns = computed<TableColumn<'name' | 'url'>[]>(() => [
	{
		key: 'name',
		label: formatMessage(messages.linkType),
		width: '12rem',
		cellClass: 'pr-3 py-3',
	},
	{
		key: 'url',
		label: formatMessage(messages.donationUrl),
		cellClass: 'py-3 pl-1',
	},
])
const linkRows = computed<LinkTableRow[]>(() => [
	...visibleFields.value.map((field) => ({
		id: field,
		inputId: 'project-link-' + field,
		name: formatMessage(fieldMessages.value[field].title),
		description: formatMessage(fieldMessages.value[field].description),
		value: current.value[field] ?? '',
		url: current.value[field] ?? '',
		field,
	})),
	...(isServerProject.value
		? []
		: donationLinks.value.map((donation) => ({
				id: 'donation-' + donation.key,
				inputId: 'donation-link-' + donation.key,
				name: donationPlatformTitle(donation) ?? '',
				description: formatMessage(messages.donationsDescription),
				value: donation.input,
				url: donation.url,
				donation,
			}))),
])
const linkSections = computed(() => [
	{
		id: 'links',
		title: formatMessage(messages.title),
		columns: linkColumns.value,
		rows: linkRows.value.filter((row) => row.field),
	},
	...(isServerProject.value
		? []
		: [
				{
					id: 'donations',
					title: formatMessage(messages.donations),
					columns: linkColumns.value.map((column) => ({
						...column,
						label: formatMessage(
							column.key === 'name' ? messages.donationPlatform : messages.donationLink,
						),
					})),
					rows: linkRows.value.filter((row) => row.donation),
				},
			]),
])
const savedDonations = computed(() =>
	Object.fromEntries(
		tags.value.donationPlatforms.flatMap((platform) => {
			const url = project.value.link_urls?.[platform.short]?.url
			return url ? [[platform.short, url]] : []
		}),
	),
)
const currentDonations = computed(() =>
	Object.fromEntries(
		donationLinks.value.filter((row) => row.id && row.url).map((row) => [row.id!, row.url]),
	),
)
const original = computed(() => ({
	...saved.value,
	donations: JSON.stringify(
		isServerProject.value ? [] : Object.entries(savedDonations.value).sort(),
	),
}))
const modified = computed(() => ({
	...current.value,
	donations: JSON.stringify(
		isServerProject.value
			? []
			: donationLinks.value
					.filter((row) => row.id && row.url)
					.map((row) => [row.id ?? '', row.url])
					.sort(),
	),
}))
const hasChanges = computed(() => JSON.stringify(original.value) !== JSON.stringify(modified.value))
const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)
const saveValidation = useProjectSaveValidation(() => modified.value)
const otherSaveMessages = computed(() =>
	saveValidation.withoutFields([
		...visibleFields.value,
		...donationLinks.value.flatMap((row) => (row.id ? [row.id] : [])),
	]),
)

const fieldValidation = useProjectNagMessages('link-field')
const sourceRequirement = useProjectNagMessages('source-availability', 'source')

function savedFieldMessages(field: string) {
	return [
		...fieldValidation.value.filter((message) => message.values?.field === field),
		...(field === 'source' ? sourceRequirement.value : []),
	]
}

function donationMessages(row: DonationRow) {
	const rejected = row.id ? saveValidation.forField(row.id) : []
	return rejected.length
		? rejected
		: row.id && row.url === savedDonations.value[row.id]
			? savedFieldMessages(row.id)
			: []
}

function donationPlatformTitle(row: DonationRow) {
	return tags.value.donationPlatforms.find((platform) => platform.short === row.id)?.name ?? row.id
}

function donationPlaceholder(row: DonationRow) {
	if (row.mode !== 'username') return formatMessage(messages.urlPlaceholder)
	return formatMessage(messages.donationUsernamePlaceholder, {
		platform:
			row.id === 'github' ? 'GitHub' : row.id === 'paypal' ? 'PayPal' : donationPlatformTitle(row),
	})
}

async function addDonation(platform: string) {
	if (saving.value || !hasPermission.value) return
	if (!donationPlatformOptions.value.some((option) => option.id === platform)) return
	const row = makeRow(platform)
	donationLinks.value.push(row)
	await nextTick()
	document.getElementById('donation-link-' + row.key)?.focus()
}

function removeDonation(row: DonationRow) {
	if (saving.value || !hasPermission.value) return
	donationLinks.value = donationLinks.value.filter((link) => link.key !== row.key)
}

function changeLink(row: LinkTableRow, value: string | number | undefined) {
	if (row.donation) setDonationInput(row.donation, value ?? '')
	else if (row.field) current.value[row.field] = String(value ?? '')
}

function changeDonationMode(row: DonationRow, mode: TabsValue) {
	if (saving.value || !hasPermission.value || row.mode === mode) return
	if (mode === 'username' || mode === 'url') toggleDonationInput(row)
}

function visitUrl(value: string) {
	try {
		const url = new URL(normalizeProjectUrl(value))
		return ['http:', 'https:'].includes(url.protocol) ? url.href : ''
	} catch {
		return ''
	}
}

function visitLink(value: string) {
	const url = visitUrl(value)
	if (url) window.open(url, '_blank', 'noopener,noreferrer')
}

function reset() {
	resetFields()
	donationLinks.value = donationRowsFromLinks(project.value.link_urls)
	saveValidation.clear()
}

const patchData = computed<Record<string, string | null>>(() => {
	const data: Record<string, string | null> = {}
	for (const field of visibleFields.value) {
		const value = current.value[field] ?? ''
		if (value !== saved.value[field]) data[field] = normalizeProjectUrl(value) || null
	}
	if (!isServerProject.value) {
		for (const platform of tags.value.donationPlatforms) {
			const url = currentDonations.value[platform.short] ?? ''
			if (url !== (savedDonations.value[platform.short] ?? '')) {
				data[platform.short] = normalizeProjectUrl(url) || null
			}
		}
	}
	return data
})
const canSave = computed(
	() =>
		hasPermission.value &&
		hasChanges.value &&
		Object.keys(patchData.value).length > 0 &&
		!saveValidation.messages.value.some((message) => message.severity === 'error'),
)
const saving = ref(false)

async function save() {
	if (!canSave.value || saving.value) return
	const submittedState = saveValidation.snapshot()
	saving.value = true
	try {
		await patchProjectV3({ link_urls: patchData.value }, true, true)
		reset()
		addNotification({
			title: formatMessage(messages.updatedTitle),
			text: formatMessage(isServerProject.value ? messages.serverUpdated : messages.updated),
			type: 'success',
		})
	} catch (error) {
		saveValidation.capture(error, submittedState)
	} finally {
		saving.value = false
	}
}
</script>
