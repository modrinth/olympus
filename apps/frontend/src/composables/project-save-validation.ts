import type { Labrinth } from '@modrinth/api-client'
import { normalizeProjectNagKind, toProjectFieldMessage } from '@modrinth/moderation'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { projectNagFields, type ProjectSettingsField } from './project-nag-validation'

function matchesField(nag: Labrinth.Projects.v3.ProjectNag, field: string, detailField = field) {
	const kinds: readonly string[] | undefined = Object.hasOwn(projectNagFields, field)
		? projectNagFields[field as ProjectSettingsField]
		: undefined
	const kind = normalizeProjectNagKind(nag.kind)
	if (kinds && (!kind || !kinds.includes(kind))) return false
	if (typeof nag.details.field === 'string') return nag.details.field === detailField
	if (Array.isArray(nag.details.fields)) return nag.details.fields.includes(detailField)
	return kinds !== undefined
}

/** Keeps rejected-save messages attached to the exact values that were submitted. */
export function useProjectSaveValidation(state: () => unknown) {
	const { projectV2 } = injectProjectPageContext()
	const rejected = ref<Labrinth.Projects.v3.ProjectNag[]>([])
	const rejectedState = ref('')
	const showMessages = computed(
		() =>
			projectV2.value.status === 'processing' && rejectedState.value === JSON.stringify(state()),
	)
	const messages = computed(() =>
		showMessages.value ? rejected.value.map((nag) => toProjectFieldMessage(nag)) : [],
	)

	const hasErrors = computed(() => messages.value.some((message) => message.severity === 'error'))

	function snapshot() {
		return JSON.stringify(state()) ?? ''
	}

	function capture(error: unknown, submittedState: string): boolean {
		if (projectV2.value.status !== 'processing') return false
		let value = error
		for (let depth = 0; depth < 5; depth++) {
			if (typeof value !== 'object' || value === null) return false
			const data = value as Record<string, unknown>
			const details = data.details
			if (typeof details === 'object' && details !== null && 'nags' in details) {
				const nags = details.nags
				if (!Array.isArray(nags)) return false
				const recognized = nags.filter(
					(nag): nag is Labrinth.Projects.v3.ProjectNag =>
						typeof nag === 'object' &&
						nag !== null &&
						typeof nag.kind === 'string' &&
						normalizeProjectNagKind(nag.kind) !== null &&
						['required', 'warning', 'suggestion'].includes(nag.severity) &&
						typeof nag.details === 'object' &&
						nag.details !== null,
				)
				rejected.value = recognized.filter((nag) => nag.severity !== 'suggestion')
				rejectedState.value = submittedState
				return recognized.length > 0
			}
			value = data.responseData ?? data.data ?? data.originalError ?? data.cause
		}
		return false
	}

	function forField(field: string, detailField = field) {
		if (!showMessages.value) return []
		return rejected.value
			.filter((nag) => matchesField(nag, field, detailField))
			.map((nag) => toProjectFieldMessage(nag))
	}

	function withoutFields(fields: (string | [field: string, detailField: string])[]) {
		if (!showMessages.value) return []
		return rejected.value
			.filter(
				(nag) =>
					!fields.some((field) =>
						Array.isArray(field) ? matchesField(nag, ...field) : matchesField(nag, field),
					),
			)
			.map((nag) => toProjectFieldMessage(nag))
	}

	function clear() {
		rejected.value = []
		rejectedState.value = ''
	}

	return { capture, clear, messages, hasErrors, forField, withoutFields, snapshot }
}
