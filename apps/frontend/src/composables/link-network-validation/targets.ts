import type { Labrinth } from '@modrinth/api-client'
import MarkdownIt from 'markdown-it'

export interface LinkTarget {
	field: string
	url: string
	image: boolean
}

const markdown = new MarkdownIt({ html: true, linkify: true })
markdown.linkify.set({ fuzzyLink: false, fuzzyEmail: false })

export function projectLinkTargets(
	project: Pick<Labrinth.Projects.v3.Project, 'link_urls' | 'license' | 'description'>,
	patch: Labrinth.Projects.v3.EditProjectRequest = {},
): LinkTarget[] {
	const links = {
		...Object.fromEntries(Object.entries(project.link_urls).map(([field, link]) => [field, link.url])),
		...patch.link_urls,
	}
	const targets: LinkTarget[] = Object.entries(links).flatMap(([field, url]) =>
		url ? [{ field, url, image: false }] : [],
	)
	const licenseUrl = patch.license_url !== undefined ? patch.license_url : project.license.url
	if (licenseUrl) {
		targets.push({ field: 'license', url: licenseUrl, image: false })
	}
	targets.sort((a, b) => a.field.localeCompare(b.field))
	const seen = new Set<string>()
	const addDescription = (value: string, image: boolean) => {
		if (!value || value.startsWith('#')) return
		try {
			const url = new URL(value, 'https://modrinth.com/')
			if (!['http:', 'https:'].includes(url.protocol)) return
			const key = `${image}:${url.href}`
			if (seen.has(key)) return
			seen.add(key)
			targets.push({ field: 'description', url: url.href, image })
		} catch {
			return
		}
	}
	const html = markdown.render(patch.description ?? project.description)
	for (const tag of html.matchAll(/<(a|img|source|video|audio|iframe)\b[^>]*>/gi)) {
		for (const attribute of tag[0].matchAll(
			/\s(href|src|srcset|poster)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/gi,
		)) {
			const name = attribute[1].toLowerCase()
			const value = markdown.utils.unescapeAll(attribute[2] ?? attribute[3] ?? attribute[4])
			const image = tag[1].toLowerCase() === 'img' || ['poster', 'srcset'].includes(name)
			if (name === 'srcset') {
				if (!value.startsWith('data:')) {
					for (const candidate of value.split(',')) {
						addDescription(candidate.trim().split(/\s+/)[0], image)
					}
				}
			} else {
				addDescription(value, image)
			}
		}
	}
	return targets
}
