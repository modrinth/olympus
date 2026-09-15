import type { RawDecimal } from '../../utils/types'
import type { ISO3166 } from '../iso3166/types'

export namespace Labrinth {
	export namespace Content {
		export namespace v3 {
			export type ContentType =
				| 'mod'
				| 'plugin'
				| 'datapack'
				| 'resourcepack'
				| 'shader'
				| 'modpack'

			export type ResolutionPreferences = {
				game_versions?: string[]
				loaders?: string[]
			}

			export type ResolveContentRequest = {
				project_id: string
				version_id?: string | null
				content_type: ContentType
				selected?: ResolutionPreferences
				target?: ResolutionPreferences
				existing_project_ids?: string[]
			}

			export type ResolveContentPlan = {
				primary: ResolvedContent
				dependencies: ResolvedContent[]
				skipped: SkippedContent[]
			}

			export type ResolvedContent = {
				project_id: string
				version_id: string
				dependent_on_version_id?: string | null
			}

			export type SkippedContent = {
				project_id: string
				version_id?: string | null
				dependent_on_version_id?: string | null
				reason:
					| 'already_installed'
					| 'duplicate_project'
					| 'conflicting_dependency'
					| 'no_compatible_version'
					| 'missing_version'
					| 'quilt_fabric_api'
			}
		}
	}

	export namespace Campaign {
		export namespace Internal {
			export type CampaignInfo = {
				total_donations_usd: RawDecimal
				target_usd: RawDecimal
				num_donators: number
			}
		}
	}

	export namespace Billing {
		export namespace Internal {
			export type PriceDuration = 'five-days' | 'monthly' | 'quarterly' | 'yearly'

			export type SubscriptionStatus = 'provisioned' | 'unprovisioned'

			export type UserSubscription = {
				id: string
				user_id: string
				price_id: string
				interval: PriceDuration
				status: SubscriptionStatus
				created: string
				metadata?: SubscriptionMetadata
				next_charge_tax_amount?: number | null
			}

			export type SubscriptionMetadata =
				| { type: 'pyro'; id: string; region?: string }
				| { type: 'medal'; id: string }

			export type ChargeStatus =
				| 'open'
				| 'processing'
				| 'succeeded'
				| 'failed'
				| 'cancelled'
				| 'expiring'

			export type ChargeType = 'one-time' | 'subscription' | 'proration' | 'refund'

			export type PaymentPlatform = 'Stripe' | 'None'

			export type Charge = {
				id: string
				user_id: string
				price_id: string
				amount: number
				currency_code: string
				status: ChargeStatus
				due: string
				last_attempt: string | null
				type: ChargeType
				subscription_id: string | null
				subscription_interval: PriceDuration | null
				platform: PaymentPlatform
				parent_charge_id: string | null
				net: number | null
			}

			export type ProductMetadata =
				| { type: 'midas' }
				| {
						type: 'pyro'
						cpu: number
						ram: number
						swap: number
						storage: number
				  }
				| {
						type: 'medal'
						cpu: number
						ram: number
						swap: number
						storage: number
						region: string
				  }

			export type Price =
				| { type: 'one-time'; price: number }
				| { type: 'recurring'; intervals: Record<PriceDuration, number> }

			export type ProductPrice = {
				id: string
				product_id: string
				prices: Price
				currency_code: string
			}

			export type Product = {
				id: string
				metadata: ProductMetadata
				prices: ProductPrice[]
				unitary: boolean
			}

			export type EditSubscriptionRequest = {
				interval?: PriceDuration
				payment_method?: string
				cancelled?: boolean
				region?: string
				product?: string
			}

			export type EditSubscriptionResponse = {
				payment_intent_id: string
				client_secret: string
				tax: number
				total: number
			}

			export type AddPaymentMethodFlowResponse = {
				client_secret: string
			}

			export type EditPaymentMethodRequest = {
				primary: boolean
			}

			export type InitiatePaymentRequest = {
				type: 'payment_method' | 'confirmation_token'
				id?: string
				token?: string
				charge:
					| { type: 'existing'; id: string }
					| { type: 'new'; product_id: string; interval?: PriceDuration }
				existing_payment_intent?: string
				metadata?: {
					type: 'pyro'
					server_name?: string
					server_region?: string
					source: unknown
				}
			}

			export type InitiatePaymentResponse = {
				payment_intent_id?: string
				client_secret?: string
				price_id: string
				tax: number
				total: number
				payment_method?: string
			}

			export type RefundChargeRequest = {
				type: 'full' | 'partial' | 'none'
				amount?: number
				unprovision?: boolean
			}

			export type CreditRequest =
				| { subscription_ids: string[]; days: number; send_email: boolean; message: string }
				| { nodes: string[]; days: number; send_email: boolean; message: string }
				| { region: string; days: number; send_email: boolean; message: string }
		}
	}

	export namespace Payout {
		export namespace v3 {
			export type PayoutBalance = {
				available: number
				withdrawn_lifetime: number
				withdrawn_ytd: number
				pending: number
				dates: Record<string, number>
				requested_form_type: string | null
				form_completion_status: string | null
			}

			export type PayoutStatus =
				| 'success'
				| 'in-transit'
				| 'cancelled'
				| 'cancelling'
				| 'failed'
				| 'unknown'

			export type PayoutMethodType = 'venmo' | 'paypal' | 'tremendous' | 'muralpay'

			export type PayoutSource = 'creator_rewards' | 'affilites'

			export type TransactionItem =
				| {
						type: 'withdrawal'
						id: string
						status: PayoutStatus
						created: string
						amount: number
						fee: number | null
						method_type: PayoutMethodType | null
						method_id: string | null
						method_address: string | null
				  }
				| {
						type: 'payout_available'
						created: string
						payout_source: PayoutSource
						amount: number
				  }

			export type WithdrawalFees = {
				net_usd: number
				fee: number
				exchange_rate: number | null
			}

			export type PayoutDecimal = number

			export type PayoutInterval = {
				standard?: { min: number; max: number }
				fixed?: { values: PayoutDecimal[] }
			}

			export type PayoutMethod = {
				id: string
				type: PayoutMethodType
				name: string
				category: string | null
				image_url: string | null
				image_logo_url: string | null
				interval: PayoutInterval
				currency_code: string | null
				exchange_rate: number | null
			}
		}
	}

	export namespace Affiliate {
		export namespace Internal {
			export type AffiliateCode = {
				id: string
				created_at: string | null
				created_by: string | null
				affiliate: string
				source_name: string
			}

			export type CreateRequest = {
				affiliate?: string
				source_name: string
			}

			export type PatchRequest = {
				source_name: string
			}
		}
	}

	export namespace Attribution {
		export namespace Internal {
			export type AttributionPermissionKind =
				| 'license'
				| 'my_project'
				| 'special_permissions'
				| 'globally_allowed'
				| 'no_permission'
			export type AttributionResolutionKind = AttributionPermissionKind

			export type AttributionLicense = string | { name: string }

			export type AttributionModerationStatusKind = 'not_allowed' | 'approved' | 'bad_proof'

			export type AttributionModerationStatus = {
				kind: AttributionModerationStatusKind
				reason: string
				moderated_at?: string
				moderated_by?: string
			}

			export type AttributionResolutionBase = {
				notes: string
				image_urls: string[]
				moderation_status?: AttributionModerationStatus | null
				updated_by_moderator: boolean
			}

			export type AttributionResolution =
				| (AttributionResolutionBase & {
						kind: 'license'
						license: AttributionLicense
						link_to_work: string
				  })
				| (AttributionResolutionBase & {
						kind: 'my_project'
						license: AttributionLicense
				  })
				| (AttributionResolutionBase & {
						kind: 'special_permissions'
						link_to_work: string
				  })
				| (AttributionResolutionBase & {
						kind: 'globally_allowed'
						link_to_work: string
				  })
				| (AttributionResolutionBase & {
						kind: 'no_permission'
						link_to_work?: string
				  })

			export type FlameProject = {
				id: number
				title: string
				url: string
				icon_url: string
			}

			export type AttributionFile = {
				name: string
				sha1: string
				versions: string[]
				moderation_external_license_id?: number
				moderation_external_license?: ExternalProjects.Internal.ExternalProject
			}

			export type AttributionVersionInfo = {
				id: string
				name: string
				version_number: string
				date_created: string
			}

			export type OverrideFileOnPlatform = {
				file_path: string
				sha1: string
				version_id: string
				platform_version_id: string
				platform_project_id: string
			}

			export type AttributionGroup = {
				id: string
				flame_project: FlameProject | null
				attribution: AttributionResolution | null
				attributed_at: string | null
				attributed_by: string | null
				files: AttributionFile[]
				versions: AttributionVersionInfo[]
				override_files_on_platform: OverrideFileOnPlatform[]
			}

			export type UpdateGroupRequest = {
				attribution: AttributionResolution
			}

			export type DeleteGroupsRequest = {
				groups: string[]
			}

			export type DeleteAllGroupsRequest = {
				project_id: string
			}

			export type AssignRequest = {
				sha1: string
				target_group_id: number
				project_id: string
			}

			export type SplitRequest = {
				sha1: string
				project_id: string
			}

			export type FileScanResponse = {
				new_attribution_groups: number
				new_attribution_files: number
				scanned_file_names: string[]
			}
		}
	}

	export namespace Images {
		export namespace v3 {
			/** Extensions accepted by POST /v3/image (Labrinth image pipeline). */
			export type ImageExtension = 'bmp' | 'gif' | 'jpeg' | 'jpg' | 'png' | 'webp'

			/** `context` query values accepted by POST /v3/image. */
			export type ImageUploadContext = 'project' | 'version' | 'thread_message' | 'report'

			export type UploadedImage = {
				id: string
				url: string
				size: number
				created: string
				owner_id: string
			} & (
				| { context: 'project'; project_id: string }
				| { context: 'version'; version_id: string }
				| { context: 'thread_message'; thread_message_id: string }
				| { context: 'report'; report_id: string | null }
			)

			export type UploadedImageFor<C extends ImageUploadContext> = Extract<
				UploadedImage,
				{ context: C }
			>

			/**
			 * Target for POST /v3/image (per-context id query params, plus `context`).
			 * `ext` is passed as a separate argument on the client module.
			 */
			export type UploadImageParams =
				| { context: 'project'; project_id: string }
				| { context: 'version'; version_id: string }
				| { context: 'thread_message'; thread_message_id: string }
				| { context: 'report'; report_id?: string }
		}
	}

	export namespace Analytics {
		export namespace Internal {
			export type AnalyticsEventUpsert = {
				announcement_url: string | null
				for_metric_kind: v3.AnalyticsEventMetricKind[] | null
				title: string
				ends: string
				starts: string
			}
		}

		export namespace v3 {
			export type AnalyticsEventId = number
			export type AnalyticsEventMetricKind = 'views' | 'revenue' | 'downloads' | 'playtime'

			export type AnalyticsEvent = {
				announcement_url: string | null
				for_metric_kind: AnalyticsEventMetricKind[] | null
				title: string
				ends: string
				id: AnalyticsEventId
				starts: string
			}

			export type FetchRequest = {
				time_range: TimeRange
				return_metrics: ReturnMetrics
				project_ids?: string[]
			}

			export type TimeRange = {
				start: string
				end: string
				resolution: TimeRangeResolution
			}

			export type TimeRangeResolution = { slices: number } | { minutes: number }

			export type ReturnMetrics = {
				project_views?: Metrics<ProjectViewsField, ProjectViewsFilters>
				project_downloads?: Metrics<ProjectDownloadsField, ProjectDownloadsFilters>
				project_playtime?: Metrics<ProjectPlaytimeField, ProjectPlaytimeFilters>
				project_revenue?: Metrics<ProjectRevenueField, ProjectRevenueFilters>
				affiliate_code_clicks?: Metrics<AffiliateCodeClicksField, AffiliateCodeClicksFilters>
				affiliate_code_conversions?: Metrics<
					AffiliateCodeConversionsField,
					AffiliateCodeConversionsFilters
				>
				affiliate_code_revenue?: Metrics<AffiliateCodeRevenueField, AffiliateCodeRevenueFilters>
			}

			export type Metrics<BucketBy, FilterBy> = {
				bucket_by?: BucketBy[]
				filter_by?: FilterBy
			}

			export type ProjectViewsField =
				| 'project_id'
				| 'domain'
				| 'site_path'
				| 'monetized'
				| 'country'

			export type ProjectDownloadsField =
				| 'project_id'
				| 'version_id'
				| 'dependent_project_id'
				| 'user_agent'
				| 'domain'
				| 'country'
				| 'monetized'
				| 'reason'
				| 'game_version'
				| 'loader'

			export type ProjectPlaytimeField =
				| 'project_id'
				| 'version_id'
				| 'loader'
				| 'game_version'
				| 'country'

			export type ProjectRevenueField = 'project_id' | 'user_id'

			export type DownloadReason = 'standalone' | 'dependency' | 'modpack' | 'update'

			export type AffiliateCodeClicksField = 'affiliate_code_id'

			export type AffiliateCodeConversionsField = 'affiliate_code_id'

			export type AffiliateCodeRevenueField = 'affiliate_code_id'

			export type ProjectViewsFilters = {
				domain?: string[]
				site_path?: string[]
				monetized?: boolean[]
				country?: string[]
			}

			export type ProjectDownloadsFilters = {
				version_id?: string[]
				dependent_project_id?: string[]
				domain?: string[]
				user_agent?: string[]
				monetized?: boolean[]
				country?: string[]
				reason?: DownloadReason[]
				game_version?: string[]
				loader?: string[]
			}

			export type ProjectPlaytimeFilters = {
				version_id?: string[]
				loader?: string[]
				game_version?: string[]
				country?: string[]
			}

			export type ProjectRevenueFilters = {
				user_id?: string[]
			}

			export type AffiliateCodeClicksFilters = {
				affiliate_code_id?: string[]
			}

			export type AffiliateCodeConversionsFilters = {
				affiliate_code_id?: string[]
			}

			export type AffiliateCodeRevenueFilters = {
				affiliate_code_id?: string[]
			}

			export type FetchResponse = {
				metrics: TimeSlice[]
				projects: Record<string, Projects.v3.Project>
				users: Record<string, Users.v3.User>
				project_events: ProjectAnalyticsEvent[]
			}

			export type FacetsResponse = {
				facets: AnalyticsFacets
			}

			export type AnalyticsFacets = {
				project_views?: Partial<ProjectViewsFacets>
				project_downloads?: Partial<ProjectDownloadsFacets>
				project_playtime?: Partial<ProjectPlaytimeFacets>
			}

			export type ProjectViewsFacets = {
				domain: string[]
				site_path: string[]
				monetized: boolean[]
				country: string[]
			}

			export type ProjectDownloadsFacets = {
				project_id: string[]
				domain: string[]
				user_agent: string[]
				version_id: string[]
				monetized: boolean[]
				country: string[]
				reason: DownloadReason[]
				game_version: string[]
				loader: string[]
			}

			export type ProjectPlaytimeFacets = {
				version_id: string[]
				loader: string[]
				game_version: string[]
				country: string[]
			}

			export type TimeSlice = AnalyticsData[]

			export type ProjectAnalyticsEvent = {
				project_id: string
				timestamp: string
			} & ProjectAnalyticsEventKind

			export type ProjectAnalyticsEventKind =
				| {
						kind: 'version_uploaded'
						version_id: string
						version_name: string
						version_number: string
				  }
				| {
						kind: 'status_changed'
						status_from: Projects.v2.ProjectStatus
						status_to: Projects.v2.ProjectStatus
				  }

			export type AnalyticsData = ProjectAnalytics | AffiliateCodeAnalytics

			export type ProjectAnalytics = {
				source_project: string
			} & ProjectMetrics

			export type ProjectMetrics =
				| ({ metric_kind: 'views' } & ProjectViews)
				| ({ metric_kind: 'downloads' } & ProjectDownloads)
				| ({ metric_kind: 'playtime' } & ProjectPlaytime)
				| ({ metric_kind: 'revenue' } & ProjectRevenue)

			export type ProjectViews = {
				domain?: string
				site_path?: string
				monetized?: boolean
				country?: string
				views: number
			}

			export type ProjectDownloads = {
				user_agent?: string
				domain?: string
				version_id?: string
				dependent_project_id?: string
				country?: string
				monetized?: boolean
				reason?: DownloadReason
				game_version?: string
				loader?: string
				downloads: number
			}

			export type ProjectPlaytime = {
				version_id?: string
				loader?: string
				game_version?: string
				country?: string
				seconds: number
			}

			export type ProjectRevenue = {
				user_id?: string
				revenue: string
			}

			export type AffiliateCodeAnalytics = {
				source_affiliate_code: string
			} & AffiliateCodeMetrics

			export type AffiliateCodeMetrics =
				| ({ metric_kind: 'clicks' } & AffiliateCodeClicks)
				| ({ metric_kind: 'conversions' } & AffiliateCodeConversions)
				| ({ metric_kind: 'revenue' } & AffiliateCodeRevenue)

			export type AffiliateCodeClicks = {
				clicks: number
			}

			export type AffiliateCodeConversions = {
				conversions: number
			}

			export type AffiliateCodeRevenue = {
				revenue: string
			}
		}
	}

	export namespace Auth {
		export namespace Internal {
			export type SubscriptionStatus = {
				subscribed: boolean
			}

			export type DiscordCommunityLinkResponse = {
				url: string
			}
		}

		export namespace v2 {
			export type LoginRequest = {
				username: string
				password: string
				challenge: string
			}

			export type LoginResponse = {
				session?: string
				flow?: string
			}

			export type Login2FARequest = {
				code: string
				flow: string
			}

			export type Login2FAResponse = {
				session: string
			}

			export type CreateAccountRequest = {
				username: string
				password: string
				email: string
				challenge: string
				sign_up_newsletter?: boolean
				account_consent?: boolean
			}

			export type CreateAccountResponse = {
				session: string
			}

			export type ValidateCreateAccountRequest = {
				username: string
				password: string
				email: string
			}

			export type CreateOAuthAccountRequest = {
				username: string
				state: string
				challenge: string
				sign_up_newsletter: boolean
				account_consent?: boolean
			}

			export type CreateOAuthAccountResponse = {
				session: string
			}

			export type ResetPasswordRequest = {
				username: string
				challenge: string
			}

			export type ChangePasswordRequest = {
				flow?: string
				old_password?: string
				new_password?: string
			}

			export type Passkey = {
				id: string
				name: string
				created_at: string
				last_used: string | null
			}

			export type PasskeyRegisterStartResponse = {
				options: Record<string, unknown>
				flow: string
			}

			export type PasskeyRegisterFinishRequest = {
				flow: string
				name: string
				credential: unknown
			}

			export type PasskeyAuthenticateStartResponse = {
				options: Record<string, unknown>
				flow: string
			}

			export type PasskeyAuthenticateFinishRequest = {
				flow: string
				credential: unknown
			}

			export type PasskeyRenameRequest = {
				name: string
			}
		}
	}

	export namespace Globals {
		export namespace Internal {
			export type Globals = {
				tax_compliance_thresholds: Record<string, number>
				captcha_enabled: boolean
			}
		}
	}

	export namespace OAuth {
		export namespace Internal {
			export type OAuthClientAccessRequest = {
				flow_id: string
				client_id: string
				client_name: string
				client_icon: string | null
				requested_scopes: number
			}

			export type AcceptRejectRequest = {
				flow: string
			}

			export type OAuthRedirectUri = {
				id: string
				client_id: string
				uri: string
			}

			export type OAuthClient = {
				id: string
				name: string
				icon_url: string | null
				raw_icon_url: string | null
				max_scopes: number
				redirect_uris: OAuthRedirectUri[]
				created_by: string
				created: string
				url: string | null
				description: string | null
			}

			export type OAuthClientCreationResult = OAuthClient & {
				client_secret: string
			}

			export type OAuthClientAuthorization = {
				id: string
				app_id: string
				user_id: string
				scopes: number
				created: string
			}

			export type CreateOAuthAppRequest = {
				name: string
				max_scopes: number
				redirect_uris: string[]
				url?: string
				description?: string
			}

			export type EditOAuthAppRequest = {
				name?: string
				max_scopes?: number
				redirect_uris?: string[]
				url?: string | null
				description?: string | null
				icon_url?: string
			}
		}
	}

	export namespace Projects {
		export namespace v2 {
			export type Environment = 'required' | 'optional' | 'unsupported' | 'unknown'

			export type ProjectStatus =
				| 'approved'
				| 'archived'
				| 'rejected'
				| 'draft'
				| 'unlisted'
				| 'processing'
				| 'withheld'
				| 'scheduled'
				| 'private'
				| 'unknown'

			export type MonetizationStatus = 'monetized' | 'demonetized' | 'force-demonetized'

			export type ProjectType =
				| 'mod'
				| 'modpack'
				| 'resourcepack'
				| 'shader'
				| 'plugin'
				| 'datapack'
				| 'project'

			export type GalleryImage = {
				url: string
				featured: boolean
				title?: string
				description?: string
				created: string
				ordering: number
			}

			export type DonationLink = {
				id: string
				platform: string
				url: string
			}

			export interface CreateProjectBase {
				title: string
				project_type: 'mod'
				slug: string
				description: string
				body: string
				requested_status: v2.ProjectStatus
				initial_versions: unknown[]
				team_members: unknown[]
				categories: string[]
				client_side: string
				server_side: string
				license_id: string
				is_draft: boolean
				organization_id?: string
			}

			export type Project = {
				id: string
				slug: string
				project_type: ProjectType
				actualProjectType: ProjectType
				team: string
				organization: string | null
				title: string
				description: string
				body: string
				published: string
				updated: string
				approved?: string
				queued?: string
				status: ProjectStatus
				requested_status?: ProjectStatus
				moderator_message?: {
					message: string
					body?: string
				}
				license: {
					id: string
					name: string
					url?: string
				}
				client_side: Environment
				server_side: Environment
				downloads: number
				followers: number
				categories: string[]
				additional_categories: string[]
				game_versions: string[]
				loaders: string[]
				versions: string[]
				icon_url?: string
				raw_icon_url?: string
				issues_url?: string
				source_url?: string
				wiki_url?: string
				discord_url?: string
				donation_urls?: DonationLink[]
				gallery?: GalleryImage[]
				color?: number
				thread_id: string
				monetization_status: MonetizationStatus
			}

			export type ProjectCheckResponse = {
				id: string
			}

			export type SearchResultHit = {
				project_id: string
				project_type: ProjectType
				slug: string
				author: string
				title: string
				description: string
				categories: string[]
				display_categories: string[]
				versions: string[]
				downloads: number
				follows: number
				icon_url: string
				date_created: string
				date_modified: string
				latest_version?: string
				license: string
				client_side: Environment
				server_side: Environment
				gallery: string[]
				color?: number
			}

			export type SearchResult = {
				hits: SearchResultHit[]
				offset: number
				limit: number
				total_hits: number
			}

			export type ProjectSearchParams = {
				query?: string
				facets?: string[][] // in the format of [["categories:forge"],["versions:1.17.1"]]
				new_filters?: string
				filters?: string
				index?: 'relevance' | 'downloads' | 'follows' | 'newest' | 'updated'
				offset?: number
				limit?: number
			}

			export interface DependencyInfo {
				projects: Project[]
				versions: Labrinth.Versions.v2.Version[]
			}

			export type BulkEditProjectRequest = {
				categories?: string[]
				add_categories?: string[]
				remove_categories?: string[]
				additional_categories?: string[]
				add_additional_categories?: string[]
				remove_additional_categories?: string[]
				donation_urls?: DonationLink[]
				add_donation_urls?: DonationLink[]
				remove_donation_urls?: DonationLink[]
				issues_url?: string | null
				source_url?: string | null
				wiki_url?: string | null
				discord_url?: string | null
			}
		}

		export namespace v3 {
			export type ProjectType =
				| 'mod'
				| 'modpack'
				| 'resourcepack'
				| 'shader'
				| 'plugin'
				| 'datapack'

			export type Environment =
				| 'client_and_server'
				| 'client_only'
				| 'client_only_server_optional'
				| 'singleplayer_only'
				| 'server_only'
				| 'server_only_client_optional'
				| 'dedicated_server_only'
				| 'client_or_server'
				| 'client_or_server_prefers_both'
				| 'unknown'

			export type GalleryItem = {
				url: string
				raw_url: string
				featured: boolean
				name?: string
				description?: string
				created: string
				ordering: number
			}

			export type Link = {
				platform: string
				donation: boolean
				url: string
			}

			export type NormalizedProjectNagKind =
				| 'minecraft-title-clause'
				| 'project-name-non-standard-text'
				| 'project-name-profanity'
				| 'project-name-slur'
				| 'project-name-version'
				| 'project-summary-links'
				| 'project-summary-matches-title'
				| 'project-summary-non-english'
				| 'project-summary-non-standard-text'
				| 'project-summary-profanity'
				| 'project-summary-slur'
				| 'project-summary-spam'
				| 'summary-special-formatting'
				| 'summary-too-short'
				| 'add-icon'
				| 'feature-gallery-image'
				| 'upload-gallery-image'
				| 'gallery-text-non-standard'
				| 'gallery-text-profanity'
				| 'gallery-text-slur'
				| 'add-description'
				| 'adjacent-headers'
				| 'description-ends-with-header'
				| 'description-too-short'
				| 'long-headers'
				| 'missing-alt-text'
				| 'project-description-banned-link'
				| 'project-description-matches-summary'
				| 'project-description-non-english'
				| 'project-description-non-standard-text'
				| 'project-description-profanity'
				| 'project-description-slur'
				| 'project-description-spam'
				| 'add-custom-license-details'
				| 'invalid-license-url'
				| 'select-license'
				| 'add-links'
				| 'add-links-server'
				| 'banned-link-usage'
				| 'gpl-license-source-required'
				| 'identical-links'
				| 'misused-discord-link'
				| 'verify-external-links'
				| 'review-permissions'
				| 'add-java-address'
				| 'all-languages'
				| 'select-compatibility'
				| 'select-country'
				| 'select-language'
				| 'too-many-languages'
				| 'all-tags-selected'
				| 'multiple-resolution-tags'
				| 'select-tags'
				| 'too-many-tags'
				| 'too-many-tags-server'
				| 'select-environment'
				| 'upload-version'
				| 'check-disclosures'
				| 'disclosures-special-formatting'
				| 'moderator-feedback'

			type ReplaceHyphensWithUnderscores<T extends string> = T extends `${infer Head}-${infer Tail}`
				? `${Head}_${ReplaceHyphensWithUnderscores<Tail>}`
				: T

			export type ProjectNagKind = ReplaceHyphensWithUnderscores<NormalizedProjectNagKind>

			export type ProjectNagSeverity = 'required' | 'warning' | 'suggestion'

			export type ProjectNag = {
				kind: ProjectNagKind
				severity: ProjectNagSeverity
				details: Record<string, unknown>
			}

			export type ProjectValidationResponse = {
				nags: ProjectNag[]
			}

			export type Project = {
				id: string
				slug?: string
				project_types: ProjectType[]
				games: string[]
				team_id: string
				organization?: string
				name: string
				summary: string
				description: string
				published: string
				updated: string
				approved?: string
				queued?: string
				status: v2.ProjectStatus
				requested_status?: v2.ProjectStatus
				license: {
					id: string
					name: string
					url?: string
				}
				downloads: number
				followers: number
				categories: string[]
				additional_categories: string[]
				loaders: string[]
				mrpack_loaders: string[]
				versions: string[]
				icon_url?: string
				raw_icon_url?: string
				link_urls: Record<string, Link>
				gallery: GalleryItem[]
				color?: number
				thread_id: string
				monetization_status: v2.MonetizationStatus
				side_types_migration_review_status: 'reviewed' | 'pending'
				environment?: Environment[]

				minecraft_server?: MinecraftServer | null
				minecraft_java_server?: MinecraftJavaServer | null
				minecraft_bedrock_server?: MinecraftBedrockServer | null
				minecraft_mod?: unknown | null

				/**
				 * @deprecated Not recommended to use.
				 **/
				[key: string]: unknown
			}

			interface CreateProjectBase {
				name: string // 3-64 chars
				slug: string // 3-64 chars, URL-safe
				summary: string // 3-255 chars
				description: string // max 65536 chars, markdown
				requested_status: v2.ProjectStatus
				organization_id?: string // automatically transfer the project to this organization
			}

			export interface MinecraftJavaServerPing {
				address: string
				data?: {
					description: string
					latency: {
						nanos: number
						secs: number
					}
					players_max: number
					players_online: number
					version_name: string
					version_protocol: number
				}
				port: number
				when: string
			}

			export interface MinecraftServer {
				max_players?: number
				region?: string
				active_version?: string | null
				languages?: string[]
				/**
				 * deprecated, use region instead
				 */
				country?: string
			}

			export interface ModpackContent {
				kind: 'modpack'
				version_id: string
				project_id?: string
				project_name?: string
				project_icon?: string
			}
			export interface VanillaContent {
				kind: 'vanilla'
				supported_game_versions: string[]
				recommended_game_version?: string
			}

			export interface MinecraftJavaServer {
				address?: string
				content?: ModpackContent | VanillaContent
				verified_plays_4w?: number | null
				verified_plays_2w?: number | null
				ping: Projects.v3.MinecraftJavaServerPing | null
			}

			export interface MinecraftBedrockServer {
				address?: string
			}

			export interface CreateServerProjectRequest {
				base: CreateProjectBase
				minecraft_server?: MinecraftServer
				minecraft_java_server?: Omit<MinecraftJavaServer, 'ping'>
				minecraft_bedrock_server?: MinecraftBedrockServer
			}

			export type EditProjectRequest = {
				name?: string
				summary?: string
				description?: string
				categories?: string[]
				additional_categories?: string[]
				license_url?: string | null
				link_urls?: Record<string, string | null>
				license_id?: string
				slug?: string
				status?: v2.ProjectStatus
				requested_status?: v2.ProjectStatus | null
				moderation_message?: string | null
				moderation_message_body?: string | null
				monetization_status?: v2.MonetizationStatus
				side_types_migration_review_status?: 'reviewed' | 'pending'
				environment?: Environment

				minecraft_server?: MinecraftServer
				minecraft_java_server?: MinecraftJavaServer
				minecraft_bedrock_server?: MinecraftBedrockServer
				[key: string]: unknown
			}

			export type Organization = {
				id: string
				slug: string
				name: string
				team_id: string
				description: string
				icon_url: string | null
				raw_icon_url: string | null
				color: number | null
				members: TeamMember[]
			}

			export type OrganizationMember = {
				team_id: string
				user: Users.v3.User
				role: string
				is_owner: boolean
				permissions: number
				organization_permissions: number
				accepted: boolean
				payouts_split: number
				ordering: number
			}

			export type TeamMember = {
				team_id: string
				user: Users.v3.User
				role: string
				is_owner: boolean
				permissions: number | null
				organization_permissions: number | null
				accepted: boolean
				payouts_split: number | null
				ordering: number
			}

			export type Team = {
				id: string
				members: TeamMember[]
			}

			export type ProjectDependencies = {
				projects: Project[]
				versions: Labrinth.Versions.v3.Version[]
			}

			export type TelemetryConsent = 'opt_in' | 'opt_out' | 'always_active'

			export type AiUsage = 'code' | 'assets' | 'text'

			export type DisclosureLockStatus = 'unlocked' | 'cannot_disable' | 'fully_locked'

			export type DerivativeSource = {
				label: string
				link?: string | null
				note?: string | null
			}

			export type ProjectDisclosure =
				| {
						type: 'ai_content'
						uses: AiUsage[]
						note?: string | null
				  }
				| {
						type: 'ai_functionality'
						note?: string | null
				  }
				| {
						type: 'advertisements'
						note?: string | null
				  }
				| {
						type: 'epilepsy_triggers'
						note?: string | null
				  }
				| {
						type: 'system_interactions'
						interactions: string[]
						note?: string | null
				  }
				| {
						type: 'telemetry'
						consent: TelemetryConsent
						data_collected: string[]
				  }
				| {
						type: 'derivative_work'
						sources: DerivativeSource[]
				  }
				| {
						type: 'paid_features'
						features: string[]
				  }
				| {
						type: 'archived'
						note?: string | null
				  }

			export type ProjectDisclosureData = ProjectDisclosure & {
				set_by_moderator?: boolean | null
				lock_status?: DisclosureLockStatus | null
				updated_at: string
				updated_by?: string | null
				deleted_at?: string | null
			}

			export type ProjectDisclosureType = ProjectDisclosure['type']

			export type ProjectDisclosureOf<T extends ProjectDisclosureType> = Extract<
				ProjectDisclosureData,
				{ type: T }
			>

			export type GetProjectDisclosures = {
				disclosures: ProjectDisclosureData[]
			}

			export type ModifyProjectDisclosures = {
				set: ProjectDisclosure[]
				remove: ProjectDisclosureType[]
				lock_status?: DisclosureLockStatus | null
			}
		}
	}

	export namespace Organizations {
		export namespace v3 {
			export type Organization = {
				id: string
				slug: string
				name: string
				team_id: string
				description: string
				icon_url: string | null
				raw_icon_url: string | null
				color: number | null
				members: Projects.v3.TeamMember[]
				moderation_notes?: Users.Common.ModerationNote | null
			}

			export type CreateOrganizationRequest = {
				slug: string
				name: string
				description: string
			}

			export type EditOrganizationRequest = {
				description?: string
				slug?: string
				name?: string
			}

			export type AddProjectRequest = {
				project_id: string
			}

			export type RemoveProjectRequest = {
				new_owner: string
			}
		}
	}

	export namespace Versions {
		export namespace v2 {
			export type VersionType = 'release' | 'beta' | 'alpha'

			export type VersionStatus =
				| 'listed'
				| 'archived'
				| 'draft'
				| 'unlisted'
				| 'scheduled'
				| 'unknown'

			export type DependencyType = 'required' | 'optional' | 'incompatible' | 'embedded'

			export type FileType = 'required-resource-pack' | 'optional-resource-pack' | 'unknown'

			export type VersionFileHash = {
				sha512: string
				sha1: string
			}

			export type VersionFile = {
				id?: string
				hashes: VersionFileHash
				url: string
				filename: string
				primary: boolean
				size: number
				file_type?: FileType
			}

			export type Dependency = {
				file_name?: string
				dependency_type: DependencyType
			} & (
				| {
						project_id: string
				  }
				| {
						version_id: string
						project_id?: string
				  }
			)

			export type Version = {
				id: string
				project_id: string
				author_id: string
				featured: boolean
				name: string
				version_number: string
				changelog: string
				changelog_url?: string | null
				date_published: string
				downloads: number
				version_type: VersionType
				status: VersionStatus
				requested_status?: VersionStatus | null
				files: VersionFile[]
				dependencies: Dependency[]
				game_versions: string[]
				loaders: string[]
			}

			export interface GetProjectVersionsParams {
				game_versions?: string[]
				loaders?: string[]
				include_changelog?: boolean
				limit?: number
				offset?: number
			}
		}

		// TODO: consolidate duplicated types between v2 and v3 versions
		export namespace v3 {
			export type FlameProject = {
				id: number
				title: string
				url: string
				icon_url: string
			}

			export type DependencyAttribution = {
				flame_project?: FlameProject
				resolution?: DependencyAttributionResolution
			}

			export type DependencyAttributionResolution =
				| {
						kind: 'license'
						license: Labrinth.Attribution.Internal.AttributionLicense
						link_to_work: string
				  }
				| {
						kind: 'globally_allowed'
						link_to_work: string
				  }
				| {
						kind: 'my_project'
						license: Labrinth.Attribution.Internal.AttributionLicense
				  }
				| {
						kind: 'special_permissions'
						link_to_work: string
				  }
				| {
						kind: 'no_permission'
						link_to_work?: string
				  }

			export interface Dependency {
				dependency_type: Labrinth.Versions.v2.DependencyType
				project_id?: string
				file_name?: string
				version_id?: string
				attribution?: DependencyAttribution
			}

			export interface GetProjectVersionsParams {
				game_versions?: string[]
				loaders?: string[]
				include_changelog?: boolean
				limit?: number
				offset?: number
				apiVersion?: 2 | 3
			}

			export type VersionChannel = 'release' | 'beta' | 'alpha'

			export type FileType =
				| 'required-resource-pack'
				| 'optional-resource-pack'
				| 'sources-jar'
				| 'dev-jar'
				| 'javadoc-jar'
				| 'signature'
				| 'unknown'

			export type FileHashType = 'sha512' | 'sha1'
			export type VersionFileHash = {
				[key in FileHashType]: string
			}

			export interface VersionFile {
				id?: string
				hashes: VersionFileHash
				url: string
				filename: string
				primary: boolean
				size: number
				file_type?: FileType
			}

			interface JavaServerVersion {
				/**
				 * The version id of the modpack
				 */
				modpack: string
			}

			export interface Version {
				name: string
				version_number: string
				changelog?: string
				dependencies: Dependency[]
				game_versions: string[]
				version_type: VersionChannel
				loaders: string[]
				featured: boolean
				status: Labrinth.Versions.v2.VersionStatus
				id: string
				project_id: string
				author_id: string
				date_published: string
				downloads: number
				files: VersionFile[]
				files_missing_attribution?: string[]
				environment?: Labrinth.Projects.v3.Environment
				mrpack_loaders?: string[]

				minecraft_java_server?: JavaServerVersion
			}

			export interface DraftVersionFile {
				fileType?: FileType
				file: File
			}

			export type DraftVersion = Omit<
				Labrinth.Versions.v3.CreateVersionRequest,
				'file_parts' | 'primary_file' | 'file_types'
			> & {
				existing_files?: VersionFile[]
				version_id?: string
				environment?: Labrinth.Projects.v3.Environment
			}

			export interface CreateVersionRequest {
				name: string
				version_number: string
				changelog: string
				dependencies?: Array<{
					version_id?: string
					project_id?: string
					file_name?: string
					dependency_type: Labrinth.Versions.v2.DependencyType
				}>
				game_versions: string[]
				version_type: 'release' | 'beta' | 'alpha'
				loaders: string[]
				featured?: boolean
				status?: 'listed' | 'archived' | 'draft' | 'unlisted' | 'scheduled' | 'unknown'
				requested_status?: 'listed' | 'archived' | 'draft' | 'unlisted' | null
				project_id: string
				file_parts: string[]
				primary_file?: string
				file_types?: Record<string, Labrinth.Versions.v3.FileType | null>
				environment?: Labrinth.Projects.v3.Environment
				mrpack_loaders?: string[]
			}

			export type ModifyVersionRequest = Partial<
				Omit<CreateVersionRequest, 'project_id' | 'file_parts' | 'primary_file' | 'file_types'>
			> & {
				file_types?: {
					algorithm: string
					hash: string
					file_type: Labrinth.Versions.v3.FileType | null
				}[]
			}

			export type AddFilesToVersionRequest = {
				file_parts: string[]
				file_types?: Record<string, Labrinth.Versions.v3.FileType | null>
			}
		}
	}

	export namespace Users {
		export namespace Common {
			export type Role = 'developer' | 'moderator' | 'admin'

			export type AuthProvider =
				| 'github'
				| 'discord'
				| 'microsoft'
				| 'gitlab'
				| 'google'
				| 'steam'
				| 'paypal'

			export type UserPayoutData = {
				paypal_address?: string
				paypal_country?: string
				venmo_handle?: string
				balance: number
			}

			export type ModerationNote = {
				notes: string
				last_modified: string
				created_at: string
				last_author: string
				user_rating: number
				version: number
			}
		}

		export namespace v2 {
			export type Role = Common.Role
			export type AuthProvider = Common.AuthProvider
			export type UserPayoutData = Common.UserPayoutData

			export type User = {
				id: string
				username: string
				name?: string
				avatar_url?: string
				bio?: string
				created: string
				role: Role
				badges: number
				auth_providers?: AuthProvider[]
				email?: string
				email_verified?: boolean
				has_password?: boolean
				has_totp?: boolean
				payout_data?: UserPayoutData
				github_id?: number
			}
		}

		export namespace v3 {
			export type Role = Common.Role
			export type AuthProvider = Common.AuthProvider
			export type UserPayoutData = Common.UserPayoutData
			export type Theme = 'light' | 'dark' | 'oled' | 'retro'
			export type LayoutOption = 'grid' | 'rows'
			export type FriendPrivacy = 'none' | 'mutual' | 'everyone'
			export type InvitePrivacy = 'none' | 'friends' | 'everyone'

			export type AppearancePreferences = {
				auto: boolean
				theme: Theme
			}

			export type BehaviorPreferences = {
				minimize_app: boolean
				hide_right_sidebar: boolean
				show_jump_in: boolean
				compact_instance_cards: boolean
				show_play_time: boolean
				hide_nametag: boolean
				show_all_screenshots: boolean
				show_files_tab_in_instances: boolean
				show_worlds_tab_in_instances: boolean
				show_screenshots_tab_in_instances: boolean
				show_skin_selector_in_sidebar: boolean
				quick_instance_count: number
				warn_on_unknown_modpacks: boolean
				skip_non_essential_warnings: boolean
			}

			export type LocalizationPreferences = {
				locale: string
			}

			export type LayoutPreferences = {
				mods: LayoutOption
				plugins: LayoutOption
				datapacks: LayoutOption
				shaders: LayoutOption
				resourcepacks: LayoutOption
				modpacks: LayoutOption
				servers: LayoutOption
				users: LayoutOption
			}

			export type SidebarPreferences = {
				right_aligned_search: boolean
				left_aligned_content: boolean
			}

			export type SocialPreferences = {
				friend_privacy: FriendPrivacy
				shared_instances_privacy: InvitePrivacy
				hosting_access_privacy: InvitePrivacy
			}

			export type UserPreferences = {
				appearance: AppearancePreferences
				behavior: BehaviorPreferences
				localization: LocalizationPreferences
				layouts: LayoutPreferences
				sidebars: SidebarPreferences
				social: SocialPreferences
			}

			export type PartialUserPreferences = {
				appearance?: Partial<AppearancePreferences>
				behavior?: Partial<BehaviorPreferences>
				localization?: Partial<LocalizationPreferences>
				layouts?: Partial<LayoutPreferences>
				sidebars?: Partial<SidebarPreferences>
				social?: Partial<SocialPreferences>
			}

			export type Pride26CampaignDonation = {
				last_donated_at: string
				has_badge: boolean
				has_midas: boolean
			}

			export type UserCampaigns = {
				pride_26: Pride26CampaignDonation | null
			}

			export type User = {
				id: string
				username: string
				avatar_url?: string
				raw_avatar_url?: string
				bio?: string
				created: string
				role: Role
				badges: number
				campaigns: UserCampaigns
				auth_providers?: AuthProvider[]
				email?: string
				email_verified?: boolean
				has_password?: boolean
				has_totp?: boolean
				payout_data?: UserPayoutData
				stripe_customer_id?: string
				allow_friend_requests?: boolean
				moderation_notes?: Common.ModerationNote | null
				github_id?: number
				discord_id?: string
				steam_id?: string
			}

			export type SearchUser = {
				id: string
				username: string
				avatar_url: string | null
			}

			export type AllProjectsResponse = {
				projects: Projects.v3.Project[]
				organizations: Record<string, Organizations.v3.Organization>
			}
		}
	}

	export namespace Friends {
		export namespace v3 {
			export type UserFriend = {
				id: string
				friend_id: string
				accepted: boolean
				created: string
			}
		}
	}

	export namespace BlockedUsers {
		export namespace Internal {
			export type BlockStatus = {
				blocked: boolean
			}
		}

		export namespace v3 {
			export type BlockedUserId = string
		}
	}

	export namespace ServerPing {
		export namespace Internal {
			export type MinecraftJavaPingRequest = {
				address: string
				timeout_ms?: number
			}
		}
	}

	export namespace Tags {
		export namespace v2 {
			export interface Category {
				icon: string
				name: string
				project_type: string
				header: string
			}

			export interface Loader {
				icon: string
				name: string
				supported_project_types: string[]
			}

			export interface GameVersion {
				version: string
				version_type: string
				date: string
				major: boolean
			}

			export interface DonationPlatform {
				short: string
				name: string
			}

			export type LicenseText = {
				title: string
				body: string
			}
		}
	}

	export namespace Teams {
		export namespace v2 {
			export type AddTeamMemberRequest = {
				user_id: string
				role?: string
				permissions?: number
				organization_permissions?: number | null
				payouts_split?: number
				ordering?: number
			}

			export type EditTeamMemberRequest = {
				permissions?: number
				organization_permissions?: number | null
				role?: string
				payouts_split?: number
				ordering?: number
			}

			export type TransferOwnershipRequest = {
				user_id: string
			}
		}
	}

	export namespace Search {
		export type SearchParams = {
			query?: string
			offset?: string | number
			index?: string
			limit?: string | number
			new_filters?: string
			facets?: string[][]
			filters?: string
			version?: string
		}

		export namespace v2 {
			export interface ResultSearchProject {
				project_id: string
				project_type: string
				all_project_types: string[]
				slug: string | null
				author: string
				author_id: string | null
				organization: string | null
				organization_id: string | null
				title: string
				description: string
				categories: string[]
				display_categories: string[]
				versions: string[]
				downloads: number
				follows: number
				icon_url: string
				date_created: string
				date_modified: string
				latest_version: string
				license: string
				client_side: string
				server_side: string
				disclosure_types: string[]
				gallery: string[]
				featured_gallery: string | null
				color: number | null
			}

			export interface SearchResults {
				hits: ResultSearchProject[]
				offset: number
				limit: number
				total_hits: number
			}
		}

		export namespace v3 {
			export interface ResultSearchProject {
				version_id?: string
				project_id: string
				project_types: string[]
				all_project_types: string[]
				slug: string | null
				author: string
				author_id: string | null
				organization: string | null
				organization_id: string | null
				name: string
				summary: string
				categories: string[]
				display_categories: string[]
				downloads: number
				follows: number
				icon_url: string | null
				date_created: string
				date_modified: string
				license: string
				gallery: string[]
				featured_gallery: string | null
				color: number | null
				loaders: string[]
				project_loader_fields?: Record<string, unknown[]> & {
					environment?: Projects.v3.Environment[]
				}
				minecraft_server?: Projects.v3.MinecraftServer | null
				minecraft_java_server?: Projects.v3.MinecraftJavaServer | null
				minecraft_bedrock_server?: Projects.v3.MinecraftBedrockServer | null
				minecraft_mod?: unknown | null
				disclosure_types: string[]
			}

			export interface SearchResults {
				hits: ResultSearchProject[]
				page: number
				hits_per_page: number
				total_hits: number
			}
		}
	}

	export namespace Threads {
		export namespace v3 {
			export type ThreadType = 'report' | 'project' | 'direct_message'

			export type MessageBody =
				| {
						type: 'text'
						body: string
						private?: boolean
						replying_to?: string
						associated_images?: string[]
				  }
				| {
						type: 'status_change'
						new_status: Projects.v2.ProjectStatus
						old_status: Projects.v2.ProjectStatus
				  }
				| {
						type: 'tech_review'
						verdict: 'safe' | 'unsafe'
				  }
				| {
						type: 'tech_review_entered'
				  }
				| {
						type: 'tech_review_exited'
				  }
				| {
						type: 'tech_review_exit_file_deleted'
				  }
				| {
						type: 'thread_closure'
				  }
				| {
						type: 'thread_reopen'
				  }
				| {
						type: 'deleted'
						private?: boolean
				  }

			export type ThreadMessage = {
				id: string | null
				author_id: string | null
				body: MessageBody
				created: string
				hide_identity: boolean
			}

			export type ThreadMember = {
				id: string
				username: string
				avatar_url: string
				role: string
				badges: number
				created: string
				bio?: string
			}

			export type Thread = {
				id: string
				type: ThreadType
				project_id: string | null
				report_id: string | null
				messages: ThreadMessage[]
				members: ThreadMember[]
			}

			export type SendMessageRequest = {
				body: MessageBody
			}
		}
	}

	export namespace Reports {
		export namespace v3 {
			export type ItemType = 'project' | 'version' | 'user' | 'shared-instance' | 'unknown'

			export type Report = {
				id: string
				report_type: string
				item_id: string
				item_type: ItemType
				shared_instance_version_id?: number
				reporter: string
				body: string
				created: string
				closed: boolean
				thread_id: string
			}

			export type CreateReportRequest = {
				report_type: string
				item_id: string
				item_type: ItemType
				body: string
				uploaded_images?: string[]
			}

			export type EditReportRequest = {
				body?: string
				closed?: boolean
			}

			export type ListReportsParams = {
				count?: number
				offset?: number
				all?: boolean
			}
		}
	}

	export namespace Moderation {
		export namespace Internal {
			export type Ownership =
				| {
						kind: 'user'
						id: string
						name: string
						icon_url: string | null
				  }
				| {
						kind: 'organization'
						id: string
						name: string
						icon_url: string | null
				  }

			export type ProjectsSort = 'oldest' | 'newest' | 'most_external_deps' | 'least_external_deps'

			export type ProjectsRequest = {
				count?: number
				offset?: number
				has_external_dependencies?: boolean
				exclude_technical_review?: boolean
				query?: string
				project_type?: string
				sort?: ProjectsSort
			}

			export type QueueProject = {
				id: string
				slug: string | null
				name: string
				summary: string
				icon_url: string | null
				status: Projects.v2.ProjectStatus
				requested_status: Projects.v2.ProjectStatus | null
				queued: string | null
				published: string
				updated: string
				project_types: string[]
				ownership: Ownership
				external_dependencies_count: number
			}

			export type ProjectsResponse = {
				total: number
				projects: QueueProject[]
			}

			export type ProjectIdsResponse = {
				ids: string[]
			}

			export type LockedByUser = {
				id: string
				username: string
				avatar_url?: string
			}

			export type LockStatusResponse = {
				locked: boolean
				is_own_lock: boolean
				locked_by?: LockedByUser
				locked_at?: string
				expires_at?: string
				expired?: boolean
			}

			export type LockAcquireResponse = {
				success: boolean
				is_own_lock: boolean
				locked_by?: LockedByUser
				locked_at?: string
				expires_at?: string
				expired?: boolean
			}

			export type ReleaseLockResponse = {
				success: boolean
			}

			export type ProjectJudgementStatus = ExternalProjects.Internal.ExternalLicenseStatus

			export type FlameJudgement = {
				type: 'flame'
				id: number
				status: ProjectJudgementStatus
				link: string
				title: string
			}

			export type UnknownJudgement = {
				type: 'unknown'
				status: ProjectJudgementStatus
				proof?: string
				link?: string
				title?: string
			}

			export type ProjectJudgement = FlameJudgement | UnknownJudgement

			export type ProjectJudgements = Record<string, ProjectJudgement>
		}
	}

	export namespace Notifications {
		export namespace v2 {
			export type NotificationAction = {
				title: string
				action_route: [string, string]
			}

			export type NotificationBody = {
				type: string
				project_id?: string
				version_id?: string
				report_id?: string
				thread_id?: string
				message_id?: string
				invited_by?: string
				organization_id?: string
				server_id?: string
				server_name?: string
				team_id?: string
				role?: string
				old_status?: string
				new_status?: string
				[key: string]: unknown
			}

			export type Notification = {
				id: string
				user_id: string
				type: string | null
				title: string
				text: string
				link: string
				read: boolean
				created: string
				actions: NotificationAction[]
				body: NotificationBody
			}
		}
	}

	export namespace Payouts {
		export namespace v3 {
			export type RevenueData = {
				time: number
				revenue: string
				creator_revenue: string
			}

			export type RevenueResponse = {
				all_time: string
				all_time_available: string
				data: RevenueData[]
			}
		}
	}

	export namespace Limits {
		export namespace v3 {
			export type UserLimits = {
				current: number
				max: number
			}
		}
	}

	export namespace Collections {
		export type CollectionStatus = 'listed' | 'unlisted' | 'private' | 'rejected' | 'unknown'

		export type Collection = {
			id: string
			user: string
			name: string
			description: string | null
			icon_url: string | null
			raw_icon_url: string | null
			color: number | null
			status: CollectionStatus
			created: string
			updated: string
			projects: string[]
		}

		export type EditCollectionRequest = {
			name?: string
			description?: string | null
			status?: CollectionStatus
			new_projects?: string[]
		}
	}

	export namespace State {
		export interface PayoutMethodInfo {
			id: string
			type: string
			name: string
			image_logo_url: string | null
		}

		export interface GeneratedState {
			categories: Tags.v2.Category[]
			loaders: Tags.v2.Loader[]
			gameVersions: Tags.v2.GameVersion[]
			donationPlatforms: Tags.v2.DonationPlatform[]
			reportTypes: string[]
			muralBankDetails?: Record<
				string,
				{
					bankNames: string[]
				}
			>
			tremendousIdMap?: Record<
				string,
				{
					name: string
					image_url: string | null
				}
			>

			homePageProjects?: Projects.v2.Project[]
			homePageSearch?: Search.v2.SearchResults
			homePageNotifs?: Search.v2.SearchResults
			products?: Billing.Internal.Product[]

			countries: ISO3166.Country[]
			subdivisions: Record<string, ISO3166.Subdivision[]>

			taxComplianceThresholds?: Record<string, number>

			errors: unknown[]
		}
	}

	export namespace ExternalProjects {
		export namespace Internal {
			export type ExternalLicenseStatus =
				| 'yes'
				| 'with-attribution-and-source'
				| 'with-attribution'
				| 'no'
				| 'permanent-no'
				| 'unidentified'

			export type LinkedFile = {
				name: string | null
				sha1: string
			}

			export type ExternalProject = {
				id: number
				title: string | null
				status: ExternalLicenseStatus
				link: string | null
				exceptions: string | null
				proof: string | null
				flame_project_id: number | null
				inserted_at: string | null
				inserted_by: number | null
				updated_at: string | null
				updated_by: number | null
				linked_files?: LinkedFile[]
			}

			export type SearchRequest = {
				title?: string
				flame_id?: number
			}

			export type UpdateLicenseRequest = {
				title?: string
				status: ExternalLicenseStatus
				link?: string
				exceptions?: string
				proof?: string
				flame_project_id?: number
			}

			export type AddFileRequest = {
				hashes: string[]
				license_id: number
			}
		}
	}

	export namespace TechReview {
		export namespace Internal {
			export type DelphiRule = {
				id: number
				name: string
				rule: string
				priority: number
				on_issue_types: string[]
				revision: number
				current_revision?: number
				created_at: string
				updated_at: string
				created_by: number | null
				updated_by: number | null
				affected_details_count: number
				affected_details: DelphiRuleAffectedDetail[]
			}

			export type DelphiRuleAffectedDetail = {
				detail_id: string
				issue_id: string
				project_id: string | null
				project_name: string | null
				project_icon_url: string | null
				version_id: string | null
				version_name: string | null
				version_number: string | null
				issue_type: string
				key: string
				jar: string | null
				file_path: string
				original_severity: DelphiSeverity
				severity: DelphiSeverity
			}

			export type GetRuleAffectedDetailsRequest = {
				limit?: number
				page?: number
			}

			export type GetRuleAffectedDetailsResponse = {
				total: number
				details: DelphiRuleAffectedDetail[]
			}

			export type WriteDelphiRule = {
				name: string
				rule: string
				priority: number
				on_issue_types: string[]
			}

			export type DelphiIssueTypeSchemaResponse = Record<string, unknown>

			export type TestDelphiRuleRequest = {
				rule: string
				inputs: RuleInput[]
			}

			export type DelphiRuleEffect = {
				severity: DelphiSeverity
			}

			export type DelphiRuleSchema = Record<string, unknown>

			export type DelphiRuleSchemaResponse = {
				input: DelphiRuleSchema
				output: DelphiRuleSchema
				components: Record<string, DelphiRuleSchema>
			}

			export type RuleInput = {
				schema_version: number
				trace: RuleTrace
				file_traces: RuleTrace[]
				scan: {
					delphi_version: number
				}
				artifact: {
					size: number | null
					hashes: Record<string, string>
				}
				project: {
					id: string | null
					types: string[]
				}
				version: {
					id: string | null
					loaders: string[]
				}
				file: {
					id: string | null
				}
			}

			export type RuleTrace = {
				key: string
				issue_type: string
				severity: DelphiSeverity
				jar: string | null
				file_path: string
				data: Record<string, unknown>
			}

			export type TestDelphiRuleResponse = {
				effects: Array<DelphiRuleEffect | null>
			}

			export type DelphiRuleScanPhase = 'scanning' | 'publishing' | 'complete'

			export type DelphiRuleScanEvent = {
				phase: DelphiRuleScanPhase
				revision: number
				scanned: number
				total: number
				effects: number
			}

			export type DelphiRuleScanErrorEvent = {
				message: string
			}

			export type SearchProjectsRequest = {
				limit?: number
				page?: number
				filter?: SearchProjectsFilter
				sort_by?: SearchProjectsSort
			}

			export type SearchProjectsFilter = {
				project_type?: string[]
				replied_to?: 'replied' | 'unreplied'
				project_status?: string[]
				issue_type?: string[]
			}

			export type SearchProjectsSort =
				| 'created_asc'
				| 'created_desc'
				| 'severity_asc'
				| 'severity_desc'

			export type UpdateIssueRequest = {
				detail_id: string
				verdict: DelphiReportIssueStatus
			}

			export type UpdateIssueDetailRequest = {
				verdict: DelphiReportIssueStatus
			}

			export type UpdateGlobalIssueRequest = {
				detail_key: string
				verdict: DelphiReportIssueStatus
			}

			export type SearchGlobalIssueDetailsRequest = {
				limit?: number
				page?: number
				query?: string | null
			}

			export type SearchGlobalIssueDetailsResponse = {
				total: number
				traces: GlobalIssueDetail[]
			}

			export type GetGlobalIssueDetailRequest = {
				detail_key: string
				limit?: number
				after_detail_id?: string | null
			}

			export type GetGlobalIssueDetailResponse = {
				trace: GlobalIssueDetail
				next_after_detail_id: string | null
			}

			export type GlobalIssueDetail = {
				detail_key: string
				verdict: DelphiReportIssueStatus
				local_trace_count: number
				local_traces: GlobalIssueDetailTrace[]
			}

			export type GlobalIssueDetailTrace = {
				detail_id: string
				issue_id: string
				issue_type: string
				project_id: string
				project_slug: string | null
				project_name: string
				version_id: string
				version_number: string
				file_id: string
				file_name: string
				jar: string | null
				file_path: string
				severity: DelphiSeverity
				local_status: DelphiReportIssueStatus
				effective_status: DelphiReportIssueStatus
			}

			export type SubmitProjectRequest = {
				verdict: 'safe' | 'unsafe'
				message?: string
			}

			export type SearchResponse = {
				project_reports: ProjectReport[]
				projects: Record<string, ProjectModerationInfo>
				threads: Record<string, Thread>
				ownership: Record<string, Ownership>
			}

			export type ProjectModerationInfo = {
				id: string
				thread_id: string
				name: string
				project_types: string[]
				icon_url: string | null
			} & Projects.v3.Project

			export type ProjectReport = {
				project_id: string
				max_severity: DelphiSeverity | null
				versions: VersionReport[]
			}

			export type VersionReport = {
				version_id: string
				version_number?: string
				files: FileReport[]
			}

			export type FileReport = {
				report_id: string
				file_id: string
				created: string
				flag_reason: FlagReason
				severity: DelphiSeverity
				file_name: string
				file_size: number
				download_url: string
				issues: FileIssue[]
			}

			export type GetIssueRequest = {
				include_hidden?: boolean
			}

			export type FileIssue = {
				id: string
				report_id: string
				issue_type: string
				details: ReportIssueDetail[]
			}

			export type ReportIssueDetail = {
				id: string
				issue_id: string
				key: string
				jar: string | null
				file_path: string
				decompiled_source: string | null
				data: Record<string, unknown>
				severity: DelphiSeverity
				local_status: DelphiReportIssueStatus | null
				global_status: DelphiReportIssueStatus | null
				status: DelphiReportIssueStatus
			}

			export type Ownership =
				| {
						kind: 'user'
						id: string
						name: string
						icon_url?: string
				  }
				| {
						kind: 'organization'
						id: string
						name: string
						icon_url?: string
				  }

			export type DBThread = {
				id: string
				project_id?: string
				report_id?: string
				type_: ThreadType
				messages: DBThreadMessage[]
				members: string[]
			}

			export type DBThreadMessage = {
				id: string
				thread_id: string
				author_id?: string
				body: MessageBody
				created: string
				hide_identity: boolean
			}

			export type MessageBody =
				| {
						type: 'text'
						body: string
						private?: boolean
						replying_to?: string
						associated_images?: string[]
				  }
				| {
						type: 'status_change'
						new_status: Projects.v2.ProjectStatus
						old_status: Projects.v2.ProjectStatus
				  }
				| {
						type: 'tech_review'
						verdict: 'safe' | 'unsafe'
				  }
				| {
						type: 'tech_review_entered'
				  }
				| {
						type: 'tech_review_exited'
				  }
				| {
						type: 'tech_review_exit_file_deleted'
				  }
				| {
						type: 'thread_closure'
				  }
				| {
						type: 'thread_reopen'
				  }
				| {
						type: 'deleted'
						private?: boolean
				  }

			export type ThreadType = 'report' | 'project' | 'direct_message'

			export type User = {
				id: string
				username: string
				avatar_url: string
				role: string
				badges: number
				created: string
				bio?: string
			}

			export type ThreadMessage = {
				id: string | null
				author_id: string | null
				body: MessageBody
				created: string
				hide_identity: boolean
			}

			export type Thread = {
				id: string
				type: ThreadType
				project_id: string | null
				report_id: string | null
				messages: ThreadMessage[]
				members: User[]
			}

			export type FlagReason = 'delphi'

			export type DelphiSeverity = 'hidden' | 'low' | 'medium' | 'high' | 'severe' | 'malware'

			export type DelphiReportIssueStatus = 'pending' | 'safe' | 'unsafe'

			export type ProjectReportResponse = {
				project_report: ProjectReport | null
				thread: Thread
			}
		}
	}

	export namespace Pats {
		export namespace v2 {
			export type PersonalAccessToken = {
				id: string
				name: string
				access_token: string | null
				scopes: number
				user_id: string
				created: string
				expires: string
				last_used: string | null
			}

			export type CreatePatRequest = {
				scopes: number
				name: string
				expires: string
			}

			export type ModifyPatRequest = {
				scopes?: number
				name?: string
				expires?: string
			}
		}
	}

	export namespace Sessions {
		export namespace v2 {
			export type Session = {
				id: string
				session: string | null
				user_id: string
				created: string
				last_login: string
				expires: string
				refresh_expires: string
				os: string | null
				platform: string | null
				user_agent: string
				city: string | null
				country: string | null
				ip: string
				current: boolean
			}
		}
	}
}
