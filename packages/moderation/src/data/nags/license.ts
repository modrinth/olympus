import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

export const licenseLinkMessages = defineMessages({
	malformedTitle: {
		id: 'nags.invalid-license-url.title',
		defaultMessage: 'Add a valid license link',
	},
	malformed: {
		id: 'nags.invalid-license-url.description.malformed',
		defaultMessage:
			'Your license URL appears to be malformed. Please provide a valid URL to your license text.',
	},
	notInAllowlistTitle: {
		id: 'nags.license-url.not-in-allowlist.title',
		defaultMessage: 'Check your license link',
	},
	notInAllowlist: {
		id: 'nags.license-url.not-in-allowlist.description',
		defaultMessage:
			'Your license URL is not a recognized license link. Please ensure it links directly to your license text.',
	},
})

const messages = defineMessages({
	detailsTitle: {
		id: 'nags.add-license-details.title',
		defaultMessage: 'Add license details',
	},
	details: {
		id: 'nags.add-license-details.description',
		defaultMessage: 'Add a valid URL and name or SPDX identifier for your custom license.',
	},
	selectTitle: { id: 'nags.select-license.title', defaultMessage: 'Select a license' },
	select: {
		id: 'nags.select-license.description',
		defaultMessage: 'Select the license your {type} is distributed under.',
	},
})

export const licenseNags = {
	'add-custom-license-details': {
		title: messages.detailsTitle,
		description: messages.details,
		destination: 'license',
	},
	'select-license': {
		title: messages.selectTitle,
		description: messages.select,
		destination: 'license',
	},
} satisfies NagDefinitions
