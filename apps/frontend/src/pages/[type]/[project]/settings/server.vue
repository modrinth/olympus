<template>
	<div>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<section class="universal-card">
			<div class="flex flex-col gap-6">
				<div class="text-2xl font-semibold text-contrast">
					{{ formatMessage(messages.serverDetailsHeading) }}
				</div>

				<!-- Region -->
				<div class="max-w-[600px]">
					<label for="server-region" class="w-fit">
						<span class="label__title">{{ formatMessage(messages.regionLabel) }}</span>
					</label>
					<Combobox
						id="server-region"
						v-model="region"
						:options="regionOptions"
						searchable
						:placeholder="formatMessage(messages.selectRegionPlaceholder)"
						:disabled="!hasPermission"
					/>
					<ValidationMessage
						:check="regionValidation"
						:project-field="projectV3?.minecraft_server?.region ?? ''"
						:current-field="region"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('server-region')" class="mt-2" />
				</div>

				<!-- Language -->
				<div class="max-w-[600px]">
					<label for="server-language" class="block w-fit">
						<span class="label__title"
							>{{ formatMessage(messages.languagesLabel) }}
							<span class="font-normal text-secondary"
								>({{ formatMessage(messages.optionalLabel) }})</span
							></span
						>
					</label>
					<MultiSelect
						id="server-language"
						v-model="languages"
						:options="languageOptions"
						searchable
						include-select-all-option
						:max-tag-rows="2"
						:placeholder="formatMessage(messages.selectLanguagesPlaceholder)"
						:disabled="!hasPermission"
					/>
					<ValidationMessage
						:check="languageValidation"
						:project-field="
							JSON.stringify([...(projectV3?.minecraft_server?.languages ?? [])].sort())
						"
						:current-field="JSON.stringify([...languages].sort())"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('server-languages')" class="mt-2" />
				</div>

				<!-- Java Address -->
				<div class="max-w-[600px]">
					<div class="flex items-center justify-between">
						<label for="java-address" class="block w-fit">
							<span class="label__title !m-0 !text-contrast">{{
								formatMessage(messages.javaAddressLabel)
							}}</span>
						</label>
					</div>
					<div
						class="mt-2 flex items-center gap-2 text-sm"
						@focusout="
							() => {
								if (!lastPingAddressChanged && javaPingResult) return
								pingJavaServer()
							}
						"
					>
						<Input
							id="java-address"
							v-model="javaAddress"
							:placeholder="formatMessage(messages.enterAddressPlaceholder)"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
					</div>
					<div
						v-if="javaAddress"
						class="mt-2 flex gap-1.5"
						:class="{
							'items-center': javaPingResult?.online,
							'items-start': javaPingResult && !javaPingResult.online,
						}"
					>
						<IconButton
							v-if="(javaAddress && javaPingResult) || javaPingLoading"
							v-tooltip="formatMessage(messages.refreshPingTooltip)"
							class="!size-6"
							type="quiet"
							color="orange"
							size="xs"
							:label="formatMessage(messages.refreshPingTooltip)"
							:disabled="javaPingLoading"
							@click="pingJavaServer"
						>
							<SpinnerIcon v-if="javaPingLoading" class="animate-spin" />
							<RefreshCwIcon v-else />
						</IconButton>
						<div
							v-if="javaPingResult !== null && !javaPingLoading && javaPingResult.online"
							class="mt-0.5 flex items-center gap-1.5 text-green"
						>
							{{ formatMessage(messages.serverOnline) }}
							<template v-if="javaPingResult.latency">
								{{
									formatMessage(messages.latencyLabel, {
										latency: javaPingResult.latency,
									})
								}}
							</template>
						</div>
						<div v-else-if="javaPingResult !== null && !javaPingLoading" class="mt-0.5 text-orange">
							<IntlFormatted :message-id="messages.pingFailedMessage">
								<template #support-link="{ children }">
									<a
										class="inline underline"
										href="https://support.modrinth.com"
										target="_blank"
										rel="noopener noreferrer"
										><component :is="() => normalizeChildren(children)"
									/></a>
								</template>
							</IntlFormatted>
						</div>
					</div>
					<div v-else class="mt-2 text-sm">
						<IntlFormatted :message-id="messages.srvRecordsHint">
							<template #srv-tooltip="{ children }"
								><component :is="() => normalizeChildren(children)" /><InfoIcon
									v-tooltip="{
										content: formatMessage(messages.srvRecordsTooltip),
										popperClass: 'max-w-xs',
									}"
							/></template>
						</IntlFormatted>
					</div>
					<ValidationMessage
						:check="javaAddressValidation"
						:project-field="projectV3?.minecraft_java_server?.address ?? ''"
						:current-field="javaAddress.trim()"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('java-address')" class="mt-2" />
				</div>

				<!-- Bedrock Address -->
				<div class="max-w-[600px]">
					<label for="bedrock-address" class="block w-fit">
						<span class="label__title !text-contrast"
							>{{ formatMessage(messages.bedrockAddressLabel) }}
							<span class="font-normal text-secondary"
								>({{ formatMessage(messages.optionalLabel) }})</span
							>
						</span>
					</label>
					<div class="mt-2 flex items-center gap-2">
						<Input
							id="bedrock-address"
							v-model="bedrockAddress"
							:placeholder="formatMessage(messages.enterAddressPlaceholder)"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
					</div>
				</div>

				<div>
					<CompatibilityCard />
					<ValidationMessage :check="compatibilityValidation" class="mt-2" />
					<ValidationMessage
						:check="saveValidation.forField('server-compatibility')"
						class="mt-2"
					/>
				</div>
			</div>
		</section>

		<ValidationMessage
			:check="
				saveValidation.withoutFields([
					'server-region',
					'server-languages',
					'java-address',
					'server-compatibility',
				])
			"
			class="my-4"
		/>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="!saveValidation.hasErrors.value"
			@reset="resetChanges"
			@save="handleSave"
		/>
	</div>
</template>

<script setup>
import { InfoIcon, RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Combobox,
	commonProjectSettingsMessages,
	ConfirmLeaveModal,
	defineMessages,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	IntlFormatted,
	MultiSelect,
	normalizeChildren,
	SERVER_LANGUAGES,
	SERVER_REGIONS,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'

import CompatibilityCard from '~/components/ui/project-settings/CompatibilityCard.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'

const PING_TIMEOUT_MS = 5000

const { formatMessage, locale } = useVIntl()

const messages = defineMessages({
	serverDetailsHeading: {
		id: 'project.settings.server.details-heading',
		defaultMessage: 'Server details',
	},
	regionLabel: {
		id: 'project.settings.server.region-label',
		defaultMessage: 'Region',
	},
	selectRegionPlaceholder: {
		id: 'project.settings.server.select-region-placeholder',
		defaultMessage: 'Select region',
	},
	languagesLabel: {
		id: 'project.settings.server.languages-label',
		defaultMessage: 'Languages',
	},
	optionalLabel: {
		id: 'project.settings.server.optional-label',
		defaultMessage: 'optional',
	},
	selectLanguagesPlaceholder: {
		id: 'project.settings.server.select-languages-placeholder',
		defaultMessage: 'Select languages',
	},
	javaAddressLabel: {
		id: 'project.settings.server.java-address-label',
		defaultMessage: 'Java address',
	},
	enterAddressPlaceholder: {
		id: 'project.settings.server.enter-address-placeholder',
		defaultMessage: 'Enter address',
	},
	refreshPingTooltip: {
		id: 'project.settings.server.refresh-ping-tooltip',
		defaultMessage: 'Refresh ping',
	},
	serverOnline: {
		id: 'project.settings.server.server-online',
		defaultMessage: 'Server is online!',
	},
	latencyLabel: {
		id: 'project.settings.server.latency-label',
		defaultMessage: 'Latency: {latency}ms',
	},
	pingFailedMessage: {
		id: 'project.settings.server.ping-failed-message',
		defaultMessage:
			"We couldn't ping this server. It may be blocked by your host so try refreshing a few times. If it still doesn't respond please <support-link>contact support</support-link>.",
	},
	srvRecordsHint: {
		id: 'project.settings.server.srv-records-hint',
		defaultMessage:
			"If you have <srv-tooltip>[SRV records]</srv-tooltip>, you do not need to add a port. Otherwise if you have a port which isn't 25565, you can include it as :12345",
	},
	srvRecordsTooltip: {
		id: 'project.settings.server.srv-records-tooltip',
		defaultMessage:
			'The address you enter here may have DNS SRV records _minecraft._tcp.(your domain) which point to your Minecraft server address and port.',
	},
	bedrockAddressLabel: {
		id: 'project.settings.server.bedrock-address-label',
		defaultMessage: 'Bedrock address',
	},
	cannotSaveTitle: {
		id: 'project.settings.server.cannot-save-title',
		defaultMessage: 'Cannot save',
	},
	cannotSaveText: {
		id: 'project.settings.server.cannot-save-text',
		defaultMessage:
			'The Java server must be reachable before saving. Please ensure the ping succeeds.',
	},
})

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()

const regionValidation = useProjectNagMessages('server-region')
const languageValidation = useProjectNagMessages('server-languages')
const javaAddressValidation = useProjectNagMessages('java-address')
const compatibilityValidation = useProjectNagMessages('server-compatibility')

useProjectSettingsHeadTitle(commonProjectSettingsMessages.server)

const javaAddress = ref('')
const bedrockAddress = ref('')
const bedrockPort = ref(19132)
const region = ref('')
const languages = ref([])

const javaPingLoading = ref(false)
const javaPingResult = ref(null)

const lastPingedAddress = ref('')

const lastPingAddressChanged = computed(() => {
	return javaAddress.value.trim() !== lastPingedAddress.value
})

let pingDebounceTimer = null

watch(javaAddress, () => {
	clearTimeout(pingDebounceTimer)
	pingDebounceTimer = setTimeout(() => {
		pingJavaServer()
	}, 500)
})

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (
		isAdminUser.value || ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
	)
})

async function pingJavaServer() {
	const address = javaAddress.value?.trim()
	if (!address) {
		javaPingResult.value = null
		return
	}

	javaPingLoading.value = true
	javaPingResult.value = null

	try {
		await Promise.race([
			client.labrinth.server_ping_internal.pingMinecraftJava({
				address,
				timeout_ms: PING_TIMEOUT_MS,
			}),
			new Promise((_, reject) =>
				setTimeout(() => reject(new Error('Ping timed out')), PING_TIMEOUT_MS),
			),
		])
		javaPingResult.value = { online: true, latency: null }
	} catch {
		javaPingResult.value = { online: false, latency: null }
	} finally {
		javaPingLoading.value = false
		lastPingedAddress.value = address
	}
}

function initFromProjectV3(v3) {
	if (!v3) return
	javaAddress.value = v3.minecraft_java_server?.address ?? ''
	bedrockAddress.value = v3.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = v3.minecraft_bedrock_server?.port ?? 19132
	region.value = v3.minecraft_server?.region ?? ''
	languages.value = v3.minecraft_server?.languages ?? []

	pingJavaServer()
}

// initialize projectV3 values once
if (projectV3.value) {
	initFromProjectV3(projectV3.value)
} else {
	const stop = watch(
		() => projectV3.value,
		(v3) => {
			if (!v3) return
			initFromProjectV3(v3)
			stop()
		},
	)
}

const regionOptions = computed(() =>
	Object.entries(SERVER_REGIONS)
		.sort(([_, a], [__, b]) => {
			const aFormatted = formatMessage(a)
			const bFormatted = formatMessage(b)
			return aFormatted.localeCompare(bFormatted, locale.value)
		})
		.map(([code, name]) => ({
			value: code,
			label: formatMessage(name),
		})),
)

const languageOptions = computed(() =>
	Object.entries(SERVER_LANGUAGES)
		.sort(([_, a], [__, b]) => {
			const aFormatted = formatMessage(a)
			const bFormatted = formatMessage(b)
			return aFormatted.localeCompare(bFormatted, locale.value)
		})
		.map(([code, name]) => ({
			value: code,
			label: formatMessage(name),
		})),
)

const javaServerPatchData = computed(() => {
	const addressChanged =
		javaAddress.value.trim() !== (projectV3.value?.minecraft_java_server?.address ?? '')
	if (addressChanged) {
		return {
			address: javaAddress.value.trim(),
		}
	}

	return {}
})

const bedrockServerPatchData = computed(() => {
	const origBedrock = projectV3.value?.minecraft_bedrock_server
	if (bedrockAddress.value !== (origBedrock?.address ?? '')) {
		return {
			address: bedrockAddress.value.trim(),
		}
	}

	return {}
})

const serverPatchData = computed(() => {
	const origServer = projectV3.value?.minecraft_server
	const regionChanged = region.value && region.value !== origServer?.region
	const languagesChanged =
		JSON.stringify([...languages.value].sort()) !==
		JSON.stringify([...(origServer?.languages ?? [])].sort())

	if (regionChanged || languagesChanged) {
		return {
			...origServer,
			...(regionChanged ? { region: region.value } : {}),
			...(languagesChanged ? { languages: languages.value } : {}),
		}
	}

	return {}
})

const v3PatchData = computed(() => {
	const data = {}
	if (Object.keys(serverPatchData.value).length > 0) {
		data.minecraft_server = serverPatchData.value
	}
	if (Object.keys(javaServerPatchData.value).length > 0) {
		data.minecraft_java_server = javaServerPatchData.value
	}
	if (Object.keys(bedrockServerPatchData.value).length > 0) {
		data.minecraft_bedrock_server = bedrockServerPatchData.value
	}
	return data
})

const saving = ref(false)

const original = computed(() => ({
	javaAddress: projectV3.value?.minecraft_java_server?.address ?? '',
	bedrockAddress: projectV3.value?.minecraft_bedrock_server?.address ?? '',
	bedrockPort: projectV3.value?.minecraft_bedrock_server?.port ?? 19132,
	region: projectV3.value?.minecraft_server?.region ?? '',
	languages: projectV3.value?.minecraft_server?.languages ?? [],
}))

const modified = computed(() => ({
	javaAddress: javaAddress.value,
	bedrockAddress: bedrockAddress.value,
	bedrockPort: bedrockPort.value,
	region: region.value,
	languages: languages.value,
}))

const hasChanges = computed(() =>
	Object.keys(original.value).some((key) => {
		const a = original.value[key]
		const b = modified.value[key]
		if (Array.isArray(a) && Array.isArray(b)) {
			return a.length !== b.length || a.some((v, i) => v !== b[i])
		}
		return a !== b
	}),
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const saveValidation = useProjectSaveValidation(() => modified.value)

function resetChanges() {
	saveValidation.clear()
	javaAddress.value = projectV3.value?.minecraft_java_server?.address ?? ''
	bedrockAddress.value = projectV3.value?.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = projectV3.value?.minecraft_bedrock_server?.port ?? 19132
	region.value = projectV3.value?.minecraft_server?.region ?? ''
	languages.value = projectV3.value?.minecraft_server?.languages ?? []
}

async function handleSave() {
	if (saving.value || saveValidation.hasErrors.value) return
	const submittedState = saveValidation.snapshot()
	if (!isAdminUser.value && javaAddress.value.trim() && !javaPingResult.value?.online) {
		addNotification({
			title: formatMessage(messages.cannotSaveTitle),
			text: formatMessage(messages.cannotSaveText),
			type: 'error',
		})
		return
	}

	saving.value = true
	try {
		const hasV3Changes = Object.keys(v3PatchData.value).length > 0
		if (hasV3Changes) {
			await patchProjectV3(v3PatchData.value, false, true)
			saveValidation.clear()
		}
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	} finally {
		saving.value = false
	}
}
</script>
