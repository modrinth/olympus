export const donationUsernamePrefixes: Record<string, string> = {
	patreon: 'https://www.patreon.com/',
	bmac: 'https://buymeacoffee.com/',
	paypal: 'https://www.paypal.me/',
	github: 'https://github.com/sponsors/',
	'ko-fi': 'https://ko-fi.com/',
}

export interface DonationInput {
	id?: string
	url: string
	input: string
	mode: 'username' | 'url'
}

export function donationUsernameFromUrl(
	platform: string | undefined,
	raw: string,
): string | undefined {
	const prefix = platform ? donationUsernamePrefixes[platform] : undefined
	if (!prefix || !raw.startsWith(prefix)) return undefined
	const username = raw.slice(prefix.length)
	if (!username || /[/?#\s]/.test(username)) return undefined
	try {
		return decodeURIComponent(username)
	} catch {
		return undefined
	}
}

export function donationInput(id?: string, url = ''): DonationInput {
	const username = donationUsernameFromUrl(id, url)
	return {
		id,
		url,
		input: username ?? url,
		mode:
			username !== undefined || (!url && id && donationUsernamePrefixes[id]) ? 'username' : 'url',
	}
}

export function setDonationInput(row: DonationInput, value: string | number, detectUrl = true) {
	row.input = String(value)
	if (detectUrl && /^https?:\/\//i.test(row.input.trim())) row.mode = 'url'
	const prefix = row.id ? donationUsernamePrefixes[row.id] : undefined
	row.url =
		row.mode === 'username' && prefix && row.input
			? prefix + encodeURIComponent(row.input)
			: row.input
}

export function toggleDonationInput(row: DonationInput) {
	row.mode = row.mode === 'username' ? 'url' : 'username'
	setDonationInput(row, row.input, false)
}
