import type { Labrinth } from '@modrinth/api-client'
import { normalizeProjectNagKind, toProjectFieldMessage } from '@modrinth/moderation'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

export type ProjectSettingsField =
	| 'name'
	| 'summary'
	| 'icon'
	| 'description'
	| 'gallery-text'
	| 'gallery-images'
	| 'license'
	| 'custom-license'
	| 'license-url'
	| 'external-links'
	| 'link-field'
	| 'source-availability'
	| 'permissions'
	| 'server-region'
	| 'server-languages'
	| 'java-address'
	| 'server-compatibility'
	| 'tags'
	| 'versions'
	| 'version-environment'
	| 'disclosure-text'
	| 'disclosures'
	| 'moderation'

export const projectNagFields = {
	name: [
		'project-name-slur',
		'project-name-profanity',
		'project-name-non-standard-text',
		'project-name-version',
		'minecraft-title-clause',
	],
	summary: [
		'project-summary-slur',
		'project-summary-profanity',
		'project-summary-non-standard-text',
		'project-summary-non-english',
		'project-summary-matches-title',
		'summary-too-short',
		'project-summary-spam',
		'summary-special-formatting',
		'project-summary-links',
	],
	icon: ['add-icon'],
	description: [
		'link-validation',
		'project-description-slur',
		'project-description-profanity',
		'project-description-non-standard-text',
		'project-description-non-english',
		'project-description-matches-summary',
		'add-description',
		'description-too-short',
		'project-description-spam',
		'long-headers',
		'description-ends-with-header',
		'adjacent-headers',
		'missing-alt-text',
	],
	'gallery-text': ['gallery-text-slur', 'gallery-text-profanity', 'gallery-text-non-standard'],
	'gallery-images': ['upload-gallery-image', 'feature-gallery-image'],
	license: ['select-license'],
	'custom-license': ['add-custom-license-details'],
	'license-url': ['link-validation'],
	'link-field': ['link-validation'],
	'external-links': ['add-links', 'add-links-server'],
	'source-availability': ['gpl-license-source-required'],
	permissions: ['review-permissions'],
	'server-region': ['select-country'],
	'server-languages': ['all-languages', 'too-many-languages', 'select-language'],
	'java-address': ['add-java-address'],
	'server-compatibility': ['select-compatibility'],
	tags: [
		'select-tags',
		'too-many-tags',
		'too-many-tags-server',
		'multiple-resolution-tags',
		'all-tags-selected',
	],
	versions: ['upload-version'],
	'version-environment': ['select-environment'],
	'disclosure-text': ['disclosures-special-formatting'],
	disclosures: ['check-disclosures'],
	moderation: ['moderator-feedback'],
} as const satisfies Record<
	ProjectSettingsField,
	readonly Labrinth.Projects.v3.NormalizedProjectNagKind[]
>

function appliesToDetails(
	nag: Labrinth.Projects.v3.ProjectNag,
	detailField?: string,
	detailIndex?: number,
) {
	if (!detailField) return true
	const field = nag.details?.field
	const fields = nag.details?.fields
	if (typeof field === 'string' && field !== detailField) return false
	if (Array.isArray(fields) && !fields.includes(detailField)) return false
	if (typeof nag.details?.gallery_index === 'number' && nag.details.gallery_index !== detailIndex) {
		return false
	}
	return true
}

export function useProjectNagMessages(
	field: ProjectSettingsField,
	detailField?: string,
	detailIndex?: () => number,
) {
	const { projectValidation, projectV2 } = injectProjectPageContext()
	const kinds = new Set<string>(projectNagFields[field])

	return computed(() =>
		(projectValidation.value?.nags ?? [])
			.filter((nag) => {
				if (nag.severity === 'suggestion') return false
				const kind = normalizeProjectNagKind(nag.kind)
				return (
					kind !== null && kinds.has(kind) && appliesToDetails(nag, detailField, detailIndex?.())
				)
			})
			.map((nag) => toProjectFieldMessage(nag, projectV2.value.project_type)),
	)
}
