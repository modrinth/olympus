import type { CookieOptions } from '#app'

export type ProjectDisplayMode = 'list' | 'grid' | 'gallery'
export type DarkColorTheme = 'dark' | 'oled' | 'retro'

export interface NumberFlag {
	min: number
	max: number
}

export type BooleanFlag = boolean

export type RadioFlag = ProjectDisplayMode | DarkColorTheme

export type FlagValue = BooleanFlag /* | NumberFlag | RadioFlag */

const validateValues = <K extends PropertyKey>(flags: Record<K, FlagValue>) => flags

export const DEFAULT_FEATURE_FLAGS = validateValues({
	// Developer flags
	developerMode: false,
	showThreadIds: false,
	demoMode: false,
	showVersionFilesInTable: false,
	showVersionEnvironmentColumn: false,
	showAdsWithPlus: false,
	alwaysShowChecklistAsPopup: true,
	testTaxForm: false,

	// Feature toggles
	projectTypesPrimaryNav: false,
	enableMedalPromotion: true,
	hidePlusPromoInUserMenu: false,
	projectBackground: false,
	searchBackground: false,
	advancedDebugInfo: false,
	FilesRefreshButton: false,
	showProjectPageDownloadModalServersPromo: false,
	showProjectPageCreateServersTooltip: true,
	showProjectPageQuickServerButton: false,
	newProjectGeneralSettings: false,
	serverRamAsBytesAlwaysOn: false,
	archonSentryCapture: false,
	hideRussiaCensorshipBanner: false,
	disablePrettyProjectUrlRedirects: false,
	hidePreviewBanner: false,
	i18nDebug: false,
	showDiscoverProjectButtons: false,
	useV1ContentTabAPI: true,
	labrinthApiCanary: false,
	dismissedExternalProjectsInfo: false,
	showAllBanners: false,
	alwaysIgnoreErrorBanner: false,
	showViewProdRouteBanner: false,
	showModeratorProjectMemberUi: false,
	alwaysShowPublishingChecklistForStaff: false,
	archonApiStaging: false,
	showHostingAccessInstanceAuditLog: false,
	versionDevInfoCollapsed: true,
	alwaysShowVersionDevInfo: false,
	advancedFiltersCollapsed: true,
	dismissedPhotosensitivityFilterWarning: false,
} as const)

export type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS

export type AllFeatureFlags = {
	[key in FeatureFlag]: (typeof DEFAULT_FEATURE_FLAGS)[key]
}

export type PartialFeatureFlags = Partial<AllFeatureFlags>

const getCookieOptions = () =>
	({
		maxAge: 60 * 60 * 24 * 365 * 10,
		sameSite: 'lax',
		secure: useRuntimeConfig().public.cookieSecure,
		httpOnly: false,
		path: '/',
	}) satisfies CookieOptions<PartialFeatureFlags>

export const useFeatureFlags = () =>
	useState<AllFeatureFlags>('featureFlags', () => {
		const config = useRuntimeConfig()

		const savedFlags = useCookie<PartialFeatureFlags>('featureFlags', getCookieOptions())

		if (!savedFlags.value) {
			savedFlags.value = {}
		}

		const flags: AllFeatureFlags = JSON.parse(JSON.stringify(DEFAULT_FEATURE_FLAGS))

		const overrides = config.public.featureFlagOverrides as PartialFeatureFlags
		for (const key in overrides) {
			if (key in flags) {
				const flag = key as FeatureFlag
				const value = overrides[flag] as (typeof flags)[FeatureFlag]
				flags[flag] = value
			}
		}

		for (const key in savedFlags.value) {
			if (key in flags) {
				const flag = key as FeatureFlag
				const value = savedFlags.value[flag] as (typeof flags)[FeatureFlag]
				flags[flag] = value
			}
		}

		return flags
	})

export const saveFeatureFlags = () => {
	const flags = useFeatureFlags()
	const cookie = useCookie<PartialFeatureFlags>('featureFlags', getCookieOptions())
	cookie.value = flags.value
}
