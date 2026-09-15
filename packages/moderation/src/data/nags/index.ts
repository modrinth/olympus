import type { Labrinth } from '@modrinth/api-client'

import type { Nag, NagStatus } from '../../types/nags.ts'
import { descriptionNags } from './description.ts'
import { nagDestinations } from './destinations.ts'
import { disclosureNags } from './disclosures.ts'
import { galleryNags } from './gallery.ts'
import { galleryTextNags } from './gallery-text.ts'
import { iconNags } from './icon.ts'
import { licenseNags } from './license.ts'
import { linkNags } from './links.ts'
import { moderationNags } from './moderation.ts'
import { nameNags } from './name.ts'
import { permissionNags } from './permissions.ts'
import { serverSettingNags } from './server-settings.ts'
import { summaryNags } from './summary.ts'
import { tagNags } from './tags.ts'
import type { FieldValidationMessage, NagDefinition } from './types.ts'
import { versionNags } from './versions.ts'

export const nagDefinitions = {
	...nameNags,
	...summaryNags,
	...iconNags,
	...galleryNags,
	...galleryTextNags,
	...descriptionNags,
	...licenseNags,
	...linkNags,
	...permissionNags,
	...serverSettingNags,
	...tagNags,
	...versionNags,
	...disclosureNags,
	...moderationNags,
} satisfies Record<Labrinth.Projects.v3.NormalizedProjectNagKind, NagDefinition>

export { nagDestinations } from './destinations.ts'
export type { FieldValidationMessage } from './types.ts'

export function normalizeProjectNagKind(
	kind: string,
): Labrinth.Projects.v3.NormalizedProjectNagKind | null {
	const normalized = kind.replaceAll('_', '-') as Labrinth.Projects.v3.NormalizedProjectNagKind
	return normalized in nagDefinitions ? normalized : null
}

function toNagStatus(severity: Labrinth.Projects.v3.ProjectNagSeverity): NagStatus {
	return severity
}

function getNagDescription(
	definition: NagDefinition,
	nag: Labrinth.Projects.v3.ProjectNag,
	projectType?: string,
) {
	return typeof definition.description === 'function'
		? definition.description({ nag, projectType })
		: definition.description
}

export function getProjectNagValues(
	nag: Labrinth.Projects.v3.ProjectNag,
	projectType?: string,
): Record<string, string | number | boolean> {
	const details = nag.details ?? {}
	const camelCaseKey = (key: string) =>
		key.replaceAll(/_([a-z])/g, (_, letter) => letter.toUpperCase())
	const values = Array.isArray(details.values)
		? details.values.join(', ')
		: typeof details.values === 'string' || typeof details.values === 'number'
			? details.values
			: undefined
	const tagValues = Array.isArray(details.tags)
		? details.tags
		: typeof details.tags === 'string'
			? details.tags.split('|')
			: []
	const tags = tagValues
		.map((tag) => String(tag).replace('8x-', '8x or lower').replace('512x+', '512x or higher'))
		.join(', ')
	const formatted: Record<string, string | number | boolean> = {}
	for (const [key, value] of Object.entries(details)) {
		if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
			formatted[camelCaseKey(key)] = value
		}
	}
	const detailProjectType =
		typeof details.project_type === 'string' ? details.project_type : undefined
	const resolvedProjectType = projectType ?? detailProjectType

	return {
		...formatted,
		...(normalizeProjectNagKind(nag.kind) === 'link-validation'
			? {
					linkField: details.field === 'ko-fi' ? 'kofi' : String(details.field ?? 'other'),
					otherLinkField:
						details.other_field === 'ko-fi' ? 'kofi' : String(details.other_field ?? 'other'),
				}
			: {}),
		...(values !== undefined && details.value === undefined ? { value: values } : {}),
		...(tagValues.length > 0 ? { tags } : {}),
		...(tagValues.length > 0 && details.count === undefined ? { count: tagValues.length } : {}),
		...(details.type === undefined && resolvedProjectType ? { type: resolvedProjectType } : {}),
	}
}

export function toProjectNag(nag: Labrinth.Projects.v3.ProjectNag, projectType?: string): Nag {
	const kind = normalizeProjectNagKind(nag.kind)
	if (!kind) throw new Error(`Unknown project nag kind: ${nag.kind}`)
	const definition = nagDefinitions[kind]
	const destinationId =
		kind === 'link-validation' && nag.details.field === 'description'
			? 'description'
			: kind === 'link-validation' && nag.details.field === 'license'
				? 'license'
				: definition.destination
	const destination = nagDestinations[destinationId]

	return {
		id: projectNagId(nag, kind),
		title:
			typeof definition.title === 'function'
				? definition.title({ nag, projectType })
				: definition.title,
		description: getNagDescription(definition, nag, projectType),
		status: toNagStatus(nag.severity),
		shouldShow: () => true,
		link: definition.linkTitle ? { ...destination, title: definition.linkTitle } : destination,
		values: getProjectNagValues(nag, projectType),
	}
}

export function toProjectFieldMessage(
	nag: Labrinth.Projects.v3.ProjectNag,
	projectType?: string,
): FieldValidationMessage {
	const kind = normalizeProjectNagKind(nag.kind)
	if (!kind) throw new Error(`Unknown project nag kind: ${nag.kind}`)
	return {
		code: projectNagId(nag, kind),
		severity: nag.severity === 'required' ? 'error' : nag.severity,
		message: getNagDescription(nagDefinitions[kind], nag, projectType),
		values: getProjectNagValues(nag, projectType),
	}
}

function projectNagId(nag: Labrinth.Projects.v3.ProjectNag, kind: string): string {
	return kind === 'link-validation'
		? `${kind}:${JSON.stringify([nag.details.field, nag.details.url, nag.details.reason])}`
		: kind
}
