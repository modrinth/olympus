import { type Labrinth, ModrinthApiError } from '@modrinth/api-client'
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { useMounted } from '@vueuse/core'
import { computed, type MaybeRefOrGetter, onScopeDispose, toValue } from 'vue'

import { getCachedLinkNags, validateCachedLinkNetwork } from './queries'
import { PROJECT_REVIEW_VALIDATION_ERROR } from './submission'
import { projectLinkTargets } from './targets'

export function useProjectLinkValidation(
	projectId: MaybeRefOrGetter<string>,
	project: MaybeRefOrGetter<Labrinth.Projects.v3.Project | undefined>,
	enabled: MaybeRefOrGetter<boolean>,
) {
	const client = injectModrinthClient()
	const mounted = useMounted()
	const queryClient = useQueryClient()
	const saveController = new AbortController()
	onScopeDispose(() => saveController.abort())
	const targets = computed(() => {
		const value = toValue(project)
		return value ? projectLinkTargets(value) : []
	})
	let checkedProjectId: string | undefined
	let refreshNetwork = false
	const query = useQuery({
		queryKey: computed(() => ['project', toValue(projectId), 'link-validation', targets.value]),
		enabled: computed(() => mounted.value && toValue(enabled) && !!toValue(project)),
		placeholderData: () => getCachedLinkNags(queryClient, toValue(projectId), targets.value),
		queryFn: async ({ signal }) => {
			const fresh = refreshNetwork || checkedProjectId !== toValue(projectId)
			checkedProjectId = toValue(projectId)
			refreshNetwork = false
			return validateCachedLinkNetwork(
				queryClient,
				client,
				toValue(projectId),
				targets.value,
				signal,
				fresh,
			)
		},
		staleTime: 0,
		retry: false,
		refetchOnWindowFocus: false,
		refetchOnReconnect: false,
	})

	return {
		validateSave: async (patch: Labrinth.Projects.v3.EditProjectRequest = {}) => {
			const value = toValue(project)
			if (!value) throw new Error(PROJECT_REVIEW_VALIDATION_ERROR)
			if (value.status !== 'processing') return
			const id = toValue(projectId)
			const nags = await validateCachedLinkNetwork(
				queryClient,
				client,
				id,
				projectLinkTargets(value, patch),
				saveController.signal,
				true,
			)
			if (id !== toValue(projectId)) throw new Error(PROJECT_REVIEW_VALIDATION_ERROR)
			if (nags.some((nag) => nag.severity === 'required')) {
				throw new ModrinthApiError(PROJECT_REVIEW_VALIDATION_ERROR, {
					responseData: { details: { nags } },
				})
			}
		},
		nags: computed(() => query.data.value ?? []),
		isChecking: computed(
			() => toValue(enabled) && (!mounted.value || query.isPending.value || query.isFetching.value),
		),
		isError: query.isError,
		refresh: () => {
			refreshNetwork = true
			return query.refetch({ cancelRefetch: false })
		},
	}
}
