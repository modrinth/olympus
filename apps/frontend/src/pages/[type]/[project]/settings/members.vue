<template>
	<div>
		<ConfirmTransferProjectModal
			v-if="transferData && project"
			ref="transferModal"
			:project="{ name: project.title, icon_url: project.icon_url }"
			:current-owner="{
				avatar_url: currentMember?.avatar_url,
				username: currentMember?.user?.username ?? '',
				role: currentMember?.role ?? 'Owner',
			}"
			:transfer-to="transferData.target"
			:on-confirm="transferData.onConfirm"
		/>
		<ConfirmModal
			ref="modal_remove"
			title="Are you sure you want to remove this project from the organization?"
			description="If you proceed, this project will no longer be managed by the organization."
			proceed-label="Remove"
			@proceed="onRemoveFromOrg"
		/>
		<Card>
			<div class="label">
				<h3>
					<span class="label__title size-card-header">Manage members</span>
				</h3>
			</div>
			<span class="label">
				<span class="label__title">Invite a member</span>
				<span class="label__description">
					Enter the Modrinth username of the person you'd like to invite to be a member of this
					project.
				</span>
			</span>
			<div class="input-group">
				<Input
					id="username"
					v-model="currentUsername"
					placeholder="Username"
					:disabled="(currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES"
					@keypress.enter="inviteTeamMember()"
				/>
				<label for="username" class="hidden">Username</label>
				<Button
					type="colored"
					color="brand"
					:disabled="(currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES"
					@click="inviteTeamMember()"
				>
					<UserPlusIcon />
					Invite
				</Button>
			</div>
			<div class="adjacent-input">
				<span class="label">
					<span class="label__title">Leave project</span>
					<span class="label__description"> Remove yourself as a member of this project. </span>
				</span>
				<Button
					type="colored"
					color="red"
					:disabled="currentMember?.is_owner"
					:title="
						currentMember?.is_owner ? 'You cannot leave the project if you are the owner!' : ''
					"
					@click="leaveProject()"
				>
					<UserXIcon />
					Leave project
				</Button>
			</div>
		</Card>
		<div
			v-for="(member, index) in allTeamMembers"
			:key="member.user.id"
			class="universal-card member"
			:class="{ open: openTeamMembers.includes(member.user.id) }"
		>
			<div class="member-header">
				<div class="info">
					<Avatar :src="member.avatar_url" :alt="member.username" size="sm" circle />
					<div class="text">
						<nuxt-link :to="'/user/' + member.user.username" class="name">
							<p>{{ member.name }}</p>
							<CrownIcon v-if="member.is_owner" v-tooltip="'Project owner'" />
						</nuxt-link>
						<p>{{ member.role }}</p>
					</div>
				</div>
				<div class="side-buttons">
					<Badge v-if="member.accepted" type="accepted" />
					<Badge v-else type="pending" />
					<IconButton
						label="Toggle details"
						class="dropdown-icon"
						@click="
							openTeamMembers.indexOf(member.user.id) === -1
								? openTeamMembers.push(member.user.id)
								: (openTeamMembers = openTeamMembers.filter((it) => it !== member.user.id))
						"
					>
						<DropdownIcon />
					</IconButton>
				</div>
			</div>
			<div class="content">
				<div class="adjacent-input">
					<label :for="`member-${allTeamMembers[index].user.username}-role`" class="w-fit">
						<span class="label__title">Role</span>
						<span class="label__description">
							The title of the role that this member plays for this project.
						</span>
					</label>
					<Input
						:id="`member-${allTeamMembers[index].user.username}-role`"
						v-model="allTeamMembers[index].role"
						:disabled="(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
					/>
				</div>
				<div class="adjacent-input">
					<label
						:for="`member-${allTeamMembers[index].user.username}-monetization-weight`"
						class="w-fit"
					>
						<span class="label__title">Monetization weight</span>
						<span class="label__description">
							Relative to all other members' monetization weights, this determines what portion of
							this project's revenue goes to this member.
						</span>
					</label>
					<Input
						:id="`member-${allTeamMembers[index].user.username}-monetization-weight`"
						v-model="allTeamMembers[index].payouts_split"
						type="number"
						:disabled="(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
					/>
				</div>
				<template v-if="!member.is_owner">
					<span class="label">
						<span class="label__title">Permissions</span>
					</span>
					<div v-if="allTeamMembers[index]" class="permissions">
						<Checkbox
							:model-value="(member?.permissions & UPLOAD_VERSION) === UPLOAD_VERSION"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION
							"
							:label="isServerProject ? 'Update content' : 'Upload version'"
							@update:model-value="allTeamMembers[index].permissions ^= UPLOAD_VERSION"
						/>
						<Checkbox
							v-if="!isServerProject"
							:model-value="(member?.permissions & DELETE_VERSION) === DELETE_VERSION"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & DELETE_VERSION) !== DELETE_VERSION
							"
							label="Delete version"
							@update:model-value="allTeamMembers[index].permissions ^= DELETE_VERSION"
						/>
						<Checkbox
							:model-value="(member?.permissions & EDIT_DETAILS) === EDIT_DETAILS"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & EDIT_DETAILS) !== EDIT_DETAILS
							"
							label="Edit details"
							@update:model-value="allTeamMembers[index].permissions ^= EDIT_DETAILS"
						/>
						<Checkbox
							:model-value="(member?.permissions & EDIT_BODY) === EDIT_BODY"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & EDIT_BODY) !== EDIT_BODY
							"
							label="Edit body"
							@update:model-value="allTeamMembers[index].permissions ^= EDIT_BODY"
						/>
						<Checkbox
							:model-value="(member?.permissions & MANAGE_INVITES) === MANAGE_INVITES"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES
							"
							label="Manage invites"
							@update:model-value="allTeamMembers[index].permissions ^= MANAGE_INVITES"
						/>
						<Checkbox
							:model-value="(member?.permissions & REMOVE_MEMBER) === REMOVE_MEMBER"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER
							"
							label="Remove member"
							@update:model-value="allTeamMembers[index].permissions ^= REMOVE_MEMBER"
						/>
						<Checkbox
							:model-value="(member?.permissions & EDIT_MEMBER) === EDIT_MEMBER"
							:disabled="(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
							label="Edit member"
							@update:model-value="allTeamMembers[index].permissions ^= EDIT_MEMBER"
						/>
						<Checkbox
							:model-value="(member?.permissions & DELETE_PROJECT) === DELETE_PROJECT"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & DELETE_PROJECT) !== DELETE_PROJECT
							"
							label="Delete project"
							@update:model-value="allTeamMembers[index].permissions ^= DELETE_PROJECT"
						/>
						<Checkbox
							:model-value="(member?.permissions & VIEW_ANALYTICS) === VIEW_ANALYTICS"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & VIEW_ANALYTICS) !== VIEW_ANALYTICS
							"
							label="View analytics"
							@update:model-value="allTeamMembers[index].permissions ^= VIEW_ANALYTICS"
						/>
						<Checkbox
							:model-value="(member?.permissions & VIEW_PAYOUTS) === VIEW_PAYOUTS"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & VIEW_PAYOUTS) !== VIEW_PAYOUTS
							"
							label="View revenue"
							@update:model-value="allTeamMembers[index].permissions ^= VIEW_PAYOUTS"
						/>
					</div>
				</template>
				<div class="input-group">
					<Button
						type="colored"
						color="brand"
						:disabled="(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
						@click="updateTeamMember(index)"
					>
						<SaveIcon />
						Save changes
					</Button>
					<Button
						v-if="!member.is_owner"
						type="colored"
						color="red"
						:disabled="(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
						@click="removeTeamMember(index)"
					>
						<UserXIcon />
						Remove member
					</Button>
					<Button
						v-if="!member.is_owner && currentMember?.is_owner && member.accepted"
						@click="openTransferModal(index, $event)"
					>
						<TransferIcon />
						Transfer ownership
					</Button>
				</div>
			</div>
		</div>
		<section class="universal-card">
			<div class="label">
				<span class="label__title size-card-header">Organization</span>
			</div>
			<div v-if="organization">
				<p>
					This project is managed by {{ organization.name }}. The defaults for member permissions
					are set in the
					<nuxt-link :to="`/organization/${organization.slug}/settings/members`">
						organization settings
					</nuxt-link>
					. You may override them below.
				</p>
				<ButtonLink
					type="quiet"
					interaction="none"
					:to="`/organization/${organization.slug}`"
					class="universal-card recessed org !h-auto !w-full !shrink !items-stretch !justify-start !whitespace-normal !p-[var(--spacing-card-bg)]"
				>
					<Avatar :src="organization.icon_url" :alt="organization.name" size="md" />
					<div class="details">
						<div class="title">
							{{ organization.name }}
						</div>
						<div class="description">
							{{ organization.description }}
						</div>
						<span class="stat-bar">
							<div class="stats">
								<UsersIcon />
								<span>
									{{ acceptedOrgMembers.length }} member<template
										v-if="acceptedOrgMembers.length !== 1"
										>s</template
									>
								</span>
							</div>
						</span>
					</div>
				</ButtonLink>
			</div>
			<p v-else>
				This project is not managed by an organization. If you are the member of any organizations,
				you can transfer management to one of them.
			</p>
			<div v-if="!organization" class="flex gap-2">
				<Combobox
					v-model="selectedOrganizationId"
					:options="organizationOptions"
					:searchable="true"
					search-placeholder="Select organization..."
					force-direction="up"
					:disabled="!currentMember?.is_owner || organizationOptions.length === 0"
				/>
				<Button
					type="colored"
					color="brand"
					:disabled="!selectedOrganization"
					@click="openTransferToOrgModal($event)"
				>
					<CheckIcon />
					<span class="w-max"> Transfer management </span>
				</Button>
			</div>
			<Button v-if="organization" @click="$refs.modal_remove.show()">
				<OrganizationIcon />
				Remove from organization
			</Button>
		</section>
		<div
			v-for="(member, index) in allOrgMembers"
			:key="member.user.id"
			class="universal-card member"
			:class="{ open: openTeamMembers.includes(member.user.id) }"
		>
			<div class="member-header">
				<div class="info">
					<Avatar :src="member.user.avatar_url" :alt="member.user.username" size="sm" circle />
					<div class="text">
						<nuxt-link :to="'/user/' + member.user.username" class="name">
							<p>{{ member.user.username }}</p>
							<CrownIcon v-if="member.is_owner" v-tooltip="'Organization owner'" />
						</nuxt-link>
						<p>{{ member.role }}</p>
					</div>
				</div>
				<div class="side-buttons">
					<Badge v-if="member.accepted" type="accepted" />
					<Badge v-else type="pending" />
					<IconButton
						label="Toggle details"
						class="dropdown-icon"
						@click="
							openTeamMembers.indexOf(member.user.id) === -1
								? openTeamMembers.push(member.user.id)
								: (openTeamMembers = openTeamMembers.filter((it) => it !== member.user.id))
						"
					>
						<DropdownIcon />
					</IconButton>
				</div>
			</div>
			<div class="content">
				<div class="adjacent-input">
					<label :for="`member-${allOrgMembers[index].user.username}-override-perms`" class="w-fit">
						<span class="label__title">Override values</span>
						<span class="label__description">
							Override organization default values and assign custom permissions, roles, and
							monetization weights to this user on the project.
						</span>
					</label>
					<Toggle
						:id="`member-${allOrgMembers[index].user.username}-override-perms`"
						v-model="allOrgMembers[index].override"
						:disabled="(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER"
					/>
				</div>
				<div class="adjacent-input">
					<label :for="`member-${allOrgMembers[index].user.username}-role`" class="w-fit">
						<span class="label__title">Role</span>
						<span class="label__description">
							The title of the role that this member plays for this project.
						</span>
					</label>
					<Input
						:id="`member-${allOrgMembers[index].user.username}-role`"
						v-model="allOrgMembers[index].role"
						:disabled="
							(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
							!allOrgMembers[index].override
						"
					/>
				</div>
				<div class="adjacent-input">
					<label
						:for="`member-${allOrgMembers[index].user.username}-monetization-weight`"
						class="w-fit"
					>
						<span class="label__title">Monetization weight</span>
						<span class="label__description">
							Relative to all other members' monetization weights, this determines what portion of
							this project's revenue goes to this member.
						</span>
					</label>
					<Input
						:id="`member-${allOrgMembers[index].user.username}-monetization-weight`"
						v-model="allOrgMembers[index].payouts_split"
						type="number"
						:disabled="
							(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
							!allOrgMembers[index].override
						"
					/>
				</div>
				<template v-if="!member.is_owner">
					<span class="label">
						<span class="label__title">Permissions</span>
					</span>
					<div class="permissions">
						<Checkbox
							:model-value="(member?.permissions & UPLOAD_VERSION) === UPLOAD_VERSION"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & UPLOAD_VERSION) !== UPLOAD_VERSION ||
								!allOrgMembers[index].override
							"
							:label="isServerProject ? 'Update content' : 'Upload version'"
							@update:model-value="allOrgMembers[index].permissions ^= UPLOAD_VERSION"
						/>
						<Checkbox
							v-if="!isServerProject"
							:model-value="(member?.permissions & DELETE_VERSION) === DELETE_VERSION"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & DELETE_VERSION) !== DELETE_VERSION ||
								!allOrgMembers[index].override
							"
							label="Delete version"
							@update:model-value="allOrgMembers[index].permissions ^= DELETE_VERSION"
						/>
						<Checkbox
							:model-value="(member?.permissions & EDIT_DETAILS) === EDIT_DETAILS"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & EDIT_DETAILS) !== EDIT_DETAILS ||
								!allOrgMembers[index].override
							"
							label="Edit details"
							@update:model-value="allOrgMembers[index].permissions ^= EDIT_DETAILS"
						/>
						<Checkbox
							:model-value="(member?.permissions & EDIT_BODY) === EDIT_BODY"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & EDIT_BODY) !== EDIT_BODY ||
								!allOrgMembers[index].override
							"
							label="Edit body"
							@update:model-value="allOrgMembers[index].permissions ^= EDIT_BODY"
						/>
						<Checkbox
							:model-value="(member?.permissions & MANAGE_INVITES) === MANAGE_INVITES"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & MANAGE_INVITES) !== MANAGE_INVITES ||
								!allOrgMembers[index].override
							"
							label="Manage invites"
							@update:model-value="allOrgMembers[index].permissions ^= MANAGE_INVITES"
						/>
						<Checkbox
							:model-value="(member?.permissions & REMOVE_MEMBER) === REMOVE_MEMBER"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & REMOVE_MEMBER) !== REMOVE_MEMBER ||
								!allOrgMembers[index].override
							"
							label="Remove member"
							@update:model-value="allOrgMembers[index].permissions ^= REMOVE_MEMBER"
						/>
						<Checkbox
							:model-value="(member?.permissions & EDIT_MEMBER) === EDIT_MEMBER"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								!allOrgMembers[index].override
							"
							label="Edit member"
							@update:model-value="allOrgMembers[index].permissions ^= EDIT_MEMBER"
						/>
						<Checkbox
							:model-value="(member?.permissions & DELETE_PROJECT) === DELETE_PROJECT"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & DELETE_PROJECT) !== DELETE_PROJECT ||
								!allOrgMembers[index].override
							"
							label="Delete project"
							@update:model-value="allOrgMembers[index].permissions ^= DELETE_PROJECT"
						/>
						<Checkbox
							:model-value="(member?.permissions & VIEW_ANALYTICS) === VIEW_ANALYTICS"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & VIEW_ANALYTICS) !== VIEW_ANALYTICS ||
								!allOrgMembers[index].override
							"
							label="View analytics"
							@update:model-value="allOrgMembers[index].permissions ^= VIEW_ANALYTICS"
						/>
						<Checkbox
							:model-value="(member?.permissions & VIEW_PAYOUTS) === VIEW_PAYOUTS"
							:disabled="
								(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
								(currentMember?.permissions & VIEW_PAYOUTS) !== VIEW_PAYOUTS ||
								!allOrgMembers[index].override
							"
							label="View revenue"
							@update:model-value="allOrgMembers[index].permissions ^= VIEW_PAYOUTS"
						/>
					</div>
				</template>
				<div class="input-group">
					<!--
					if we save changes and update an org member which:
					- is not currently overridden (!allOrgMembers[index].oldOverride)
					- and we're not changing them to be overridden (!allOrgMembers[index].override)

					then we end up editing an org member which, in the backend, doesn't exist.
					the api doesn't let us do that, we can only do:
					- !override -> override: POST member
					- override -> !override: DELETE member
					- override -> override: PATCH member
					- !override -> !override: do nothing

					we don't allow clicking the button in that last case.
					-->
					<Button
						type="colored"
						color="brand"
						:disabled="
							(currentMember?.permissions & EDIT_MEMBER) !== EDIT_MEMBER ||
							(!allOrgMembers[index].oldOverride && !allOrgMembers[index].override)
						"
						@click="updateOrgMember(index)"
					>
						<SaveIcon />
						Save changes
					</Button>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup>
import {
	CheckIcon,
	CrownIcon,
	DropdownIcon,
	OrganizationIcon,
	SaveIcon,
	TransferIcon,
	UserPlusIcon,
	UsersIcon,
	UserXIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Badge,
	Button,
	ButtonLink,
	Card,
	Checkbox,
	Combobox,
	commonProjectSettingsMessages,
	ConfirmModal,
	IconButton,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	Toggle,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import ConfirmTransferProjectModal from '~/components/ui/ConfirmTransferProjectModal.vue'
import { removeSelfFromTeam } from '~/helpers/teams.js'

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const {
	projectV2: project,
	projectV3,
	organization,
	allMembers,
	currentMember,
	invalidate,
} = injectProjectPageContext()

useProjectSettingsHeadTitle(commonProjectSettingsMessages.members)

const isServerProject = computed(() => projectV3.value?.minecraft_server != null)

const auth = await useAuth()

const allTeamMembers = ref([])
const allOrgMembers = ref([])

const acceptedOrgMembers = computed(() => {
	return organization.value?.members?.filter((x) => x.accepted) || []
})

function initMembers() {
	const orgMembers = organization.value?.members || []

	const selectedMembersForOrg = orgMembers.map((partialOrgMember) => {
		const foundMember = allMembers.value.find((tM) => tM.user.id === partialOrgMember.user.id)

		const base = foundMember ?? partialOrgMember
		return {
			...base,
			override: !!foundMember,
			oldOverride: !!foundMember,
			is_owner: partialOrgMember.is_owner,
		}
	})

	allOrgMembers.value = selectedMembersForOrg

	allTeamMembers.value = allMembers.value
		.filter((x) => !selectedMembersForOrg.some((y) => y.user.id === x.user.id))
		.map((x) => ({ ...x }))
}

watch([allMembers, organization, project, currentMember], initMembers)
initMembers()

const currentUsername = ref('')
const openTeamMembers = ref([])
const selectedOrganizationId = ref('')
const transferData = ref(null)
const transferModal = ref(null)

const { data: organizations } = useQuery({
	queryKey: computed(() => ['user', auth.value?.user?.id, 'organizations']),
	queryFn: () => client.labrinth.users_v2.getOrganizations(auth.value?.user.id),
	enabled: computed(() => !!auth.value?.user?.id),
})

const organizationOptions = computed(() =>
	(organizations.value ?? []).map((organization) => ({
		value: organization.id,
		label: organization.name,
	})),
)

const selectedOrganization = computed(() =>
	(organizations.value ?? []).find((org) => org.id === selectedOrganizationId.value),
)

const UPLOAD_VERSION = 1 << 0
const DELETE_VERSION = 1 << 1
const EDIT_DETAILS = 1 << 2
const EDIT_BODY = 1 << 3
const MANAGE_INVITES = 1 << 4
const REMOVE_MEMBER = 1 << 5
const EDIT_MEMBER = 1 << 6
const DELETE_PROJECT = 1 << 7
const VIEW_ANALYTICS = 1 << 8
const VIEW_PAYOUTS = 1 << 9

const onAddToOrg = useClientTry(async () => {
	if (!selectedOrganizationId.value) return

	await client.labrinth.organizations_v3.addProject(selectedOrganizationId.value, {
		project_id: project.value.id,
	})

	await updateMembers()

	addNotification({
		title: 'Project transferred',
		text: 'Your project has been transferred to the organization.',
		type: 'success',
	})
})

const onRemoveFromOrg = useClientTry(async () => {
	if (!project.value.organization || !auth.value?.user?.id) return

	await client.labrinth.organizations_v3.removeProject(
		project.value.organization,
		project.value.id,
		{ new_owner: auth.value.user.id },
	)

	await updateMembers()

	addNotification({
		title: 'Project removed',
		text: 'Your project has been removed from the organization.',
		type: 'success',
	})
})

const leaveProject = async () => {
	await removeSelfFromTeam(project.value.team)
	navigateTo('/dashboard/projects')
}

const inviteTeamMember = async () => {
	startLoading()

	try {
		const user = await client.labrinth.users_v2.get(currentUsername.value)
		await client.labrinth.teams_v2.addMember(project.value.team, {
			user_id: user.id.trim(),
		})
		currentUsername.value = ''
		await updateMembers()
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data?.description || err?.message || err || 'Unknown error',
			type: 'error',
		})
	}

	stopLoading()
}

const removeTeamMember = async (index) => {
	startLoading()

	try {
		await client.labrinth.teams_v2.removeMember(
			project.value.team,
			allTeamMembers.value[index].user.id,
		)
		await updateMembers()
		addNotification({
			title: 'Member removed',
			text: "Your project's member has been removed.",
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data?.description || err?.message || err || 'Unknown error',
			type: 'error',
		})
	}

	stopLoading()
}

const updateTeamMember = async (index) => {
	startLoading()

	try {
		const data = !allTeamMembers.value[index].is_owner
			? {
					permissions: allTeamMembers.value[index].permissions,
					role: allTeamMembers.value[index].role,
					payouts_split: allTeamMembers.value[index].payouts_split,
				}
			: {
					payouts_split: allTeamMembers.value[index].payouts_split,
					role: allTeamMembers.value[index].role,
				}

		await client.labrinth.teams_v2.editMember(
			project.value.team,
			allTeamMembers.value[index].user.id,
			data,
		)
		await updateMembers()
		addNotification({
			title: 'Member(s) updated',
			text: "Your project's member(s) has been updated.",
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data?.description || err?.message || err || 'Unknown error',
			type: 'error',
		})
	}

	stopLoading()
}

const openTransferModal = (index, e) => {
	transferData.value = {
		target: {
			avatar_url: allTeamMembers.value[index]?.avatar_url,
			username: allTeamMembers.value[index]?.user?.username,
			role: allTeamMembers.value[index]?.role || 'Member',
		},
		onConfirm: () => transferOwnership(index),
	}
	nextTick(() => {
		transferModal.value?.show(e)
	})
}

const openTransferToOrgModal = (e) => {
	if (!selectedOrganization.value) return
	transferData.value = {
		target: {
			avatar_url: selectedOrganization.value.icon_url,
			name: selectedOrganization.value.name,
		},
		onConfirm: () => onAddToOrg(),
	}
	nextTick(() => {
		transferModal.value?.show(e)
	})
}

const transferOwnership = async (index) => {
	startLoading()

	try {
		await client.labrinth.teams_v2.transferOwnership(project.value.team, {
			user_id: allTeamMembers.value[index].user.id,
		})
		addNotification({
			title: 'Member ownership transferred',
			text: `${allTeamMembers.value[index].user.username} is now the owner of the project.`,
			type: 'success',
		})
		await updateMembers()
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data?.description || err?.message || err || 'Unknown error',
			type: 'error',
		})
	}

	stopLoading()
}

async function updateOrgMember(index) {
	startLoading()

	try {
		if (allOrgMembers.value[index].override && !allOrgMembers.value[index].oldOverride) {
			await client.labrinth.teams_v2.addMember(project.value.team, {
				permissions: allOrgMembers.value[index].permissions,
				role: allOrgMembers.value[index].role,
				payouts_split: allOrgMembers.value[index].payouts_split,
				user_id: allOrgMembers.value[index].user.id,
			})
		} else if (!allOrgMembers.value[index].override && allOrgMembers.value[index].oldOverride) {
			await client.labrinth.teams_v2.removeMember(
				project.value.team,
				allOrgMembers.value[index].user.id,
			)
		} else {
			await client.labrinth.teams_v2.editMember(
				project.value.team,
				allOrgMembers.value[index].user.id,
				{
					permissions: allOrgMembers.value[index].permissions,
					role: allOrgMembers.value[index].role,
					payouts_split: allOrgMembers.value[index].payouts_split,
				},
			)
		}
		await updateMembers()
		addNotification({
			title: 'Member(s) updated',
			text: "Your project's member(s) has been updated.",
			type: 'success',
		})
	} catch (err) {
		addNotification({
			title: 'An error occurred',
			text: err?.data?.description || err?.message || err || 'Unknown error',
			type: 'error',
		})
	}

	stopLoading()
}

const updateMembers = async () => {
	await invalidate()
}
</script>

<style lang="scss" scoped>
.org {
	display: grid;
	grid-template-columns: auto 1fr;
	gap: var(--gap-md);

	.details {
		display: flex;
		flex-direction: column;
		gap: var(--gap-sm);

		.title {
			color: var(--color-contrast);
			font-weight: 600;
			font-size: var(--font-size-lg);
		}

		.description {
			color: var(--color-secondary);
		}

		.stat-bar {
			display: flex;
			align-items: center;
			gap: var(--gap-md);
			margin-top: auto;
		}

		.stats {
			display: flex;
			align-items: center;
			gap: var(--gap-xs);

			svg {
				color: var(--color-secondary);
			}
		}
	}
}

.member {
	.member-header {
		display: flex;
		justify-content: space-between;

		.info {
			display: flex;

			.text {
				margin: auto 0 auto 0.5rem;
				font-size: var(--font-size-sm);

				.name {
					font-weight: bold;

					display: flex;
					align-items: center;
					gap: 0.25rem;

					svg {
						color: var(--color-orange);
					}
				}

				p {
					margin: 0.2rem 0;
				}
			}
		}

		.side-buttons {
			display: flex;
			align-items: center;

			.dropdown-icon {
				margin-left: 1rem;

				svg {
					transition: 150ms ease transform;
				}
			}
		}
	}

	.content {
		display: none;
		flex-direction: column;
		padding-top: var(--spacing-card-md);

		.main-info {
			margin-bottom: var(--spacing-card-lg);
		}

		.permissions {
			margin-bottom: var(--spacing-card-md);
			max-width: 45rem;
			display: grid;
			grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
			grid-gap: 0.5rem;
		}
	}

	&.open {
		.member-header {
			.dropdown-icon svg {
				transform: rotate(180deg);
			}
		}

		.content {
			display: flex;
		}
	}
}
</style>
