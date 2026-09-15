<script setup lang="ts">
import {
	AlignLeftIcon,
	BookTextIcon,
	ChartIcon,
	GlobeIcon,
	ImageIcon,
	InfoIcon,
	IssuesIcon,
	LinkIcon,
	ServerIcon,
	SignatureIcon,
	TagsIcon,
	UsersIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonProjectSettingsMessages,
	injectProjectPageContext,
	Toggle,
	useVIntl,
} from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'
import { useLocalStorage, useScroll } from '@vueuse/core'
import { computed } from 'vue'

import ModerationProjectNags from '~/components/ui/moderation/ModerationProjectNags.vue'
import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

const {
	projectV2: project,
	projectV3,
	versions,
	currentMember,
	setProcessing,
	projectValidation,
	projectValidationLoading,
	projectLinksNetworkValidationLoading,
} = injectProjectPageContext()

const flags = useFeatureFlags()

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)

const navItems = computed(() => {
	const base = `${project.value.project_type}/${project.value.slug ? project.value.slug : project.value.id}`

	const showEnvironment =
		projectV3.value?.project_types?.some((type) => ['mod', 'modpack'].includes(type)) &&
		isStaff(currentMember.value?.user)

	const hasPermissionsPage = computed(() =>
		projectV3.value?.project_types?.some((type) => ['modpack'].includes(type)),
	)

	const items = [
		{
			link: `/${base}/settings`,
			label: formatMessage(commonProjectSettingsMessages.general),
			icon: InfoIcon,
		},
		flags.value.newProjectGeneralSettings
			? {
					link: `/${base}/settings/general`,
					label: formatMessage(commonProjectSettingsMessages.general),
					badge: formatMessage(commonMessages.newBadge),
					icon: InfoIcon,
				}
			: null,
		isServerProject.value && {
			link: `/${base}/settings/server`,
			label: formatMessage(commonProjectSettingsMessages.server),
			icon: ServerIcon,
		},
		{
			link: `/${base}/settings/disclosures`,
			label: formatMessage(commonProjectSettingsMessages.disclosures),
			icon: IssuesIcon,
		},
		{
			link: `/${base}/settings/tags`,
			label: formatMessage(commonProjectSettingsMessages.tags),
			icon: TagsIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/description`,
			label: formatMessage(commonProjectSettingsMessages.description),
			icon: AlignLeftIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/versions`,
			label: formatMessage(commonProjectSettingsMessages.versions),
			icon: VersionIcon,
		},
		hasPermissionsPage.value && {
			link: `/${base}/settings/permissions`,
			label: formatMessage(commonProjectSettingsMessages.permissions),
			icon: SignatureIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/license`,
			label: formatMessage(commonProjectSettingsMessages.license),
			icon: BookTextIcon,
		},
		isServerProject.value && {
			link: `/${base}/settings/description`,
			label: formatMessage(commonProjectSettingsMessages.description),
			icon: AlignLeftIcon,
		},
		{
			link: `/${base}/settings/gallery`,
			label: formatMessage(commonProjectSettingsMessages.gallery),
			icon: ImageIcon,
		},
		{
			link: `/${base}/settings/links`,
			label: formatMessage(commonProjectSettingsMessages.links),
			icon: LinkIcon,
		},
		{
			link: `/${base}/settings/members`,
			label: formatMessage(commonProjectSettingsMessages.members),
			icon: UsersIcon,
		},
		!isServerProject.value && {
			link: `/${base}/settings/analytics`,
			label: formatMessage(commonProjectSettingsMessages.analytics),
			icon: ChartIcon,
		},
		!isServerProject.value && { type: 'heading', label: 'moderation', shown: showEnvironment },
		!isServerProject.value && {
			link: `/${base}/settings/environment`,
			label: formatMessage(commonProjectSettingsMessages.environment),
			icon: GlobeIcon,
			shown: showEnvironment,
		},
	]
	return items.filter(Boolean) as any[]
})

const tags = useGeneratedState()
const route = useRoute()
const collapsedChecklist = useLocalStorage(`project-checklist-collapsed-${project.value.id}`, false)

// To persist scroll position through settings pages
// This scroll code is jank asf, if anyone has a better way please do suggest it
const scroll = useScroll(window)
watch(route, () => {
	const scrollY = scroll.y.value
	setTimeout(() => window.scrollTo(0, scrollY), 10)
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
</script>

<template>
	<div class="mb-8 flex w-full flex-col gap-4">
		<ModerationProjectNags
			v-if="
				projectV3 &&
				((currentMember && (project.status === 'draft' || project.status === 'processing')) ||
					tags.rejectedStatuses.includes(project.status))
			"
			:project="project"
			:project-v3="projectV3"
			:versions="versions ?? undefined"
			:current-member="currentMember"
			:collapsed="collapsedChecklist"
			:route-name="route.name as string"
			:tags="tags"
			:validation-nags="projectValidation?.nags ?? []"
			:validation-loading="projectValidationLoading || projectLinksNetworkValidationLoading"
			:validation-available="projectValidation !== null"
			:submit-project="setProcessing"
			@toggle-collapsed="() => (collapsedChecklist = !collapsedChecklist)"
		/>
		<div class="grid gap-6 lg:grid-cols-[1fr_3fr]">
			<div>
				<NavStack :items="navItems" />
				<div v-if="isStaff(currentMember?.user)" class="mt-4 flex items-center gap-2">
					<Toggle id="moderator-see-user-ui-toggle" v-model="moderatorSeeUserUi" small />
					<label for="moderator-see-user-ui-toggle"> Show member UI </label>
				</div>
			</div>
			<div class="min-w-0">
				<NuxtPage />
			</div>
		</div>
	</div>
</template>
