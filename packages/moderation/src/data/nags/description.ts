import { defineMessages } from '@modrinth/ui'

import type { NagDefinitions } from './types.ts'

const messages = defineMessages({
	reviewTitle: {
		id: 'nags.project-description-content.title',
		defaultMessage: 'Review the description',
	},
	matchesSummary: {
		id: 'project.text-validation.description-matches-summary',
		defaultMessage: `Your project's description should provide unique information and not repeat the project's summary.`,
	},
	addTitle: { id: 'nags.add-description.title', defaultMessage: 'Add a description' },
	add: {
		id: 'nags.add-description.description',
		defaultMessage: `A description that clearly describes your project's content, purpose, and appeal is required.`,
	},
	adjacentTitle: { id: 'nags.adjacent-headers.title', defaultMessage: 'Remove adjacent headers' },
	adjacent: {
		id: 'nags.adjacent-headers.description',
		defaultMessage: 'Headers of the same level should not be placed next to each other.',
	},
	endsTitle: {
		id: 'nags.description-ends-with-header.title',
		defaultMessage: 'Remove ending header',
	},
	ends: {
		id: 'nags.description-ends-with-header.description',
		defaultMessage: `Your project's description should not end with a header that is not followed by any text.`,
	},
	shortTitle: { id: 'nags.description-too-short.title', defaultMessage: 'Expand the description' },
	short: {
		id: 'nags.description-too-short.description',
		defaultMessage: `Your project's description is too brief. Add more details to clearly describe the project's content, purpose, and appeal.`,
	},
	longTitle: { id: 'nags.long-headers.title', defaultMessage: 'Shorten headers' },
	long: {
		id: 'nags.long-headers.description',
		defaultMessage:
			'{count, plural, one {# header} other {# headers}} in your description {count, plural, one {is} other {are}} too long. Headers should be concise and act as section titles, not full sentences.',
	},
	altTitle: { id: 'nags.missing-alt-text.title', defaultMessage: 'Add image alt text' },
	alt: {
		id: 'nags.missing-alt-text.description',
		defaultMessage:
			'Some of your images are missing alt text, which is important for accessibility, especially for visually impaired users.',
	},
	fixTitle: {
		id: 'nags.invalid-project-description.title',
		defaultMessage: 'Modify the description',
	},
	nonEnglish: {
		id: 'nags.project-description-non-english.description',
		defaultMessage: `Your project's description must be written in English or include an English translation.`,
	},
	nonStandard: {
		id: 'nags.project-description-non-standard-text.description',
		defaultMessage: `Excessive use of non-standard text characters, such as “Fancy text” or “Zalgo”, is not allowed in your project's description.`,
	},
	profanity: {
		id: 'nags.project-description-profanity.description',
		defaultMessage: `Your project's description cannot contain excessive profanity. Detected: “{value}”.`,
	},
	slur: {
		id: 'nags.project-description-slur.description',
		defaultMessage: `Your project's description must not contain offensive terms. Detected: “{value}”.`,
	},
	spamTitle: {
		id: 'nags.project-description-spam.title',
		defaultMessage: 'Remove description spam',
	},
	spam: {
		id: 'nags.project-description-spam.description',
		defaultMessage: `Repeated characters, words, or phrases cannot be used to pad your project's description.`,
	},
	translateTitle: {
		id: 'nags.project-description-non-english.title',
		defaultMessage: 'Translate the description',
	},
})

export const descriptionNags = {
	'project-description-matches-summary': {
		title: messages.reviewTitle,
		description: messages.matchesSummary,
		destination: 'description',
	},
	'add-description': {
		title: messages.addTitle,
		description: messages.add,
		destination: 'description',
	},
	'adjacent-headers': {
		title: messages.adjacentTitle,
		description: messages.adjacent,
		destination: 'description',
	},
	'description-ends-with-header': {
		title: messages.endsTitle,
		description: messages.ends,
		destination: 'description',
	},
	'description-too-short': {
		title: messages.shortTitle,
		description: messages.short,
		destination: 'description',
	},
	'long-headers': {
		title: messages.longTitle,
		description: messages.long,
		destination: 'description',
	},
	'missing-alt-text': {
		title: messages.altTitle,
		description: messages.alt,
		destination: 'description',
	},
	'project-description-non-english': {
		title: messages.translateTitle,
		description: messages.nonEnglish,
		destination: 'description',
	},
	'project-description-non-standard-text': {
		title: messages.fixTitle,
		description: messages.nonStandard,
		destination: 'description',
	},
	'project-description-profanity': {
		title: messages.fixTitle,
		description: messages.profanity,
		destination: 'description',
	},
	'project-description-slur': {
		title: messages.fixTitle,
		description: messages.slur,
		destination: 'description',
	},
	'project-description-spam': {
		title: messages.spamTitle,
		description: messages.spam,
		destination: 'description',
	},
} satisfies NagDefinitions
