import { type AbstractModrinthClient, type Labrinth, ModrinthApiError } from '@modrinth/api-client'

import { checkDiscordInvite, discordInviteCode } from './discord.ts'
import { githubRepositoryPath, probeGithubRepository } from './github.ts'
import type { LinkTarget } from './targets.ts'

type ProjectNag = Labrinth.Projects.v3.ProjectNag

export function linkNag(target: LinkTarget, reason: string): ProjectNag {
	return {
		kind: 'link_validation',
		severity: 'required',
		details: { field: target.field, url: target.url, reason },
	}
}

async function checkInvite(
	client: AbstractModrinthClient,
	target: LinkTarget,
	signal: AbortSignal,
): Promise<ProjectNag[]> {
	const code = discordInviteCode(target.url)
	if (!code) return []
	try {
		return (await checkDiscordInvite(client, code, signal))
			? [linkNag(target, 'discord_invite')]
			: []
	} catch (error) {
		signal.throwIfAborted()
		return error instanceof ModrinthApiError &&
			error.statusCode !== undefined &&
			error.statusCode >= 400 &&
			error.statusCode < 500
			? [linkNag(target, 'unverifiable')]
			: []
	}
}

export async function mapConcurrent<T, R>(items: T[], run: (item: T) => Promise<R>): Promise<R[]> {
	let next = 0
	const results = new Array<R>(items.length)
	await Promise.all(
		Array.from({ length: Math.min(8, items.length) }, async () => {
			while (next < items.length) {
				const index = next++
				results[index] = await run(items[index])
			}
		}),
	)
	return results
}

export async function validateLinkNetwork(
	client: AbstractModrinthClient,
	targets: LinkTarget[],
	signal: AbortSignal,
): Promise<ProjectNag[]> {
	const deadline = AbortSignal.any([signal, AbortSignal.timeout(25_000)])
	return (
		await mapConcurrent(targets, async (target) => {
			try {
				deadline.throwIfAborted()
				if (target.field === 'discord') {
					return await checkInvite(client, target, deadline)
				}
				const githubRepository = !target.image && githubRepositoryPath(target.url, true)
				if (!githubRepository) return []
				const observed = await probeGithubRepository(
					client,
					target.url,
					githubRepository,
					deadline,
				)
				return observed.accessible === false ? [linkNag(target, 'unverifiable')] : []
			} catch {
				signal.throwIfAborted()
				return []
			}
		})
	).flat()
}
