import { defineMessages } from '@modrinth/ui'

import { licenseLinkMessages } from './license.ts'
import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	addTitle: { id: 'nags.add-links.title', defaultMessage: 'Add external links' },
	addServerTitle: { id: 'nags.add-links-server.title', defaultMessage: 'Add external links' },
	add: {
		id: 'nags.add-links.description',
		defaultMessage:
			'Add any relevant links to external resources, such as source code, an issue tracker, or a permanent Discord invite.',
	},
	addServer: {
		id: 'nags.add-links-server.description',
		defaultMessage:
			'Add any relevant links to external resources, such as a website, store, or a permanent Discord invite.',
	},
	gplTitle: { id: 'nags.gpl-license-source-required.title', defaultMessage: 'Provide source code' },
	gpl: {
		id: 'nags.gpl-license-source-required.description',
		defaultMessage: `Your {type}'s license requires source code to be published. Please provide a source code link, add sources files, or change the selected license.`,
	},
	linkTitle: { id: 'nags.link-validation.title', defaultMessage: 'Review this link' },
	global_blocklist_matchTitle: {
		id: 'nags.link-validation.global-blocklist-match.title',
		defaultMessage: 'Remove prohibited links',
	},
	external_blocklist_matchTitle: {
		id: 'nags.link-validation.external-blocklist-match.title',
		defaultMessage: 'Replace incorrect links',
	},
	wrong_fieldTitle: {
		id: 'nags.link-validation.wrong-field.title',
		defaultMessage: 'Fix incorrect links',
	},
	ip_addressTitle: {
		id: 'nags.link-validation.ip-address.title',
		defaultMessage: 'Remove IP Address link',
	},
	malformedTitle: {
		id: 'nags.link-validation.malformed.title',
		defaultMessage: 'Fix invalid links',
	},
	not_in_allowlistTitle: {
		id: 'nags.link-validation.not-in-allowlist.title',
		defaultMessage: 'Check link accuracy',
	},
	duplicateTitle: {
		id: 'nags.link-validation.duplicate.title',
		defaultMessage: 'Remove duplicate links',
	},
	unverifiableTitle: {
		id: 'nags.link-validation.unverifiable.title',
		defaultMessage: 'Check link availability',
	},
	downloadTitle: {
		id: 'nags.link-validation.download.title',
		defaultMessage: 'Remove download links',
	},
	discord_inviteTitle: {
		id: 'nags.link-validation.discord-invite.title',
		defaultMessage: 'Replace invalid Discord invites',
	},
	source_repositoryTitle: {
		id: 'nags.link-validation.source-repository.title',
		defaultMessage: 'Link to a source repository',
	},
	global_blocklist_match: {
		id: 'nags.link-validation.global-blocklist-match',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link uses a prohibited domain. Remove or replace this link.',
	},
	external_blocklist_match: {
		id: 'nags.link-validation.external-blocklist-match',
		defaultMessage:
			'Your {linkField} link is not allowed. Please remove this link, or replace it with something appropriate for the {linkField} link type.',
	},
	wrong_field: {
		id: 'nags.link-validation.wrong-field',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link is not correct for {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} links. Remove or replace it with the correct link.',
	},
	ip_address: {
		id: 'nags.link-validation.ip-address',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link uses an IP address. Use a URL with a domain name.',
	},
	malformed: {
		id: 'nags.link-validation.malformed',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link is invalid. Enter a complete HTTPS URL.',
	},
	not_in_allowlist: {
		id: 'nags.link-validation.not-in-allowlist',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link may not be appropriate for the link type. Please ensure that it is suitable for its intended purpose.',
	},
	duplicate: {
		id: 'nags.link-validation.duplicate',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link is also used in {otherLinkField, select, issues {Issue tracker} source {Source code} wiki {Wiki} discord {Discord invite} site {Website} store {Store} license {License} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {another field}}. Each link should be listed only once.',
	},
	unverifiable: {
		id: 'nags.link-validation.unverifiable',
		defaultMessage:
			'Your {linkField, select, issues {issue tracker} source {source code} wiki {wiki} discord {Discord invite} site {website} store {store} license {license} description {description} patreon {Patreon} bmac {Buy Me a Coffee} paypal {PayPal} github {GitHub Sponsors} kofi {Ko-fi} other {donation}} link could not be verified. Check that it is online and accessible.',
	},
	download: {
		id: 'nags.link-validation.download',
		defaultMessage:
			'Links that start a download are not allowed. Please remove any such links from your description.',
	},
	discord_invite: {
		id: 'nags.link-validation.discord-invite',
		defaultMessage:
			'Your Discord invite is invalid, expired, or does not invite users to a server. Replace it with an active server invite.',
	},
	source_repository: {
		id: 'nags.link-validation.source-repository',
		defaultMessage:
			'Your source code link must lead to a repository on a supported host platform, or to a self-hosted Gitea/Forgejo service.',
	},
	visitLinks: { id: 'nags.visit-links-settings.title', defaultMessage: 'Visit links settings' },
})

const linkReasons = {
	discord_invite: messages.discord_invite,
	global_blocklist_match: messages.global_blocklist_match,
	external_blocklist_match: messages.external_blocklist_match,
	wrong_field: messages.wrong_field,
	ip_address: messages.ip_address,
	malformed: messages.malformed,
	not_in_allowlist: messages.not_in_allowlist,
	duplicate: messages.duplicate,
	unverifiable: messages.unverifiable,
	download: messages.download,
	source_repository: messages.source_repository,
}

const linkReasonTitles = {
	discord_invite: messages.discord_inviteTitle,
	global_blocklist_match: messages.global_blocklist_matchTitle,
	external_blocklist_match: messages.external_blocklist_matchTitle,
	wrong_field: messages.wrong_fieldTitle,
	ip_address: messages.ip_addressTitle,
	malformed: messages.malformedTitle,
	not_in_allowlist: messages.not_in_allowlistTitle,
	duplicate: messages.duplicateTitle,
	unverifiable: messages.unverifiableTitle,
	download: messages.downloadTitle,
	source_repository: messages.source_repositoryTitle,
}

export const linkNags = {
	'link-validation': {
		title: ({ nag }) => {
			const reason = nag.details.reason
			if (nag.details.field === 'license') {
				if (reason === 'malformed') return licenseLinkMessages.malformedTitle
				if (reason === 'not_in_allowlist') return licenseLinkMessages.notInAllowlistTitle
			}
			return typeof reason === 'string' && reason in linkReasonTitles
				? linkReasonTitles[reason as keyof typeof linkReasonTitles]
				: messages.linkTitle
		},
		description: ({ nag }) => {
			const reason = nag.details.reason
			if (nag.details.field === 'license') {
				if (reason === 'malformed') return licenseLinkMessages.malformed
				if (reason === 'not_in_allowlist') return licenseLinkMessages.notInAllowlist
			}
			return typeof reason === 'string' && reason in linkReasons
				? linkReasons[reason as keyof typeof linkReasons]
				: messages.unverifiable
		},
		destination: 'links',
	},
	'add-links': { title: messages.addTitle, description: messages.add, destination: 'links' },
	'add-links-server': {
		title: messages.addServerTitle,
		description: messages.addServer,
		destination: 'links',
	},
	'gpl-license-source-required': {
		title: messages.gplTitle,
		description: messages.gpl,
		destination: 'links',
		linkTitle: messages.visitLinks,
	},
} satisfies NagDefinitions
