import type { Labrinth } from '@modrinth/api-client'
import type { DeepReadonly, Ref, ShallowRef } from 'vue'

import { createContext } from '.'

export const PROJECT_DEP_MARKER_QUERY = { dep: '1' } as const

export type CdnDownloadReason = 'standalone' | 'dependency'

export interface ProjectPageContext {
	// Data refs
	projectV2: Ref<Labrinth.Projects.v2.Project>
	projectV3: Ref<Labrinth.Projects.v3.Project>
	currentMember: Ref<Labrinth.Projects.v3.TeamMember | null>
	allMembers: Ref<Labrinth.Projects.v3.TeamMember[]>
	organization: Ref<Labrinth.Projects.v3.Organization | null>
	projectValidation: Ref<Labrinth.Projects.v3.ProjectValidationResponse | null>
	projectValidationLoading: Ref<boolean>
	projectLinksNetworkValidationLoading: Ref<boolean>
	// Lazy version loading (client-side only)
	versions: Ref<Labrinth.Versions.v3.Version[] | null>
	versionsLoading: Ref<boolean>
	versionsLoaded: Ref<boolean>
	// Lazy dependencies loading (client-side only)
	dependencies: Ref<Labrinth.Projects.v2.DependencyInfo | null>
	dependenciesLoading: Ref<boolean>

	thread: ShallowRef<Labrinth.Threads.v3.Thread | null | undefined>

	cdnDownloadReason: DeepReadonly<Ref<CdnDownloadReason>>

	// Invalidate all project queries (auto-refetches active ones)
	invalidate: () => Promise<void>
	refreshProjectValidation: () => Promise<Labrinth.Projects.v3.ProjectValidationResponse | null>

	// Lazy loading
	loadVersions: () => void
	loadDependencies: () => void

	// Mutation functions
	patchProject: (
		data: Record<string, unknown>,
		quiet?: boolean,
		throwOnError?: boolean,
	) => Promise<boolean>
	patchProjectV3: (
		data: Record<string, unknown>,
		quiet?: boolean,
		throwOnError?: boolean,
	) => Promise<boolean>
	patchIcon: (icon: File) => Promise<boolean>
	setProcessing: () => Promise<boolean>
	createGalleryItem: (
		file: File,
		title?: string,
		description?: string,
		featured?: boolean,
		ordering?: number,
		throwOnError?: boolean,
	) => Promise<boolean>
	editGalleryItem: (
		imageUrl: string,
		title?: string,
		description?: string,
		featured?: boolean,
		ordering?: number,
		throwOnError?: boolean,
	) => Promise<boolean>
	deleteGalleryItem: (imageUrl: string) => Promise<boolean>
}

export const [injectProjectPageContext, provideProjectPageContext] =
	createContext<ProjectPageContext>('root', 'projectPageContext')
