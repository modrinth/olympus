<template>
	<template v-if="project && projectV3Loaded">
		<Teleport v-if="flags.projectBackground" to="#fixed-background-teleport">
			<ProjectBackgroundGradient :project="project" />
		</Teleport>
		<template v-if="isSettings">
			<div v-if="canAccessSettings" class="normal-page no-sidebar" :class="`align-${marginTarget}`">
				<div class="normal-page__header mb-6">
					<PageHeader :title="project.title" :row-class="'items-center'">
						<template #leading>
							<ButtonLink
								v-if="settingsBackDestination"
								v-tooltip="settingsBackDestination.label"
								:to="settingsBackDestination.to"
								size="lg"
								class="!w-10 !rounded-full !px-0"
							>
								<LeftArrowIcon />
							</ButtonLink>
							<Avatar
								:src="project.icon_url"
								:raw-src="project.raw_icon_url"
								:tint-by="project.id"
								size="64px"
							/>
						</template>
						<template #metadata>
							<PageHeaderMetadata>
								<PageHeaderMetadataItem>
									{{
										formatMessage(messages.editingProject, {
											projectType: projectTypeDisplay.toLowerCase(),
										})
									}}
								</PageHeaderMetadataItem>
								<PageHeaderMetadataItem>
									{{
										formatMessage(commonMessages.projectCreated, {
											date: formatRelativeTime(project.published),
										})
									}}
								</PageHeaderMetadataItem>
							</PageHeaderMetadata>
						</template>
						<template #actions>
							<PageHeaderActions>
								<ButtonLink :to="`${projectPath}`">
									<CompassIcon />
									{{ formatMessage(messages.projectPage) }}
								</ButtonLink>
							</PageHeaderActions>
						</template>
					</PageHeader>
					<ProjectMemberHeader
						v-if="currentMember && false"
						:project="project"
						:versions="versions"
						:current-member="currentMember"
						:is-settings="isSettings"
						:set-processing="setProcessing"
						:all-members="allMembers"
						:update-members="invalidateProject"
						:auth="auth"
						:tags="tags"
					/>
				</div>
				<div class="normal-page__content">
					<NuxtPage />
				</div>
			</div>
		</template>

		<div v-else>
			<OpenInAppModal ref="openInAppModal" />
			<div
				class="over-the-top-download-animation"
				:class="{ 'animation-hidden': !overTheTopDownloadAnimation }"
			>
				<div>
					<div
						class="animation-ring-3 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-40"
					></div>

					<div
						class="animation-ring-2 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight opacity-60"
					></div>

					<div
						class="animation-ring-1 flex items-center justify-center rounded-full border-4 border-solid border-brand bg-brand-highlight"
					>
						<DownloadIcon class="h-20 w-20 text-contrast" />
					</div>
				</div>
			</div>
			<ProjectDownloadModal
				ref="downloadModal"
				:project-id="projectId"
				:download-reason="downloadReason"
				@download="triggerDownloadAnimation"
			/>
			<CollectionCreateModal ref="modal_collection" :project-ids="[project.id]" />
			<ModpackScanModal ref="scanModal" :project_id="project.id" />

			<div
				v-if="projectInstallContext && !isSettings"
				ref="stickyInstallHeaderRef"
				class="sticky top-0 z-20 mx-auto max-w-[80rem] border-0 border-solid border-divider bg-surface-1 px-6 pt-4"
				:class="[isInstallHeaderStuck ? 'border-t' : '']"
			>
				<BrowseInstallHeader
					:install-context="projectHeaderInstallContext"
					divider
					bottom-padding
				/>
			</div>
			<SelectedProjectsFloatingBar
				v-if="projectInstallContext && !isSettings"
				:install-context="projectInstallContext"
			/>
			<div
				class="new-page sidebar"
				:class="[
					{
						'alt-layout': cosmetics.leftContentLayout,
						'checklist-open':
							showModerationChecklist &&
							!collapsedModerationChecklist &&
							!flags.alwaysShowChecklistAsPopup,
						'checklist-collapsed':
							showModerationChecklist &&
							collapsedModerationChecklist &&
							!flags.alwaysShowChecklistAsPopup,
					},
					`align-${marginTarget}`,
				]"
			>
				<div
					class="normal-page__header relative mb-4"
					:class="projectInstallContext && !isSettings ? 'mt-0' : 'mt-4'"
				>
					<div class="mb-6">
						<ModerationProjectNags
							v-if="
								projectV3 &&
								currentMember &&
								(projectV3.status === 'draft' ||
									projectV3.status === 'processing' ||
									tags.rejectedStatuses.includes(projectV3.status))
							"
							:project="project"
							:project-v3="projectV3"
							:versions="versions ?? undefined"
							:current-member="currentMember"
							:collapsed="collapsedChecklist"
							:route-name="route.name"
							:tags="tags"
							:validation-nags="projectValidation?.nags ?? []"
							:validation-loading="reviewSubmissionLoading"
							:validation-available="projectValidation !== null"
							:submit-project="setProcessing"
							@toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
						/>
					</div>
					<ProjectPageHeader
						v-if="projectV3Loaded"
						:project="project"
						:project-v3="projectV3"
						:show-status-badge="!!currentMember || projectV3.status !== 'approved'"
						@category="(category) => router.push(`${projectSearchUrl}?f=categories:${category}`)"
					>
						<template #actions>
							<ButtonLink
								v-if="auth.user && currentMember"
								v-tooltip="formatMessage(messages.editProject)"
								type="colored"
								color="brand"
								size="xl"
								:to="`${projectPath}/settings`"
								class="!w-12 !rounded-full !px-0 !font-bold lg:!hidden"
							>
								<SettingsIcon />
							</ButtonLink>
							<ButtonLink
								v-if="auth.user && currentMember"
								type="colored"
								color="brand"
								size="xl"
								:to="`${projectPath}/settings`"
								class="!font-bold max-lg:!hidden"
							>
								<SettingsIcon />
								{{ formatMessage(messages.editProject) }}
							</ButtonLink>

							<div class="hidden sm:contents">
								<IconButton
									v-if="!isServerProject && auth.user && currentMember"
									v-tooltip="formatMessage(commonMessages.downloadButton)"
									:type="projectHeaderPrimaryColor === 'brand' ? 'colored' : 'base'"
									:color="projectHeaderPrimaryColor === 'brand' ? 'brand' : undefined"
									size="xl"
									:label="formatMessage(commonMessages.downloadButton)"
									@click="handleProjectHeaderPrimary"
								>
									<DownloadIcon />
								</IconButton>
								<Button
									v-else-if="!isServerProject"
									:type="projectHeaderPrimaryColor === 'brand' ? 'colored' : 'base'"
									:color="projectHeaderPrimaryColor === 'brand' ? 'brand' : undefined"
									size="xl"
									@click="handleProjectHeaderPrimary"
								>
									<DownloadIcon />
									{{ formatMessage(commonMessages.downloadButton) }}
								</Button>
								<IconButton
									v-if="isServerProject && auth.user && currentMember"
									v-tooltip="formatMessage(commonMessages.playButton)"
									:type="projectHeaderPrimaryColor === 'brand' ? 'colored' : 'base'"
									:color="projectHeaderPrimaryColor === 'brand' ? 'brand' : undefined"
									size="xl"
									:label="formatMessage(commonMessages.playButton)"
									@click="handleProjectHeaderPrimary"
								>
									<PlayIcon />
								</IconButton>
								<Button
									v-else-if="isServerProject"
									:type="projectHeaderPrimaryColor === 'brand' ? 'colored' : 'base'"
									:color="projectHeaderPrimaryColor === 'brand' ? 'brand' : undefined"
									size="xl"
									@click="handleProjectHeaderPrimary"
								>
									<PlayIcon />
									{{ formatMessage(commonMessages.playButton) }}
								</Button>
							</div>

							<div class="contents sm:hidden">
								<IconButton
									v-if="!isServerProject"
									:type="projectHeaderPrimaryColor === 'brand' ? 'colored' : 'base'"
									:color="projectHeaderPrimaryColor === 'brand' ? 'brand' : undefined"
									size="xl"
									:label="formatMessage(commonMessages.downloadButton)"
									class="flex sm:hidden"
									@click="handleProjectHeaderPrimary"
								>
									<DownloadIcon />
								</IconButton>
								<IconButton
									v-else
									:type="projectHeaderPrimaryColor === 'brand' ? 'colored' : 'base'"
									:color="projectHeaderPrimaryColor === 'brand' ? 'brand' : undefined"
									size="xl"
									:label="formatMessage(commonMessages.playButton)"
									class="flex sm:hidden"
									@click="handleProjectHeaderPrimary"
								>
									<PlayIcon />
								</IconButton>
							</div>

							<Tooltip
								v-if="
									showProjectHeaderCreateServerAction && flags.showProjectPageCreateServersTooltip
								"
								theme="dismissable-prompt"
								class="inline-flex"
								open
								placement="bottom-start"
							>
								<ButtonLink
									v-tooltip="formatMessage(messages.createServerTooltip)"
									size="xl"
									:to="projectHeaderCreateServerTo"
									:aria-label="formatMessage(messages.serversPromoTitle)"
									class="!w-12 !rounded-full !px-0"
									@click="dismissProjectHeaderCreateServerPrompt"
								>
									<ServerPlusIcon />
								</ButtonLink>
								<template #popper>
									<div class="grid max-w-[18rem] gap-2">
										<div class="flex items-center justify-between gap-4">
											<div class="flex items-center gap-2">
												<h3 class="m-0 text-base font-bold text-contrast">
													{{ formatMessage(messages.serversPromoTitle) }}
												</h3>
												<span
													class="rounded-full bg-brand-highlight px-2 py-0.5 text-xs font-bold text-brand"
												>
													{{ formatMessage(commonMessages.newBadge) }}
												</span>
											</div>
											<IconButton
												v-tooltip="formatMessage(messages.dontShowAgain)"
												class="!size-6"
												size="xs"
												:label="formatMessage(messages.dontShowAgain)"
												@click="dismissProjectHeaderCreateServerPrompt"
											>
												<XIcon aria-hidden="true" />
											</IconButton>
										</div>
										<p class="m-0 text-sm font-medium leading-tight text-secondary">
											{{ formatMessage(messages.serversPromoDescription) }}
										</p>
										<p class="m-0 text-sm font-semibold text-contrast">
											<IntlFormatted
												:message-id="messages.serversPromoPricing"
												:values="{ price: formatPrice(500, 'USD', true) }"
											>
												<template #small="{ children }">
													<small><component :is="() => children" /></small>
												</template>
											</IntlFormatted>
										</p>
									</div>
								</template>
							</Tooltip>
							<ButtonLink
								v-else-if="showProjectHeaderCreateServerAction"
								v-tooltip="formatMessage(messages.createServerTooltip)"
								size="xl"
								:to="projectHeaderCreateServerTo"
								:aria-label="formatMessage(messages.serversPromoTitle)"
								class="!w-12 !rounded-full !px-0"
								@click="dismissProjectHeaderCreateServerPrompt"
							>
								<ServerPlusIcon />
							</ButtonLink>

							<ClientOnly>
								<IconButton
									v-if="auth.user"
									v-tooltip="
										following
											? formatMessage(commonMessages.unfollowButton)
											: formatMessage(commonMessages.followButton)
									"
									size="xl"
									:label="
										following
											? formatMessage(commonMessages.unfollowButton)
											: formatMessage(commonMessages.followButton)
									"
									@click="followProjectFromHeader"
								>
									<HeartIcon :fill="following ? 'currentColor' : 'none'" />
								</IconButton>
								<ButtonLink
									v-else
									v-tooltip="formatMessage(commonMessages.followButton)"
									size="xl"
									:to="signInRouteObj"
									class="!w-12 !rounded-full !px-0"
								>
									<HeartIcon aria-hidden="true" />
								</ButtonLink>
								<template #fallback>
									<ButtonLink
										v-tooltip="formatMessage(commonMessages.followButton)"
										size="xl"
										:to="signInRouteObj"
										class="!w-12 !rounded-full !px-0"
									>
										<HeartIcon aria-hidden="true" />
									</ButtonLink>
								</template>
							</ClientOnly>

							<ProjectCollectionSaveButton
								:auth-user="auth.user"
								:sign-in-route="signInRouteObj"
								:project-id="project.id"
								:collections="collections"
								:saved="collections.some((x) => x.projects.includes(project.id))"
								:base-id="baseId"
								:no-collections-label="formatMessage(messages.noCollectionsFound)"
								:create-new-collection-label="formatMessage(messages.createNewCollection)"
								:collect-project="onUserCollectProject"
								:create-collection="(event) => modalCollection?.show(event)"
							/>

							<TeleportOverflowMenu
								type="quiet"
								size="xl"
								:label="formatMessage(commonMessages.moreOptionsButton)"
								:tooltip="formatMessage(commonMessages.moreOptionsButton)"
								:options="projectHeaderMoreActions"
							>
								<MoreVerticalIcon />
							</TeleportOverflowMenu>
						</template>
					</ProjectPageHeader>
					<ProjectMemberHeader
						v-if="currentMember"
						:project="project"
						:versions="versions"
						:current-member="currentMember"
						:is-settings="isSettings"
						:route-name="route.name"
						:set-processing="setProcessing"
						:collapsed="collapsedChecklist"
						:toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
						:all-members="allMembers"
						:update-members="invalidateProject"
						:auth="auth"
						:tags="tags"
					/>
					<Admonition
						v-if="
							auth.user &&
							tags.staffRoles.includes(auth.user.role) &&
							project.actualProjectType === 'modpack' &&
							hasModpackArchiveInWarningWindow
						"
						type="warning"
						:header="formatMessage(messages.modpackArchiveWarningTitle)"
						class="mt-3"
					>
						{{ formatMessage(messages.modpackArchiveWarningDescription) }}
						<template #actions>
							<Button
								type="colored"
								color="orange"
								:loading="isCheckingModpackArchives"
								@click="checkModpackArchives"
							>
								<FileArchiveIcon />
								{{
									formatMessage(
										isCheckingModpackArchives
											? messages.checkingModpackArchives
											: messages.checkModpackArchives,
									)
								}}
							</Button>
						</template>
					</Admonition>
					<Admonition
						v-if="
							currentMember &&
							projectV3?.side_types_migration_review_status === 'pending' &&
							projectV3?.environment?.length === 1 &&
							projectV3?.environment[0] !== 'unknown'
						"
						type="warning"
						:header="
							formatMessage(
								hasEditDetailsPermission
									? messages.environmentMigrationTitle
									: messages.environmentMigrationNoPermissionTitle,
							)
						"
						class="mt-3"
					>
						{{
							formatMessage(
								hasEditDetailsPermission
									? messages.environmentMigrationMessage
									: messages.environmentMigrationNoPermissionMessage,
							)
						}}
						<nuxt-link
							to="/news/article/new-environments"
							target="_blank"
							class="mt-1 block w-fit font-semibold text-orange hover:underline"
						>
							{{ formatMessage(messages.environmentMigrationLink) }}
						</nuxt-link>
						<Button
							v-if="hasEditDetailsPermission"
							type="colored"
							color="orange"
							class="mt-3 w-fit"
							@click="() => projectEnvironmentModal.show()"
						>
							<SettingsIcon /> {{ formatMessage(messages.reviewEnvironmentSettings) }}
						</Button>
					</Admonition>
					<ArchivedProjectBanner
						v-if="isArchived"
						:title="project.title"
						:reason="archivedDisclosure?.note"
						class="mt-4"
					/>
				</div>

				<div class="normal-page__sidebar">
					<ProjectSidebarServerInfo
						v-if="isServerProject"
						:loading="!serverDataLoaded"
						:project-v3="projectV3"
						:tags="tags"
						:required-content="serverRequiredContent"
						:recommended-version="serverRecommendedVersion"
						:supported-versions="serverSupportedVersions"
						:loaders="serverModpackLoaders"
						:status-online="projectV3?.minecraft_java_server?.ping?.data != null"
						class="card flex-card"
					/>
					<ProjectSidebarCompatibility
						v-if="
							projectV3Loaded && !isServerProject && route.name !== 'type-project-version-version'
						"
						:project="project"
						:tags="tags"
						:project-v3="projectV3"
						class="card flex-card"
					/>
					<AdPlaceholder v-if="!auth.user && tags.approvedStatuses.includes(projectV3.status)" />
					<ProjectSidebarLinks
						:project="project"
						:project-v3="projectV3"
						:link-target="$external()"
						class="card flex-card"
					/>
					<ProjectSidebarTags :project="project" class="card flex-card" />
					<ProjectSidebarCreators
						:organization="organization"
						:members="members"
						:loading="creatorsLoading"
						:org-link="(slug) => `/organization/${slug}`"
						:user-link="(username) => `/user/${username}`"
						class="card flex-card"
					/>
					<ProjectSidebarDetails
						:project="project"
						:link-target="$external()"
						:hide-license="isServerProject"
						:show-followers="isServerProject"
						class="card flex-card"
					/>
				</div>

				<div class="normal-page__content">
					<NavTabs :links="navLinks" replace page-nav />
					<NuxtPage @on-download="triggerDownloadAnimation" @delete-version="deleteVersion" />
				</div>
			</div>
		</div>

		<ClientOnly>
			<ModerationChecklist
				v-if="auth.user && tags.staffRoles.includes(auth.user.role) && showModerationChecklist"
				:collapsed="collapsedModerationChecklist"
				@exit="setModerationChecklistOpen(false)"
				@toggle-collapsed="collapsedModerationChecklist = !collapsedModerationChecklist"
			/>
		</ClientOnly>

		<template v-if="hasEditDetailsPermission">
			<ProjectEnvironmentModal ref="projectEnvironmentModal" />
		</template>
	</template>
</template>

<script setup>
import {
	ChartIcon,
	ClipboardCopyIcon,
	CompassIcon,
	DownloadIcon,
	FileArchiveIcon,
	FolderSearchIcon,
	HeartIcon,
	LeftArrowIcon,
	MoreVerticalIcon,
	PackageSearchIcon,
	PlayIcon,
	ReportIcon,
	ScaleIcon,
	ScanEyeIcon,
	SearchIcon,
	ServerPlusIcon,
	SettingsIcon,
	XIcon,
} from '@modrinth/assets'
import { getMarginTarget, moderationSettings } from '@modrinth/moderation'
import {
	Admonition,
	ArchivedProjectBanner,
	Avatar,
	BrowseInstallHeader,
	Button,
	ButtonLink,
	commonMessages,
	defineMessages,
	formatDependencyProjectFilterOption,
	formatProjectTypeSentence,
	getActiveDisclosures,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	IntlFormatted,
	NavTabs,
	OpenInAppModal,
	PageHeader,
	PageHeaderActions,
	PageHeaderMetadata,
	PageHeaderMetadataItem,
	PROJECT_DEP_MARKER_QUERY,
	ProjectBackgroundGradient,
	ProjectEnvironmentModal,
	ProjectPageHeader,
	ProjectSidebarCompatibility,
	ProjectSidebarCreators,
	ProjectSidebarDetails,
	ProjectSidebarLinks,
	ProjectSidebarServerInfo,
	ProjectSidebarTags,
	provideProjectPageContext,
	SelectedProjectsFloatingBar,
	TeleportOverflowMenu,
	Tooltip,
	useDebugLogger,
	useFormatPrice,
	useRelativeTime,
	useStickyObserver,
	useVIntl,
} from '@modrinth/ui'
import { formatProjectType, isStaff } from '@modrinth/utils'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useLocalStorage } from '@vueuse/core'
import { nextTick, onScopeDispose, readonly, ref, useTemplateRef, watch, watchEffect } from 'vue'

import { navigateTo } from '#app'
import AdPlaceholder from '~/components/ui/AdPlaceholder.vue'
import CollectionCreateModal from '~/components/ui/create/CollectionCreateModal.vue'
import ModerationChecklist from '~/components/ui/moderation/checklist/ModerationChecklist.vue'
import ModerationProjectNags from '~/components/ui/moderation/ModerationProjectNags.vue'
import ModpackScanModal from '~/components/ui/moderation/ModpackScanModal.vue'
import ProjectCollectionSaveButton from '~/components/ui/ProjectCollectionSaveButton.vue'
import ProjectDownloadModal from '~/components/ui/ProjectDownloadModal/index.vue'
import ProjectMemberHeader from '~/components/ui/ProjectMemberHeader.vue'
import { getSignInRouteObj } from '~/composables/auth.ts'
import { useProjectLinkValidation } from '~/composables/link-network-validation'
import {
	canSubmitProjectForReview,
	PROJECT_REVIEW_VALIDATION_ERROR,
} from '~/composables/link-network-validation/submission'
import { saveFeatureFlags } from '~/composables/featureFlags.ts'
import { notifyCopied } from '~/composables/moderation.ts'
import { STALE_TIME, STALE_TIME_LONG, warmProjectCheckCaches } from '~/composables/queries/project'
import { versionQueryOptions } from '~/composables/queries/version'
import { useServerInstallContent } from '~/composables/use-server-install-content'
import { userCollectProject, userFollowProject } from '~/composables/user.js'
import { injectCurrentProjectId } from '~/providers/current-project.ts'
import { loadChecklistState } from '~/services/moderation/checklist-storage.ts'
import { useModerationQueue } from '~/services/moderation/queue.ts'
import { getReportPath, reportProject } from '~/utils/report-helpers.ts'

definePageMeta({
	key: (route) => `${route.params.project}`,
})

const data = useNuxtApp()
const route = useRoute()
const router = useRouter()
const signInRouteObj = computed(() => getSignInRouteObj(route))
const config = useRuntimeConfig()
const moderationQueue = useModerationQueue()
const keybinds = useModerationKeybinds()
const modSettings = useModerationSettings()
const marginTarget = computed(() => getMarginTarget(modSettings.value))
const notifications = injectNotificationManager()
const { addNotification } = notifications

const auth = await useAuth()
const user = await useUser()

// Route slug or ID — resolve to canonical ID before fetching project data
const routeParam = computed(() => {
	const param = route.params.project
	return Array.isArray(param) ? param[0] : param
})

const { createProjectDownloadUrl } = useCdnDownloadContext()

const downloadReason = ref('standalone')

function absorbDepQuery() {
	if (route.query.dep === PROJECT_DEP_MARKER_QUERY.dep) {
		downloadReason.value = 'dependency'
		if (import.meta.client) {
			const newQuery = { ...route.query }
			delete newQuery.dep
			void router.replace({ path: route.path, query: newQuery, hash: route.hash })
		}
	}
}

watch(() => route.query.dep, absorbDepQuery, { immediate: true })

const tags = useGeneratedState()
const flags = useFeatureFlags()
const cosmetics = useCosmetics()
const formatRelativeTime = useRelativeTime()

const { formatMessage } = useVIntl()
const formatPrice = useFormatPrice()

const debug = useDebugLogger('DownloadModal')

const downloadModal = ref()
const openInAppModal = ref()
const overTheTopDownloadAnimation = ref()
const scanModal = ref()
const isCheckingModpackArchives = ref(false)

const projectV3Loaded = computed(() => !projectV3Pending.value || projectV3.value != null)
const isServerProject = computed(() => projectV3.value?.minecraft_server != null)
const stickyInstallHeaderRef = ref(null)
const { isStuck: isInstallHeaderStuck } = useStickyObserver(
	stickyInstallHeaderRef,
	'ProjectInstallHeader',
)

const projectEnvironmentModal = useTemplateRef('projectEnvironmentModal')

const baseId = useId()

const serverProject = computed(() => {
	if (!project.value) {
		return undefined
	}
	return {
		name: project.value.title,
		slug: project.value.slug || project.value.id,
		numPlayers: projectV3.value?.minecraft_java_server?.ping?.data?.players_online,
		icon: project.value.icon_url,
		statusOnline: !!projectV3.value?.minecraft_java_server?.ping?.data,
		region: projectV3.value?.minecraft_server?.region,
	}
})

function handlePlayServerProject() {
	openInAppModal.value?.show({
		serverProject: serverProject.value,
	})
}

const messages = defineMessages({
	backToAllProjects: {
		id: 'project.settings.back-to-all-projects',
		defaultMessage: 'Back to all projects',
	},
	backToDiscover: {
		id: 'project.install-context.back-to-discover',
		defaultMessage: 'Back to discover',
	},
	backToProjectPage: {
		id: 'project.settings.back-to-project-page',
		defaultMessage: 'Back to project page',
	},
	changelogTab: {
		id: 'project.navigation.changelog',
		defaultMessage: 'Changelog',
	},
	createServer: {
		id: 'project.actions.create-server',
		defaultMessage: 'Create a server',
	},
	createServerTooltip: {
		id: 'project.actions.create-server-tooltip',
		defaultMessage: 'Create a server',
	},
	createNewCollection: {
		id: 'project.collections.create-new',
		defaultMessage: 'Create new collection',
	},
	descriptionTab: {
		id: 'project.description.title',
		defaultMessage: 'Description',
	},
	dontShowAgain: {
		id: 'project.actions.dont-show-again',
		defaultMessage: "Don't show again",
	},
	editProject: {
		id: 'project.actions.edit-project',
		defaultMessage: 'Edit project',
	},
	errorLoadingProject: {
		id: 'project.error.loading',
		defaultMessage: 'Error loading project data{message}',
	},
	environmentMigrationMessage: {
		id: 'project.environment.migration.message',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. Please verify that the metadata is correct.",
	},
	environmentMigrationTitle: {
		id: 'project.environment.migration.title',
		defaultMessage: 'Please review environment metadata',
	},
	environmentMigrationNoPermissionMessage: {
		id: 'project.environment.migration-no-permission.message',
		defaultMessage:
			"We've just overhauled the Environments system on Modrinth and new options are now available. You don't have permission to modify these settings, but please let another member of the project know that the environment metadata needs to be verified.",
	},
	environmentMigrationNoPermissionTitle: {
		id: 'project.environment.migration-no-permission.title',
		defaultMessage: 'Environment metadata needs to be reviewed',
	},
	environmentMigrationLink: {
		id: 'project.environment.migration.learn-more',
		defaultMessage: 'Learn more about this change',
	},
	galleryTab: {
		id: 'project.gallery.title',
		defaultMessage: 'Gallery',
	},
	moderationTab: {
		id: 'project.moderation.title',
		defaultMessage: 'Moderation',
	},
	noCollectionsFound: {
		id: 'project.collections.none-found',
		defaultMessage: 'No collections found.',
	},
	pageNotFound: {
		id: 'project.error.page-not-found',
		defaultMessage: 'The page could not be found',
	},
	projectIconUpdated: {
		id: 'project.notification.icon-updated.title',
		defaultMessage: 'Project icon updated',
	},
	projectIconUpdatedMessage: {
		id: 'project.notification.icon-updated.message',
		defaultMessage: "Your project's icon has been updated.",
	},
	projectNotFound: {
		id: 'project.error.project-not-found',
		defaultMessage: 'Project not found',
	},
	projectUpdated: {
		id: 'project.notification.updated.title',
		defaultMessage: 'Project updated',
	},
	projectUpdatedMessage: {
		id: 'project.notification.updated.message',
		defaultMessage: 'Your project has been updated.',
	},
	projectReviewSaveFailed: {
		id: 'project.notification.review-save-failed.title',
		defaultMessage: 'Failed to save project in review',
	},
	projectReviewSaveFailedDescription: {
		id: 'project.notification.review-save-failed.description',
		defaultMessage: 'You cannot save edits to your project which result in failing validation.',
	},
	reviewEnvironmentSettings: {
		id: 'project.environment.migration.review-button',
		defaultMessage: 'Review environment settings',
	},
	projectPage: {
		id: 'project.actions.project-page',
		defaultMessage: 'Project page',
	},
	reviewProject: {
		id: 'project.actions.review-project',
		defaultMessage: 'Review project',
	},
	viewDependents: {
		id: 'project.actions.view-dependents',
		defaultMessage: 'View dependents',
	},
	viewProjectTypeDependents: {
		id: 'project.actions.view-project-type-dependents',
		defaultMessage: 'View {projectType} dependents',
	},
	viewModpacks: {
		id: 'project.actions.view-modpacks',
		defaultMessage: 'View modpacks',
	},
	rescanModpack: {
		id: 'project.actions.rescan-modpack',
		defaultMessage: 'Rescan modpack',
	},
	checkModpackArchives: {
		id: 'project.actions.check-modpack-archives',
		defaultMessage: 'Check modpack unzip',
	},
	checkingModpackArchives: {
		id: 'project.actions.checking-modpack-archives',
		defaultMessage: 'Checking...',
	},
	checkModpackArchivesSuccess: {
		id: 'project.notification.check-modpack-archives.success',
		defaultMessage:
			'{count, plural, one {The modpack file can be unzipped.} other {All # modpack files can be unzipped.}}',
	},
	checkModpackArchivesFailed: {
		id: 'project.notification.check-modpack-archives.failed',
		defaultMessage: 'Some modpack files could not be unzipped',
	},
	checkModpackArchivesNoFiles: {
		id: 'project.notification.check-modpack-archives.no-files',
		defaultMessage: 'No .mrpack files were found for this project.',
	},
	modpackArchiveWarningTitle: {
		id: 'project.modpack-archive-warning.title',
		defaultMessage: 'This modpack was published during export bug',
	},
	modpackArchiveWarningDescription: {
		id: 'project.modpack-archive-warning.description',
		defaultMessage: 'Importing this .mrpack might be broken.',
	},
	serversPromoDescription: {
		id: 'project.actions.servers-promo.description',
		defaultMessage: 'Modrinth Hosting is the easiest way to play with your friends without hassle!',
	},
	serversPromoPricing: {
		id: 'project.actions.servers-promo.pricing',
		defaultMessage: 'Starting at {price}<small> / month</small>',
	},
	serversPromoTitle: {
		id: 'project.actions.servers-promo.title',
		defaultMessage: 'Create a server',
	},
	versionsTab: {
		id: 'project.versions.title',
		defaultMessage: 'Versions',
	},
	editingProject: {
		id: 'project.settings.editing-project',
		defaultMessage: 'Editing {projectType} project',
	},
})

const modalCollection = useTemplateRef('modal_collection')

const collections = computed(() =>
	user.value && user.value.collections ? user.value.collections : [],
)

if (
	!routeParam.value ||
	!(
		tags.value.projectTypes.find((x) => x.id === route.params.type) ||
		route.params.type === 'project'
	)
) {
	throw createError({
		fatal: false,
		statusCode: 404,
		message: formatMessage(messages.pageNotFound),
	})
}

// Use DI client for TanStack Query
const client = injectModrinthClient()
const queryClient = useQueryClient()

// Resolve route slug/ID to the canonical project ID (middleware warms this cache)
const { data: projectCheck, error: projectCheckError } = useQuery({
	queryKey: computed(() => ['project', 'check', routeParam.value]),
	queryFn: () => client.labrinth.projects_v2.check(routeParam.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!routeParam.value),
})

const projectId = computed(() => projectCheck.value?.id)

function showProjectLoadError(error) {
	const status = error.statusCode ?? error.status ?? 500
	showError({
		fatal: true,
		statusCode: status,
		message:
			status === 404
				? formatMessage(messages.projectNotFound)
				: formatMessage(messages.errorLoadingProject, {
						message: error.message ? `: ${error.message}` : '',
					}),
	})
}

watch(
	projectCheckError,
	(error) => {
		if (error) {
			showProjectLoadError(error)
		}
	},
	{ immediate: true },
)

// V2 Project — keyed by canonical ID
const { data: projectRaw, error: projectV2Error } = useQuery({
	queryKey: computed(() => ['project', 'v2', projectId.value]),
	queryFn: () => client.labrinth.projects_v2.get(projectId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!projectId.value),
})

watch(
	projectV2Error,
	(error) => {
		if (error) {
			showProjectLoadError(error)
		}
	},
	{ immediate: true },
)

// Transform project via computed
const project = computed(() => {
	if (!projectRaw.value) return null
	return {
		...projectRaw.value,
		actualProjectType: projectRaw.value.project_type,
		project_type: data.$getProjectTypeForUrl(
			projectRaw.value.project_type,
			projectRaw.value.loaders,
			tags.value,
		),
	}
})

const routeProjectType = computed(() =>
	Array.isArray(route.params.type) ? route.params.type[0] : route.params.type,
)
const projectInstallType = computed(() => ({
	id: project.value?.actualProjectType ?? routeProjectType.value,
}))
const serverInstallModalRef = ref(null)
const serverInstallDebug = useDebugLogger('ProjectServerInstall')
const { installContext: serverBrowseInstallContext } = useServerInstallContent({
	projectType: projectInstallType,
	onboardingModalRef: serverInstallModalRef,
	debug: serverInstallDebug,
})
const projectDiscoverBackUrl = computed(() => {
	const discoverType =
		routeProjectType.value === 'project'
			? (project.value?.actualProjectType ?? project.value?.project_type ?? 'mod')
			: (routeProjectType.value ?? project.value?.actualProjectType ?? 'mod')

	return `/discover/${discoverType}s${getInstallContextQueryString(['sid', 'wid', 'from', 'shi'])}`
})
const projectInstallContext = computed(() => {
	const context = serverBrowseInstallContext.value
	if (!context) return null
	return {
		...context,
		backUrl: projectDiscoverBackUrl.value,
		backLabel: formatMessage(messages.backToDiscover),
		discardSelectedAndBack: async () => {
			await (context.clearSelected ?? context.clearQueued)?.()
			await navigateTo(projectDiscoverBackUrl.value)
		},
	}
})
const projectHeaderInstallContext = computed(() => {
	const context = projectInstallContext.value
	if (!context) return null
	return {
		...context,
		onBack: undefined,
		selectedProjects: [],
		isInstallingSelected: false,
	}
})

const sharedProjectId = injectCurrentProjectId(null)
if (sharedProjectId) {
	watchEffect(() => {
		sharedProjectId.value = projectId.value ?? undefined
	})
	onScopeDispose(() => {
		sharedProjectId.value = undefined
	})
}

// V3 Project
const {
	data: projectV3,
	error: _projectV3Error,
	isPending: projectV3Pending,
} = useQuery({
	queryKey: computed(() => ['project', 'v3', projectId.value]),
	queryFn: () => client.labrinth.projects_v3.get(projectId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!projectId.value),
})

// Server sidebar: modpack version + project for required content
const serverModpackVersionId = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	return content?.kind === 'modpack' ? content.version_id : null
})

const { data: serverModpackVersion, isPending: serverModpackVersionPending } = useQuery({
	queryKey: computed(() => ['version', 'v3', serverModpackVersionId.value]),
	queryFn: () => client.labrinth.versions_v3.getVersion(serverModpackVersionId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!serverModpackVersionId.value),
})

const serverDataLoaded = computed(() => {
	if (!projectV3.value) return false
	if (serverModpackVersionId.value && serverModpackVersionPending.value) return false
	return true
})

const serverRequiredContent = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	if (!content || content.kind !== 'modpack') return null
	const primaryFile =
		serverModpackVersion.value?.files?.find((f) => f.primary) ??
		serverModpackVersion.value?.files?.[0]
	return {
		name: content.project_name ?? '',
		versionNumber: serverModpackVersion.value?.version_number ?? '',
		icon: content.project_icon,
		onclickName:
			content.project_id && content.project_id !== projectId.value
				? () => {
						navigateTo({
							path: `/project/${content.project_id}`,
							query: { ...PROJECT_DEP_MARKER_QUERY },
						})
					}
				: undefined,
		onclickVersion:
			content.project_id && content.project_id !== projectId.value
				? () => {
						navigateTo({
							path: `/project/${content.project_id}/version/${serverModpackVersion.value?.id}`,
							query: { ...PROJECT_DEP_MARKER_QUERY },
						})
					}
				: undefined,
		onclickDownload: primaryFile?.url
			? () =>
					navigateTo(createProjectDownloadUrl(primaryFile.url, { reason: 'dependency' }), {
						external: true,
					})
			: undefined,
		showCustomModpackTooltip: content.project_id === projectId.value,
	}
})

const serverRecommendedVersion = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	if (!content) return null

	if (content.kind === 'modpack') {
		return serverModpackVersion.value?.game_versions?.[0] ?? null
	}

	if (content.kind === 'vanilla') {
		return content.recommended_game_version ?? null
	}

	return null
})

const serverSupportedVersions = computed(() => {
	const content = projectV3.value?.minecraft_java_server?.content
	if (!content) return []

	if (content.kind === 'vanilla') {
		return content.supported_game_versions?.filter((v) => !!v) ?? []
	}

	return []
})

const serverModpackLoaders = computed(() => {
	if (!serverModpackVersion.value) return []
	return serverModpackVersion.value.mrpack_loaders ?? []
})

watch(serverModpackVersionId, (versionId) => {
	if (!versionId) return
	queryClient.prefetchQuery(versionQueryOptions.v3(versionId, client))
})

// Members
const {
	data: allMembersRaw,
	error: _membersError,
	isPending: membersPending,
} = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'members']),
	queryFn: () => client.labrinth.projects_v3.getMembers(projectId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!projectId.value),
})

// Transform members via computed
const allMembers = computed(() => {
	if (!allMembersRaw.value) return []
	return allMembersRaw.value.map((it) => ({
		...it,
		avatar_url: it.user.avatar_url,
		name: it.user.username,
	}))
})

// Dependencies - lazy loaded client-side only
const dependenciesEnabled = ref(false)
const {
	data: dependenciesRaw,
	error: _dependenciesError,
	isFetching: dependenciesLoading,
} = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'dependencies']),
	queryFn: () => client.labrinth.projects_v2.getDependencies(projectId.value),
	staleTime: STALE_TIME_LONG,
	enabled: computed(() => !!projectId.value && dependenciesEnabled.value),
})

const dependencies = computed(() => dependenciesRaw.value ?? null)

// V3 Versions - lazy loaded client-side only (except for staff, who need v3 versions for moderation)
const versionsEnabled = ref(false)
const {
	data: versionsV3,
	error: _versionsV3Error,
	isFetching: versionsV3Loading,
} = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'versions', 'v3']),
	queryFn: () =>
		client.labrinth.versions_v3.getProjectVersions(projectId.value, {
			include_changelog: false,
			apiVersion: 3,
		}),
	staleTime: STALE_TIME_LONG,
	enabled: computed(() => !!projectId.value && (versionsEnabled.value || isStaff(auth.value.user))),
})

// Organization
// Only fetch organization if project belongs to one
const { data: organizationRaw, isPending: organizationPending } = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'organization']),
	queryFn: () => client.labrinth.projects_v3.getOrganization(projectId.value),
	staleTime: STALE_TIME,
	enabled: computed(() => !!projectId.value && !!projectRaw.value?.organization),
})

// When project is removed from org, enabled becomes false but TanStack keeps stale data.
// Return null when the project no longer belongs to an organization.
const organization = computed(() => (projectRaw.value?.organization ? organizationRaw.value : null))

const DISCLOSURE_STALE_TIME = 1000 * 60 * 5
const { data: disclosuresResponse } = useQuery({
	queryKey: computed(() => ['project', 'disclosures', 'v3', projectId.value]),
	queryFn: () => client.labrinth.projects_v3.getDisclosures(projectId.value),
	staleTime: DISCLOSURE_STALE_TIME,
	enabled: computed(() => !!projectId.value),
})

const archivedDisclosure = computed(() =>
	getActiveDisclosures(disclosuresResponse.value?.disclosures).find(
		(disclosure) => disclosure.type === 'archived',
	),
)
const isArchived = computed(() => !!archivedDisclosure.value)

const creatorsLoading = computed(
	() =>
		!projectRaw.value ||
		membersPending.value ||
		(!!projectRaw.value.organization && organizationPending.value),
)

const { data: thread } = useQuery({
	queryKey: computed(() => ['thread', projectRaw.value?.thread_id]),
	queryFn: () => client.labrinth.threads_v3.getThread(projectRaw.value.thread_id),
	enabled: computed(() => !!projectRaw.value?.thread_id),
})

const isSettings = computed(() => route.name.startsWith('type-project-settings'))
useFavicon(() => (isSettings.value ? 'settings' : 'default'))

// Jank modpack loaders fix
const versionsRaw = computed(() => {
	return (versionsV3.value ?? []).map((version) => {
		const files = Array.isArray(version.files) ? version.files : []
		const gameVersions = Array.isArray(version.game_versions) ? version.game_versions : []
		const loaders = Array.isArray(version.loaders) ? version.loaders : []
		const isModpack = version.project_types?.includes('modpack')
		const mrpackLoaders = Array.isArray(version.mrpack_loaders) ? version.mrpack_loaders : []

		return {
			...version,
			files,
			game_versions: gameVersions,
			loaders: isModpack && mrpackLoaders.length ? mrpackLoaders : loaders,
		}
	})
})

// Apply version computations (slug generation, author lookup, etc.)
const versions = computed(() => {
	if (!versionsRaw.value.length || !allMembers.value.length) return versionsRaw.value
	return data.$computeVersions(versionsRaw.value, allMembers.value)
})

// Versions loading state
const versionsLoading = computed(() => versionsV3Loading.value)
const versionsLoaded = computed(() => versionsV3.value !== undefined || !!_versionsV3Error.value)

// Load versions on demand (client-side only)
function loadVersions() {
	debug('loadVersions called', {
		projectId: projectId.value,
		alreadyEnabled: versionsEnabled.value,
	})
	versionsEnabled.value = true
}

// Load dependencies on demand (client-side only)
function loadDependencies() {
	dependenciesEnabled.value = true
}

// Check if project has versions using the ID array from the V2 project
// This allows showing/hiding UI elements without loading full version data
const hasVersions = computed(() => (project.value?.versions?.length ?? 0) > 0)

async function invalidateProject() {
	const id = projectId.value
	if (!id) {
		return
	}
	await queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] })
	await queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] })
	// Prefix match — invalidates members, versions, dependencies, organization
	await queryClient.invalidateQueries({ queryKey: ['project', id] })
}

async function redirectIfNewSlug(newSlug, id) {
	if (newSlug === undefined || newSlug === route.params.project) {
		return
	}

	warmProjectCheckCaches(queryClient, { id, slug: newSlug })

	await navigateTo(
		{
			name: route.name,
			params: {
				type: route.params.type,
				project: newSlug,
			},
			query: route.query,
			hash: route.hash,
		},
		{ replace: true },
	)
}

function mergeV3ProjectPatch(old, data) {
	if (!old) {
		return old
	}
	const merged = { ...old }
	for (const [key, value] of Object.entries(data)) {
		if (
			value &&
			typeof value === 'object' &&
			!Array.isArray(value) &&
			merged[key] &&
			typeof merged[key] === 'object' &&
			!Array.isArray(merged[key])
		) {
			merged[key] = { ...merged[key], ...value }
		} else {
			merged[key] = value
		}
	}
	return merged
}

function addProjectMutationErrorNotification(error) {
	const description =
		error?.v1Error?.description ??
		error?.responseData?.description ??
		error?.data?.description ??
		error?.message
	const response = error?.responseData ?? error?.data ?? error?.v1Error
	const isProjectValidationError =
		Array.isArray(response?.details?.nags) || description === PROJECT_REVIEW_VALIDATION_ERROR

	addNotification({
		title: formatMessage(
			isProjectValidationError && project.value.status === 'processing'
				? messages.projectReviewSaveFailed
				: commonMessages.errorNotificationTitle,
		),
		text: isProjectValidationError
			? formatMessage(messages.projectReviewSaveFailedDescription)
			: description,
		type: 'error',
	})
}

// Mutation for patching project data
const patchProjectMutation = useMutation({
	mutationFn: async ({ projectId, data }) => {
		await client.labrinth.projects_v2.edit(projectId, data)
		await redirectIfNewSlug(data.slug, projectId)
		return data
	},

	onMutate: async ({ projectId, data, optimistic = true }) => {
		await linkValidation.validateSave({
			description: data.body,
			license_url: data.license_url,
			link_urls: Object.fromEntries(
				['issues', 'source', 'wiki', 'discord']
					.filter((field) => data[`${field}_url`] !== undefined)
					.map((field) => [field, data[`${field}_url`]]),
			),
		})
		if (!optimistic) return
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })
		await queryClient.cancelQueries({ queryKey: ['project', 'v3', projectId] })

		const previousV2 = queryClient.getQueryData(['project', 'v2', projectId])
		const previousV3 = queryClient.getQueryData(['project', 'v3', projectId])

		queryClient.setQueryData(['project', 'v2', projectId], (old) => {
			if (!old) return old
			return { ...old, ...data }
		})
		if (data.slug !== undefined) {
			queryClient.setQueryData(['project', 'v3', projectId], (old) =>
				old ? { ...old, slug: data.slug } : old,
			)
		}

		return { previousV2, previousV3, projectId }
	},

	onError: (err, _variables, context) => {
		if (context?.previousV2) {
			queryClient.setQueryData(['project', 'v2', context.projectId], context.previousV2)
		}
		if (context?.previousV3) {
			queryClient.setQueryData(['project', 'v3', context.projectId], context.previousV3)
		}
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Mutation for changing project status (setProcessing)
const patchStatusMutation = useMutation({
	mutationFn: async (variables) => {
		await client.labrinth.projects_v2.edit(variables.projectId, { status: variables.status })
	},

	onMutate: async ({ projectId, status }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })

		const previousProject = queryClient.getQueryData(['project', 'v2', projectId])

		queryClient.setQueryData(['project', 'v2', projectId], (old) => {
			if (!old) return old
			return { ...old, status }
		})

		return { previousProject, projectId }
	},

	onSuccess: async (_data, { threadId }) => {
		if (threadId) {
			await queryClient.invalidateQueries({ queryKey: ['thread', threadId] })
		}
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', context.projectId], context.previousProject)
		}
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Mutation for patching V3 project data
const patchProjectV3Mutation = useMutation({
	mutationFn: async ({ projectId, data }) => {
		await client.labrinth.projects_v3.edit(projectId, data)
		await redirectIfNewSlug(data.slug, projectId)
		return data
	},

	onMutate: async ({ projectId, data, optimistic = true }) => {
		await linkValidation.validateSave(data)
		if (!optimistic) return
		await queryClient.cancelQueries({ queryKey: ['project', 'v3', projectId] })
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })

		const previousV3 = queryClient.getQueryData(['project', 'v3', projectId])
		const previousV2 = queryClient.getQueryData(['project', 'v2', projectId])

		queryClient.setQueryData(['project', 'v3', projectId], (old) => mergeV3ProjectPatch(old, data))
		if (data.slug !== undefined) {
			queryClient.setQueryData(['project', 'v2', projectId], (old) =>
				old ? { ...old, slug: data.slug } : old,
			)
		}

		return { previousV3, previousV2, projectId }
	},

	onError: (err, _variables, context) => {
		if (context?.previousV3) {
			queryClient.setQueryData(['project', 'v3', context.projectId], context.previousV3)
		}
		if (context?.previousV2) {
			queryClient.setQueryData(['project', 'v2', context.projectId], context.previousV2)
		}
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Mutation for patching project icon
const patchIconMutation = useMutation({
	mutationFn: async ({ projectId, icon }) => {
		const ext = icon.type.split('/')[icon.type.split('/').length - 1]
		await client.labrinth.projects_v3.changeIcon(projectId, icon, ext)
	},

	onSuccess: () => {
		addNotification({
			title: formatMessage(messages.projectIconUpdated),
			text: formatMessage(messages.projectIconUpdatedMessage),
			type: 'success',
		})
	},

	onError: (err) => {
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

const createGalleryItemMutation = useMutation({
	mutationFn: async ({ projectId, file, title, description, featured, ordering }) => {
		const ext = file.type.split('/')[file.type.split('/').length - 1]
		await client.labrinth.projects_v2.createGalleryImage(projectId, file, {
			ext,
			featured: featured ?? false,
			title,
			description,
			ordering,
		})
	},

	onMutate: async ({ projectId, title, description, featured, ordering }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })

		const previousProject = queryClient.getQueryData(['project', 'v2', projectId])

		queryClient.setQueryData(['project', 'v2', projectId], (old) => {
			if (!old) return old
			const newItem = {
				url: '',
				raw_url: '',
				featured: featured ?? false,
				title: title ?? '',
				description: description ?? '',
				created: new Date().toISOString(),
				ordering: ordering ?? old.gallery.length,
			}
			return {
				...old,
				gallery: [...old.gallery, newItem],
			}
		})

		return { previousProject, projectId }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', context.projectId], context.previousProject)
		}
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

const editGalleryItemMutation = useMutation({
	mutationFn: async ({ projectId, imageUrl, title, description, featured, ordering }) => {
		await client.labrinth.projects_v2.editGalleryImage(projectId, imageUrl, {
			featured: featured ?? false,
			title,
			description,
			ordering,
		})
	},

	onMutate: async ({ projectId, imageUrl, title, description, featured, ordering }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })

		const previousProject = queryClient.getQueryData(['project', 'v2', projectId])

		queryClient.setQueryData(['project', 'v2', projectId], (old) => {
			if (!old) return old
			return {
				...old,
				gallery: old.gallery.map((item) => {
					if (item.url === imageUrl) {
						return {
							...item,
							title: title ?? item.title,
							description: description ?? item.description,
							featured: featured ?? item.featured,
							ordering: ordering ?? item.ordering,
						}
					}
					return item
				}),
			}
		})

		return { previousProject, projectId }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', context.projectId], context.previousProject)
		}
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

const deleteGalleryItemMutation = useMutation({
	mutationFn: async ({ projectId, imageUrl }) => {
		await client.labrinth.projects_v2.deleteGalleryImage(projectId, imageUrl)
	},

	onMutate: async ({ projectId, imageUrl }) => {
		await queryClient.cancelQueries({ queryKey: ['project', 'v2', projectId] })

		const previousProject = queryClient.getQueryData(['project', 'v2', projectId])

		queryClient.setQueryData(['project', 'v2', projectId], (old) => {
			if (!old) return old
			return {
				...old,
				gallery: old.gallery.filter((item) => item.url !== imageUrl),
			}
		})

		return { previousProject, projectId }
	},

	onError: (err, _variables, context) => {
		if (context?.previousProject) {
			queryClient.setQueryData(['project', 'v2', context.projectId], context.previousProject)
		}
		addProjectMutationErrorNotification(err)
	},

	onSettled: async () => {
		await invalidateProject()
	},
})

// Members should be an array of all members, without the accepted ones, and with the user with the Owner role at the start
// The rest of the members should be sorted by role, then by name
const members = computed(() => {
	const acceptedMembers = allMembers.value.filter((x) => x.accepted)
	const owner = acceptedMembers.find((x) =>
		organization.value
			? organization.value.members?.some(
					(orgMember) => orgMember.user.id === x.user.id && orgMember.is_owner,
				)
			: x.is_owner,
	)

	const rest = acceptedMembers.filter((x) => !owner || x.user.id !== owner.user.id) || []

	rest.sort((a, b) => {
		if (a.role === b.role) {
			return a.user.username.localeCompare(b.user.username)
		} else {
			return a.role.localeCompare(b.role)
		}
	})

	return owner ? [owner, ...rest] : rest
})

const isMember = computed(
	() => auth.value.user && allMembers.value.some((x) => x.user.id === auth.value.user.id),
)

const currentMember = computed(() => {
	let val = auth.value.user ? allMembers.value.find((x) => x.user.id === auth.value.user.id) : null

	if (!val && auth.value.user && organization.value && organization.value.members) {
		val = organization.value.members.find((x) => x.user.id === auth.value.user.id)
	}

	if (
		!val &&
		auth.value.user &&
		project.value &&
		tags.value.staffRoles.includes(auth.value.user.role)
	) {
		val = {
			team_id: project.value.team_id,
			user: auth.value.user,
			role: auth.value.role,
			permissions: auth.value.user.role === 'admin' ? 1023 : 12,
			accepted: true,
			payouts_split: 0,
			avatar_url: auth.value.user.avatar_url,
			name: auth.value.user.username,
			staffOnly: true,
		}
	}

	return val
})

const {
	data: projectValidationResponse,
	isFetching: projectValidationLoading,
	isError: backendValidationError,
	refetch: refetchProjectValidation,
} = useQuery({
	queryKey: computed(() => ['project', projectId.value, 'validation', 'v3']),
	queryFn: () => client.labrinth.projects_v3.validate(projectId.value),
	staleTime: 0,
	enabled: computed(() => !!projectId.value && !!currentMember.value?.accepted),
})

const linkValidation = useProjectLinkValidation(
	projectId,
	projectV3,
	() => !!currentMember.value?.accepted,
)
const projectLinksNetworkValidationLoading = linkValidation.isChecking
const reviewSubmissionPending = ref(false)
const reviewSubmissionLoading = computed(
	() =>
		projectValidationLoading.value ||
		projectLinksNetworkValidationLoading.value ||
		reviewSubmissionPending.value,
)
const projectValidation = computed(() => {
	const validation = projectValidationResponse.value
	if (!validation || backendValidationError.value || linkValidation.isError.value) return null
	return { ...validation, nags: [...validation.nags, ...linkValidation.nags.value] }
})

async function refreshProjectValidation() {
	const projectIdAtStart = projectId.value
	const [result, network] = await Promise.all([
		refetchProjectValidation({ cancelRefetch: false }),
		linkValidation.refresh(),
	])
	await nextTick()
	if (
		!result.isSuccess ||
		!network.isSuccess ||
		projectId.value !== projectIdAtStart ||
		projectValidationLoading.value ||
		projectLinksNetworkValidationLoading.value
	) {
		return null
	}
	return projectValidation.value
}

const canAccessSettings = computed(() => !!currentMember.value?.accepted)

const hasEditDetailsPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (currentMember.value?.permissions & EDIT_DETAILS) === EDIT_DETAILS
})

watch(
	[isSettings, allMembers, canAccessSettings],
	() => {
		if (isSettings.value && allMembers.value.length > 0 && !canAccessSettings.value) {
			showError({
				fatal: true,
				statusCode: 401,
				statusMessage: 'Unauthorized',
			})
		}
	},
	{ flush: 'sync', immediate: true },
)

const projectTypeDisplay = computed(() => {
	if (!project.value) return ''
	const projectType = isServerProject.value ? 'minecraft_java_server' : project.value.project_type
	return formatProjectType(data.$getProjectTypeForDisplay(projectType, project.value.loaders))
})

const following = computed(() => {
	if (!user.value?.follows || !project.value) {
		return false
	}
	return !!user.value.follows.find((x) => x.id === project.value.id)
})

const PROJECT_NOT_FOUND_DESCRIPTION =
	"There's no project here, check that you have the right link! It may still be under review or no longer publicly available on Modrinth."

const title = computed(() =>
	project.value
		? `${project.value.title} - Minecraft ${projectTypeDisplay.value}`
		: 'Project not found',
)
const description = computed(() => {
	if (!project.value) {
		return PROJECT_NOT_FOUND_DESCRIPTION
	}

	const creator = organization.value?.name || members.value.find((x) => x.is_owner)?.user?.username
	const byLine = creator ? ` by ${creator}` : ''

	return `${project.value.description} - Download the Minecraft ${projectTypeDisplay.value} ${project.value.title}${byLine} on Modrinth`
})

const canCreateServerFrom = computed(() => {
	if (!project.value) return false
	return project.value.project_type === 'modpack' && project.value.server_side !== 'unsupported'
})

const projectSearchUrl = computed(
	() => `/discover/${isServerProject.value ? 'servers' : `${project.value?.project_type}s`}`,
)
const projectPath = computed(() =>
	project.value
		? `/${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`
		: '',
)

const settingsEntryRouteName = ref()

function setSettingsEntryRoute() {
	const backPath = window.history.state?.back
	if (!isSettings.value || typeof backPath !== 'string') {
		settingsEntryRouteName.value = undefined
		return
	}
	settingsEntryRouteName.value = router.resolve(backPath).name?.toString()
}

onMounted(setSettingsEntryRoute)
watch(isSettings, setSettingsEntryRoute)

const settingsBackDestination = computed(() => {
	switch (settingsEntryRouteName.value) {
		case 'dashboard-projects':
			return {
				label: formatMessage(messages.backToAllProjects),
				to: '/dashboard/projects',
			}
		case 'type-project':
			return {
				label: formatMessage(messages.backToProjectPage),
				to: projectPath.value,
			}
		default:
			return undefined
	}
})

const projectHeaderPrimaryColor = computed(() =>
	currentMember.value || route.name === 'type-project-version-version' ? 'standard' : 'brand',
)
const showProjectHeaderCreateServerAction = computed(
	() => canCreateServerFrom.value && flags.value.showProjectPageQuickServerButton,
)
const projectHeaderCreateServerTo = computed(() =>
	project.value ? `/hosting?project=${project.value.id}#plan` : '/hosting',
)

const MRPACK_ARCHIVE_WARNING_START = new Date('2026-08-10T17:00:00.000Z').getTime()
const MRPACK_ARCHIVE_WARNING_END = new Date('2026-08-13T20:00:00.000Z').getTime()
const hasModpackArchiveInWarningWindow = computed(() =>
	(versionsV3.value ?? []).some((version) => {
		const publishedAt = new Date(version.date_published).getTime()
		return (
			version.files.some((file) => file.filename.toLowerCase().endsWith('.mrpack')) &&
			publishedAt >= MRPACK_ARCHIVE_WARNING_START &&
			publishedAt <= MRPACK_ARCHIVE_WARNING_END
		)
	}),
)

async function checkModpackArchives() {
	if (!project.value || isCheckingModpackArchives.value) return

	isCheckingModpackArchives.value = true
	startLoading()

	try {
		const versions = await client.labrinth.versions_v2.getProjectVersions(project.value.id)
		const filesByUrl = new Map(
			versions
				.flatMap((version) => version.files)
				.filter((file) => file.filename.toLowerCase().endsWith('.mrpack'))
				.map((file) => [file.url, file]),
		)
		const files = [...filesByUrl.values()]

		if (files.length === 0) {
			addNotification({
				title: formatMessage(commonMessages.errorNotificationTitle),
				text: formatMessage(messages.checkModpackArchivesNoFiles),
				type: 'error',
			})
			return
		}

		const { default: JSZip } = await import('jszip')
		const failures = []

		for (const file of files) {
			try {
				const response = await fetch(file.url)
				if (!response.ok) {
					throw new Error(`Download failed (${response.status} ${response.statusText})`)
				}

				await JSZip.loadAsync(await response.blob(), { checkCRC32: true })
			} catch (error) {
				failures.push({
					filename: file.filename,
					error: error?.message ?? String(error),
				})
			}
		}

		if (failures.length > 0) {
			addNotification({
				title: formatMessage(messages.checkModpackArchivesFailed),
				text: failures.map((failure) => `${failure.filename}: ${failure.error}`).join('\n'),
				type: 'error',
			})
			return
		}

		addNotification({
			title: formatMessage(commonMessages.successLabel),
			text: formatMessage(messages.checkModpackArchivesSuccess, { count: files.length }),
			type: 'success',
		})
	} catch (error) {
		addNotification({
			title: formatMessage(commonMessages.errorNotificationTitle),
			text: error?.data?.description ?? error?.message ?? String(error),
			type: 'error',
		})
	} finally {
		isCheckingModpackArchives.value = false
		stopLoading()
	}
}

const projectHeaderMoreActions = computed(() => {
	const isStaff = !!(auth.value.user && tags.value.staffRoles.includes(auth.value.user.role))
	const projectId = project.value?.id
	const dependentSearchTypes = getDependentSearchTypes()
	const dependentSearchActions = dependentSearchTypes
		.filter((projectType) => projectType !== 'modpack')
		.map((projectType) => ({
			id: `view-${projectType}-dependents`,
			label: formatMessage(
				dependentSearchTypes.length === 1
					? messages.viewDependents
					: messages.viewProjectTypeDependents,
				{
					projectType: formatProjectTypeSentence(formatMessage, projectType),
				},
			),
			icon: SearchIcon,
			type: 'link',
			to: {
				path: `/discover/${projectType}s`,
				query: {
					dep: formatDependencyProjectFilterOption(projectId, ['required']),
				},
			},
		}))
	const isPluginOnly = dependentSearchTypes.length === 1 && dependentSearchTypes[0] === 'plugin'

	return [
		{
			id: 'analytics',
			label: formatMessage(commonMessages.analyticsButton),
			icon: ChartIcon,
			type: 'link',
			to: `${projectPath.value}/settings/analytics`,
			shown: !!auth.value.user && !!currentMember.value,
		},
		...dependentSearchActions,
		{
			id: 'view-modpacks',
			label: formatMessage(messages.viewModpacks),
			icon: PackageSearchIcon,
			type: 'link',
			to: {
				path: '/discover/modpacks',
				query: {
					dep: formatDependencyProjectFilterOption(projectId, ['required']),
				},
			},
			shown: !isPluginOnly && project.value?.actualProjectType !== 'modpack',
		},
		{ type: 'divider' },
		{
			id: 'moderation-checklist',
			label: formatMessage(messages.reviewProject),
			icon: ScaleIcon,
			action: openModerationChecklistFromMenu,
			tone: 'orange',
			shown: !!auth.value.user && isStaff && !showModerationChecklist.value,
		},
		{
			id: 'tech-review',
			label: 'Tech review',
			icon: ScanEyeIcon,
			type: 'link',
			to: `/moderation/technical-review/${project.value?.id}`,
			tone: 'orange',
			shown: !!auth.value.user && isStaff,
		},
		{
			id: 'moderation-modpack-rescan',
			label: formatMessage(messages.rescanModpack),
			icon: FolderSearchIcon,
			action: () => scanModal.value?.show(),
			tone: 'orange',
			shown: !!auth.value.user && isStaff && project.value?.actualProjectType === 'modpack',
		},
		{
			id: 'moderation-modpack-check-archives',
			label: formatMessage(
				isCheckingModpackArchives.value
					? messages.checkingModpackArchives
					: messages.checkModpackArchives,
			),
			icon: FileArchiveIcon,
			action: checkModpackArchives,
			tone: 'orange',
			disabled: isCheckingModpackArchives.value,
			shown: !!auth.value.user && isStaff && project.value?.actualProjectType === 'modpack',
		},
		{ type: 'divider', shown: !!auth.value.user && isStaff },
		{
			id: 'report',
			label: formatMessage(commonMessages.reportButton),
			icon: ReportIcon,
			action: reportProjectFromHeader,
			tone: 'red',
			shown: !isMember.value,
		},
		{
			id: 'copy-id',
			label: formatMessage(commonMessages.copyIdButton),
			icon: ClipboardCopyIcon,
			action: copyId,
		},
		{
			id: 'copy-permalink',
			label: formatMessage(commonMessages.copyPermalinkButton),
			icon: ClipboardCopyIcon,
			action: copyPermalink,
		},
	]
})

function getDependentSearchTypes() {
	if (!project.value) return []

	if (project.value.actualProjectType !== 'mod') {
		return [isServerProject.value ? 'server' : project.value.actualProjectType]
	}

	const loaders = project.value.loaders ?? []
	const projectTypes = []

	if (loaders.some((loader) => tags.value.loaderData.modLoaders.includes(loader))) {
		projectTypes.push('mod')
	}
	if (loaders.some((loader) => tags.value.loaderData.allPluginLoaders.includes(loader))) {
		projectTypes.push('plugin')
	}
	if (loaders.some((loader) => tags.value.loaderData.dataPackLoaders.includes(loader))) {
		projectTypes.push('datapack')
	}

	return projectTypes.length > 0 ? projectTypes : ['mod']
}

const createCanonicalUrl = () =>
	project.value ? `https://modrinth.com/project/${project.value.id}` : undefined

useHead({
	link: [
		{
			rel: 'canonical',
			href: createCanonicalUrl,
		},
	],
})

if (!route.name.startsWith('type-project-settings')) {
	useSeoMeta({
		title: () => title.value,
		description: () => description.value,
		ogTitle: () => title.value,
		ogDescription: () => project.value?.description ?? PROJECT_NOT_FOUND_DESCRIPTION,
		ogImage: () =>
			project.value
				? (project.value?.icon_url ?? 'https://cdn.modrinth.com/placeholder-square.png')
				: 'https://cdn.modrinth.com/not-found.png',
		ogUrl: createCanonicalUrl,
		robots: () => (project.value?.status === 'approved' ? 'all' : 'noindex'),
	})
} else {
	useSeoMeta({
		robots: 'noindex',
		ogUrl: createCanonicalUrl,
	})
}

const onUserCollectProject = useClientTry(userCollectProject)

function handleProjectHeaderPrimary(event) {
	if (isServerProject.value) {
		handlePlayServerProject()
	} else {
		downloadModal.value?.show(event)
	}
}

function dismissProjectHeaderCreateServerPrompt() {
	flags.value.showProjectPageCreateServersTooltip = false
	saveFeatureFlags()
}

function followProjectFromHeader() {
	if (!project.value) return
	userFollowProject(project.value)
}

function reportProjectFromHeader() {
	if (!project.value) return
	if (auth.value.user) {
		reportProject(project.value.id)
	} else {
		navigateTo(getSignInRouteObj(route, getReportPath('project', project.value.id)))
	}
}

watch(
	[versionsV3, _versionsV3Error],
	([data, error]) => {
		debug('versionsV3 query changed', {
			hasData: !!data,
			count: data?.length ?? 0,
			error: error?.message ?? null,
			projectId: projectId.value,
		})
	},
	{ immediate: true },
)

async function setProcessing() {
	if (
		patchStatusMutation.isPending.value ||
		!canSubmitProjectForReview(projectValidation.value, reviewSubmissionLoading.value)
	) {
		return false
	}
	reviewSubmissionPending.value = true
	startLoading()
	try {
		const validation = await refreshProjectValidation()
		if (!canSubmitProjectForReview(validation, false)) return false
		await patchStatusMutation.mutateAsync({
			projectId: project.value.id,
			status: 'processing',
			threadId: project.value.thread_id,
		})
		return true
	} catch {
		return false
	} finally {
		reviewSubmissionPending.value = false
		stopLoading()
	}
}

async function patchProject(resData, quiet = false, throwOnError = false) {
	startLoading()

	return new Promise((resolve, reject) => {
		patchProjectMutation.mutate(
			{ projectId: project.value.id, data: resData, optimistic: !throwOnError },
			{
				onSuccess: async () => {
					if (!quiet) {
						addNotification({
							title: formatMessage(messages.projectUpdated),
							text: formatMessage(messages.projectUpdatedMessage),
							type: 'success',
						})
					}
					resolve(true)
				},
				onError: (error) => (throwOnError ? reject(error) : resolve(false)),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function patchProjectV3(resData, quiet = false, throwOnError = false) {
	startLoading()

	return new Promise((resolve, reject) => {
		patchProjectV3Mutation.mutate(
			{ projectId: project.value.id, data: resData, optimistic: !throwOnError },
			{
				onSuccess: async () => {
					if (!quiet) {
						addNotification({
							title: formatMessage(messages.projectUpdated),
							text: formatMessage(messages.projectUpdatedMessage),
							type: 'success',
						})
					}
					resolve(true)
				},
				onError: (error) => (throwOnError ? reject(error) : resolve(false)),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function patchIcon(icon) {
	startLoading()

	return new Promise((resolve) => {
		patchIconMutation.mutate(
			{ projectId: project.value.id, icon },
			{
				onSuccess: () => resolve(true),
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function createGalleryItem(
	file,
	title,
	description,
	featured,
	ordering,
	throwOnError = false,
) {
	startLoading()

	return new Promise((resolve, reject) => {
		createGalleryItemMutation.mutate(
			{
				projectId: project.value.id,
				file,
				title,
				description,
				featured,
				ordering,
			},
			{
				onSuccess: () => resolve(true),
				onError: (error) => (throwOnError ? reject(error) : resolve(false)),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function editGalleryItem(
	imageUrl,
	title,
	description,
	featured,
	ordering,
	throwOnError = false,
) {
	startLoading()

	return new Promise((resolve, reject) => {
		editGalleryItemMutation.mutate(
			{
				projectId: project.value.id,
				imageUrl,
				title,
				description,
				featured,
				ordering,
			},
			{
				onSuccess: () => resolve(true),
				onError: (error) => (throwOnError ? reject(error) : resolve(false)),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function deleteGalleryItem(imageUrl) {
	startLoading()

	return new Promise((resolve) => {
		deleteGalleryItemMutation.mutate(
			{ projectId: project.value.id, imageUrl },
			{
				onSuccess: () => resolve(true),
				onError: () => resolve(false),
				onSettled: () => stopLoading(),
			},
		)
	})
}

async function copyId() {
	await navigator.clipboard.writeText(project.value.id)
}

async function copyPermalink() {
	await navigator.clipboard.writeText(`${config.public.siteUrl}/project/${project.value.id}`)
}

const collapsedChecklist = useLocalStorage(
	computed(() => `project-checklist-collapsed-${projectId.value ?? ''}`),
	false,
)

const showModerationChecklist = ref(false)
const collapsedModerationChecklist = useLocalStorage('collapsed-moderation-checklist', false)

function consumeShowChecklistHistoryState() {
	if (!import.meta.client) return false
	if (!window.history?.state?.showChecklist) return false

	const state = { ...window.history.state }
	delete state.showChecklist
	window.history.replaceState(state, '', window.location.href)
	return true
}

function setModerationChecklistOpen(open) {
	showModerationChecklist.value = open
}

async function openModerationChecklistFromMenu() {
	const projectId = project.value?.id
	if (!projectId) return

	await moderationQueue.ready
	setModerationChecklistOpen(true)
}

watch(
	() => project.value?.id,
	async (projectId, _previousProjectId, onCleanup) => {
		if (!import.meta.client || !projectId) return

		let cancelled = false
		onCleanup(() => {
			cancelled = true
		})

		const openedFromNavigation = consumeShowChecklistHistoryState()
		await moderationQueue.ready
		if (cancelled) return

		if (openedFromNavigation) {
			setModerationChecklistOpen(true)
			return
		}

		const storedState = await loadChecklistState(projectId)
		if (cancelled) return

		if (storedState !== null) {
			showModerationChecklist.value = storedState.open ?? false
			return
		}

		const shouldRecoverFromQueue =
			moderationQueue.isQueueMode && moderationQueue.getCurrentProjectId() === projectId
		showModerationChecklist.value = shouldRecoverFromQueue
	},
	{ immediate: true },
)

function triggerDownloadAnimation() {
	overTheTopDownloadAnimation.value = true
	setTimeout(() => (overTheTopDownloadAnimation.value = false), 500)
}

const INSTALL_CONTEXT_QUERY_KEYS = ['sid', 'wid', 'from', 'shi']

function getInstallContextQueryString(keys = INSTALL_CONTEXT_QUERY_KEYS) {
	const params = new URLSearchParams()

	for (const key of keys) {
		const value = route.query[key]
		if (Array.isArray(value)) {
			for (const item of value) {
				if (item != null) {
					params.append(key, item)
				}
			}
		} else if (value != null) {
			params.append(key, value)
		}
	}

	const queryString = params.toString()
	return queryString ? `?${queryString}` : ''
}

function withInstallContextQuery(path) {
	return `${path}${getInstallContextQueryString()}`
}

async function deleteVersion(id) {
	if (!id) return

	startLoading()

	await client.labrinth.versions_v3.deleteVersion(id)

	await invalidateProject()

	stopLoading()
}

// moderation project keybinds

onMounted(() => window.addEventListener('keydown', handleKeybinds))
onUnmounted(() => window.removeEventListener('keydown', handleKeybinds))

function handleKeybinds(event) {
	if (!isStaff(auth.value.user)) return
	if (
		!showModerationChecklist.value &&
		!modSettings.value.get(moderationSettings.General.ProjectKeybinds)
	)
		return

	keybinds.value.handle(event, {
		project: projectRaw.value,
		scope: 'project',
		notifyCopied,
	})
}

const navLinks = computed(() => {
	if (!project.value) return []

	const routeType = route.params.type || project.value.project_type
	const projectUrl = `/${routeType}/${project.value.slug ? project.value.slug : project.value.id}`

	const galleryCount =
		routeType === 'server'
			? project.value.gallery.filter((item) => item.name === '__mc_server_banner__').length
			: project.value.gallery.length

	return [
		{
			label: formatMessage(messages.descriptionTab),
			href: withInstallContextQuery(projectUrl),
		},
		{
			label: formatMessage(messages.galleryTab),
			href: withInstallContextQuery(`${projectUrl}/gallery`),
			shown: galleryCount > 0 || !!currentMember.value,
		},
		{
			label: formatMessage(messages.changelogTab),
			href: withInstallContextQuery(`${projectUrl}/changelog`),
			shown:
				hasVersions.value && projectV3Loaded.value && projectV3.value?.minecraft_server == null,
			onHover: loadVersions,
		},
		{
			label: formatMessage(messages.versionsTab),
			href: withInstallContextQuery(`${projectUrl}/versions`),
			shown:
				(hasVersions.value || !!currentMember.value) &&
				projectV3Loaded.value &&
				projectV3.value?.minecraft_server == null,
			subpages: [`${projectUrl}/version/`],
			onHover: loadVersions,
		},
		{
			label: formatMessage(messages.moderationTab),
			href: withInstallContextQuery(`${projectUrl}/moderation`),
			shown: !!currentMember.value,
		},
	]
})

provideProjectPageContext({
	// Data refs
	projectV2: project,
	projectV3,
	currentMember,
	allMembers,
	organization,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
	// Lazy version loading
	versions,
	versionsLoading,
	versionsLoaded,
	// Lazy dependencies loading
	dependencies,
	dependenciesLoading: computed(() => dependenciesLoading.value),
	cdnDownloadReason: readonly(downloadReason),

	thread,

	// Invalidate all project queries (auto-refetches active ones)
	invalidate: invalidateProject,
	refreshProjectValidation,

	// Lazy loading
	loadVersions,
	loadDependencies,

	// Mutation functions
	patchProject,
	patchProjectV3,
	patchIcon,
	setProcessing,

	// Gallery mutation functions
	createGalleryItem,
	editGalleryItem,
	deleteGalleryItem,
})
</script>

<style lang="scss" scoped>
.settings-header {
	display: flex;
	flex-direction: row;
	gap: var(--spacing-card-sm);
	align-items: center;
	margin-bottom: var(--spacing-card-bg);

	.settings-header__icon {
		flex-shrink: 0;
	}

	.settings-header__text {
		h1 {
			font-size: var(--font-size-md);
			margin-top: 0;
			margin-bottom: var(--spacing-card-sm);
		}
	}
}

.popout-checkbox {
	padding: var(--gap-sm) var(--gap-md);
	white-space: nowrap;

	&:hover {
		filter: brightness(0.95);
	}
}

.popout-heading {
	padding: var(--gap-sm) var(--gap-md);
	padding-bottom: 0;
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.menu-text {
	padding: 0 var(--gap-md);
	font-size: var(--font-size-nm);
	color: var(--color-secondary);
}

.menu-search {
	margin: var(--gap-sm) var(--gap-md);
	width: calc(100% - var(--gap-md) * 2);
}

.collections-list {
	max-height: 40rem;
	overflow-y: auto;
	background-color: var(--color-bg);
	border-radius: var(--radius-md);
	margin: var(--gap-sm) var(--gap-md);
	padding: var(--gap-sm);
}

.normal-page__info:empty {
	display: none;
}

.over-the-top-download-animation {
	position: fixed;
	z-index: 100;
	inset: 0;
	display: flex;
	justify-content: center;
	align-items: center;
	pointer-events: none;
	scale: 0.5;
	transition: all 0.5s ease-out;
	opacity: 1;

	&.animation-hidden {
		scale: 0.8;
		opacity: 0;

		.animation-ring-1 {
			width: 25rem;
			height: 25rem;
		}

		.animation-ring-2 {
			width: 50rem;
			height: 50rem;
		}

		.animation-ring-3 {
			width: 100rem;
			height: 100rem;
		}
	}

	> div {
		position: relative;
		display: flex;
		justify-content: center;
		align-items: center;
		width: fit-content;
		height: fit-content;

		> * {
			position: absolute;
			scale: 1;
			transition: all 0.2s ease-out;
			width: 20rem;
			height: 20rem;
		}
	}
}

.servers-popup {
	box-shadow:
		0 0 12px 1px color-mix(in srgb, var(--color-brand) 60%, transparent),
		var(--shadow-floating);

	&::before {
		width: 0;
		height: 0;
		border-left: 6px solid transparent;
		border-right: 6px solid transparent;
		border-bottom: 6px solid var(--color-button-bg);
		content: ' ';
		position: absolute;
		top: -7px;
		left: 17px;
	}
	&::after {
		width: 0;
		height: 0;
		border-left: 5px solid transparent;
		border-right: 5px solid transparent;
		border-bottom: 5px solid var(--color-raised-bg);
		content: ' ';
		position: absolute;
		top: -5px;
		left: 18px;
	}
}

.new-page {
	column-gap: 1.5rem;
}
</style>
