import type { AbstractModrinthClient, Labrinth } from '@modrinth/api-client'
import type { QueryClient } from '@tanstack/vue-query'

import { normalizeProjectUrl } from '../../helpers/project-url.ts'
import { discordInviteCode } from './discord.ts'
import { mapConcurrent, validateLinkNetwork } from './network.ts'
import type { LinkTarget } from './targets.ts'

export function linkValue(field: string, url: string): string {
	return (field === 'discord' && discordInviteCode(url)) || normalizeProjectUrl(url)
}

function linkQueryKey(projectId: string, target: LinkTarget) {
	return [
		'project-link-validation',
		projectId,
		{ ...target, url: linkValue(target.field, target.url) },
	] as const
}

export function getCachedLinkNags(
	queryClient: QueryClient,
	projectId: string,
	targets: LinkTarget[],
): Labrinth.Projects.v3.ProjectNag[] {
	return targets.flatMap((target) => {
		const nags = queryClient.getQueryData<Labrinth.Projects.v3.ProjectNag[]>(
			linkQueryKey(projectId, target),
		)
		return (nags ?? []).map((nag) => ({ ...nag, details: { ...nag.details, url: target.url } }))
	})
}

export async function validateCachedLinkNetwork(
	queryClient: QueryClient,
	client: AbstractModrinthClient,
	projectId: string,
	checks: LinkTarget[],
	signal: AbortSignal,
	fresh = false,
): Promise<Labrinth.Projects.v3.ProjectNag[]> {
	const deadline = AbortSignal.any([signal, AbortSignal.timeout(25_000)])
	const results = await mapConcurrent(checks, async (check) => {
		deadline.throwIfAborted()
		const nags = await queryClient.fetchQuery({
			queryKey: linkQueryKey(projectId, check),
			queryFn: ({ signal: querySignal }) =>
				validateLinkNetwork(client, [check], AbortSignal.any([deadline, querySignal])),
			staleTime: fresh ? 0 : Infinity,
			retry: false,
		})
		deadline.throwIfAborted()
		return nags.map((nag) => ({ ...nag, details: { ...nag.details, url: check.url } }))
	})
	return results.flat()
}
