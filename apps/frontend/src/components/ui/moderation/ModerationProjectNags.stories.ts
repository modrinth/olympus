import type { Labrinth } from '@modrinth/api-client'
import { type Nag, nagDefinitions, toProjectNag } from '@modrinth/moderation'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import { DEFAULT_FEATURE_FLAGS } from '../../../composables/featureFlags'
import ModerationProjectNags from './ModerationProjectNags.vue'

const categories = [
	'adventure',
	'cursed',
	'decoration',
	'economy',
	'equipment',
	'food',
	'game-mechanics',
	'library',
	'magic',
]

function createProject(status: Labrinth.Projects.v2.ProjectStatus): Labrinth.Projects.v2.Project {
	return {
		id: 'storybook-publishing-checklist',
		slug: 'storybook-publishing-checklist',
		project_type: 'mod',
		actualProjectType: 'mod',
		team: 'storybook-team',
		organization: null,
		title: 'Publishing Checklist Preview',
		description: 'A project fixture for previewing the publishing checklist.',
		body: '',
		published: '2026-01-01T00:00:00.000Z',
		updated: '2026-01-01T00:00:00.000Z',
		status,
		license: {
			id: 'MIT',
			name: 'MIT License',
		},
		client_side: 'required',
		server_side: 'optional',
		downloads: 0,
		followers: 0,
		categories,
		additional_categories: [],
		game_versions: [],
		loaders: [],
		versions: [],
		gallery: [],
		thread_id: 'storybook-thread',
		monetization_status: 'monetized',
	}
}

function createProjectV3(status: Labrinth.Projects.v2.ProjectStatus): Labrinth.Projects.v3.Project {
	return {
		id: 'storybook-publishing-checklist',
		slug: 'storybook-publishing-checklist',
		project_types: ['mod'],
		games: ['minecraft:java-edition'],
		team_id: 'storybook-team',
		name: 'Publishing Checklist Preview',
		summary: 'A short summary.',
		description: '',
		published: '2026-01-01T00:00:00.000Z',
		updated: '2026-01-01T00:00:00.000Z',
		status,
		license: {
			id: 'MIT',
			name: 'MIT License',
		},
		downloads: 0,
		followers: 0,
		categories,
		additional_categories: [],
		loaders: [],
		mrpack_loaders: [],
		versions: [],
		link_urls: {},
		gallery: [],
		thread_id: 'storybook-thread',
		monetization_status: 'monetized',
		side_types_migration_review_status: 'reviewed',
		environment: ['unknown'],
	}
}

const tags = {
	categories: [],
	rejectedStatuses: ['rejected', 'withheld'],
	gameVersions: [],
	loaders: [],
}

const previewValues = {
	count: 3,
	fullUrl: 'https://example.com/prohibited-link',
	languageCount: 12,
	length: 12,
	maxTagCount: 8,
	minChars: 50,
	projectType: 'mod',
	status: 'rejected',
	tagCount: 9,
	tags: '16x|32x',
	totalAvailableTags: 20,
	url: 'https://example.com/prohibited-link',
	value: 'example',
}

const suggestionKinds = new Set<Labrinth.Projects.v3.NormalizedProjectNagKind>([
	'add-icon',
	'feature-gallery-image',
	'add-links',
	'add-links-server',
	'select-language',
	'select-tags',
	'check-disclosures',
])

const warningKinds = new Set<Labrinth.Projects.v3.NormalizedProjectNagKind>([
	'missing-alt-text',
	'too-many-languages',
	'too-many-tags',
	'multiple-resolution-tags',
	'moderator-feedback',
])

function createValidationNag(
	kind: Labrinth.Projects.v3.NormalizedProjectNagKind,
	details: Labrinth.Projects.v3.ProjectNag['details'] = {},
): Labrinth.Projects.v3.ProjectNag {
	return {
		kind: kind.replaceAll('-', '_') as Labrinth.Projects.v3.ProjectNagKind,
		severity: suggestionKinds.has(kind)
			? 'suggestion'
			: warningKinds.has(kind)
				? 'warning'
				: 'required',
		details: { ...previewValues, ...details },
	}
}

interface NagPreviewVariant {
	details?: Labrinth.Projects.v3.ProjectNag['details']
	projectType?: string
}

const linkFields = [
	'issues',
	'source',
	'wiki',
	'discord',
	'site',
	'store',
	'license',
	'description',
	'patreon',
	'bmac',
	'paypal',
	'github',
	'ko-fi',
	'other',
]

const fieldLinkReasons = [
	'global_blocklist_match',
	'external_blocklist_match',
	'wrong_field',
	'ip_address',
	'malformed',
	'not_in_allowlist',
	'duplicate',
	'unverifiable',
]

const nagVariants: Partial<
	Record<Labrinth.Projects.v3.NormalizedProjectNagKind, NagPreviewVariant[]>
> = {
	'link-validation': [
		{},
		...fieldLinkReasons.flatMap((reason) =>
			linkFields.map((field, index) => ({
				details: {
					reason,
					field,
					other_field: linkFields[(index + 1) % linkFields.length],
				},
			})),
		),
		{ details: { reason: 'download', field: 'description' } },
		{ details: { reason: 'discord_invite', field: 'discord' } },
		{ details: { reason: 'source_repository', field: 'source' } },
	],
	'upload-gallery-image': [{}, { projectType: 'resourcepack' }, { projectType: 'shader' }],
	'long-headers': [{}, { details: { count: 1 } }],
	'all-tags-selected': [{}, { details: { totalAvailableTags: 1 } }],
	'multiple-resolution-tags': [{}, { details: { count: 1, tags: ['16x'] } }],
	'too-many-tags': [{}, { details: { tagCount: 1 } }],
	'too-many-tags-server': [{}, { details: { tagCount: 1 } }],
	'too-many-languages': [{}, { details: { languageCount: 1 } }],
}

const everyNag: Nag[] = Object.keys(nagDefinitions).flatMap((kind) => {
	const normalizedKind = kind as Labrinth.Projects.v3.NormalizedProjectNagKind
	return (nagVariants[normalizedKind] ?? [{}]).map((variant, index) => {
		const nag = toProjectNag(
			createValidationNag(normalizedKind, variant.details),
			variant.projectType ?? previewValues.projectType,
		)
		return { ...nag, id: `${nag.id}:preview:${index}` }
	})
})

const draftNags = [
	'add-icon',
	'add-description',
	'upload-version',
	'select-environment',
	'add-links',
	'too-many-tags',
	'check-disclosures',
] satisfies Labrinth.Projects.v3.NormalizedProjectNagKind[]

const meta = {
	title: 'Website/Moderation/PublishingChecklist',
	component: ModerationProjectNags,
	beforeEach: () => {
		const previousFlags = Object.getOwnPropertyDescriptor(globalThis, 'useFeatureFlags')
		Object.defineProperty(globalThis, 'useFeatureFlags', {
			configurable: true,
			value: () => ref({ ...DEFAULT_FEATURE_FLAGS }),
		})
		return () => {
			if (previousFlags) Object.defineProperty(globalThis, 'useFeatureFlags', previousFlags)
			else Reflect.deleteProperty(globalThis, 'useFeatureFlags')
		}
	},
	decorators: [
		(story) => ({
			components: { story },
			template: '<div class="mx-auto w-full p-4" style="max-width: 1100px"><story /></div>',
		}),
	],
	parameters: {
		layout: 'fullscreen',
	},
	args: {
		project: createProject('draft'),
		projectV3: createProjectV3('draft'),
		versions: [],
		collapsed: false,
		disableHorizontalScroll: true,
		routeName: 'type-project',
		tags,
	},
	render: (args) => ({
		components: { ModerationProjectNags },
		setup() {
			const collapsed = ref(args.collapsed)
			return { args, collapsed }
		},
		template: /* html */ `
			<ModerationProjectNags
				v-bind="args"
				:collapsed="collapsed"
				@toggle-collapsed="collapsed = !collapsed"
			/>
		`,
	}),
} satisfies Meta<typeof ModerationProjectNags>

export default meta
type Story = StoryObj<typeof meta>

export const EntirePublishingChecklist: Story = {
	args: {
		validationNags: draftNags.map((kind) => createValidationNag(kind)),
	},
	parameters: {
		docs: {
			description: {
				story: 'The complete expanded checklist for an unfinished draft project.',
			},
		},
	},
}

export const EveryNag: Story = {
	args: {
		nags: everyNag,
		validationNags: draftNags.map((kind) => createValidationNag(kind)),
	},
	parameters: {
		docs: {
			description: {
				story:
					'Every registered nag and its message variants, including link reasons and fields, license errors, gallery project types, and singular/plural copy.',
			},
		},
	},
}

export const RejectedProject: Story = {
	args: {
		project: createProject('rejected'),
		projectV3: createProjectV3('rejected'),
		validationNags: [createValidationNag('moderator-feedback')],
	},
}

export const WithheldProject: Story = {
	args: {
		project: createProject('withheld'),
		projectV3: createProjectV3('withheld'),
		validationNags: [createValidationNag('moderator-feedback')],
	},
}
