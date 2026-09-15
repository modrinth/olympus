export function normalizeProjectUrl(value: string): string {
	const url = value.trim()
	if (!url || /^[a-z][a-z\d+.-]*:/i.test(url)) return url
	return `https://${url}`
}
