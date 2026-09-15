import type { Labrinth } from '@modrinth/api-client'

export const PROJECT_REVIEW_VALIDATION_ERROR =
	'project must have no required validation nags before or while under review'

export function canSubmitProjectForReview(
	validation: Pick<Labrinth.Projects.v3.ProjectValidationResponse, 'nags'> | null | undefined,
	loading: boolean,
): boolean {
	return !loading && !!validation && !validation.nags.some((nag) => nag.severity === 'required')
}
