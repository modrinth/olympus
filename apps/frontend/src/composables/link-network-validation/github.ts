import { type AbstractModrinthClient, ModrinthApiError } from '@modrinth/api-client'

export function githubRepositoryPath(value: string, allowIssueTracker = false): string | null {
	try {
		const url = new URL(value)
		if (
			url.protocol !== 'https:' ||
			url.username ||
			url.password ||
			url.port ||
			!['github.com', 'www.github.com'].includes(url.hostname)
		)
			return null
		const parts = url.pathname.split('/').filter(Boolean)
		if (parts.length !== 2 && !(allowIssueTracker && parts.length === 3 && parts[2] === 'issues'))
			return null
		const [owner, name] = parts
		if (
			[
				'sponsors',
				'settings',
				'orgs',
				'users',
				'topics',
				'collections',
				'marketplace',
				'features',
				'enterprise',
				'login',
				'join',
				'explore',
				'search',
				'organizations',
			].includes(owner.toLowerCase())
		)
			return null
		const repository = name.replace(/\.git$/, '')
		if (!/^[a-z\d-]+$/i.test(owner) || !/^[a-z\d_.-]+$/i.test(repository)) return null
		return `/${owner}/${repository}`
	} catch {
		return null
	}
}

/** Verifies repositories and issue trackers through GitHub's public API, resolving renames. */
export async function probeGithubRepository(
	client: AbstractModrinthClient,
	url: string,
	path: string,
	signal: AbortSignal,
): Promise<{ url: string; accessible: boolean | null }> {
	const unavailable = { url, accessible: false }
	try {
		const originalUrl = new URL(url)
		const issueTracker = originalUrl.pathname.split('/').filter(Boolean)[2] === 'issues'
		const repository = await client.request<Record<string, unknown> | null>(path, {
			api: 'https://api.github.com',
			version: 'repos',
			skipAuth: true,
			headers: { 'Content-Type': '', Accept: 'application/vnd.github+json' },
			retry: false,
			timeout: 5000,
			signal,
		})
		if (
			repository?.private !== false ||
			repository.disabled !== false ||
			typeof repository.html_url !== 'string' ||
			!githubRepositoryPath(repository.html_url) ||
			new URL(repository.html_url).hostname !== 'github.com'
		)
			return unavailable
		if (issueTracker) {
			if (repository.has_issues === false) return unavailable
			if (repository.has_issues !== true) return { url, accessible: null }
			const destination = new URL(repository.html_url)
			destination.pathname = `${destination.pathname.replace(/\/$/, '')}/issues`
			destination.search = originalUrl.search
			destination.hash = originalUrl.hash
			return { url: destination.href, accessible: true }
		}
		return { url: repository.html_url, accessible: true }
	} catch (error) {
		signal.throwIfAborted()
		return {
			url,
			accessible:
				error instanceof ModrinthApiError &&
				error.statusCode !== undefined &&
				error.statusCode >= 400 &&
				error.statusCode < 500
					? false
					: null,
		}
	}
}
