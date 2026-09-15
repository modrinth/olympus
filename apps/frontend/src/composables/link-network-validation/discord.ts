import { type AbstractModrinthClient, ModrinthApiError } from '@modrinth/api-client'

import { normalizeProjectUrl } from '../../helpers/project-url.ts'

const DISCORD_API = 'https://discord.com/api'
const DISCORD_INVITE_DOMAINS = ['discord.com', 'discordapp.com']
const DISCORD_SHORT_INVITE_DOMAIN = 'discord.gg'

export function discordInviteCode(value: string): string | undefined {
	try {
		const url = new URL(normalizeProjectUrl(value))
		if (url.protocol !== 'https:' || url.username || url.password) return
		const host = url.hostname.replace(/\.$/, '')
		const matchesDomain = (domain: string) => host === domain || host.endsWith(`.${domain}`)
		const parts = url.pathname.split('/').filter(Boolean)
		const code =
			matchesDomain(DISCORD_SHORT_INVITE_DOMAIN) && parts.length === 1
				? parts[0]
				: DISCORD_INVITE_DOMAINS.some(matchesDomain) && parts.length === 2 && parts[0] === 'invite'
					? parts[1]
					: undefined
		return code && /^[a-zA-Z0-9_-]+$/.test(code) ? code : undefined
	} catch {
		return undefined
	}
}

export async function checkDiscordInvite(
	client: AbstractModrinthClient,
	code: string,
	signal: AbortSignal,
): Promise<boolean> {
	try {
		const invite = await client.request<Record<string, unknown>>(
			`/invites/${encodeURIComponent(code)}`,
			{
				api: DISCORD_API,
				version: 10,
				skipAuth: true,
				headers: { 'Content-Type': '', Accept: 'application/json' },
				retry: false,
				timeout: 5000,
				signal,
			},
		)
		return (
			!invite.guild ||
			(typeof invite.expires_at === 'string' && Date.parse(invite.expires_at) < Date.now())
		)
	} catch (error) {
		if (error instanceof ModrinthApiError && error.statusCode === 404) return true
		throw error
	}
}
