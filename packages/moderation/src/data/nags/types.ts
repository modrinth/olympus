import type { Labrinth } from '@modrinth/api-client'
import type { MessageDescriptor } from '@modrinth/ui'

import type { NagDestinationId } from '../../types/nags.ts'

export interface NagDefinition {
	title:
		| MessageDescriptor
		| ((context: {
				nag: Labrinth.Projects.v3.ProjectNag
				projectType?: string
		  }) => MessageDescriptor)
	description:
		| MessageDescriptor
		| ((context: {
				nag: Labrinth.Projects.v3.ProjectNag
				projectType?: string
		  }) => MessageDescriptor)
	destination: NagDestinationId
	linkTitle?: MessageDescriptor
}

export interface FieldValidationMessage {
	code: string
	severity: 'error' | 'warning' | 'suggestion'
	message: MessageDescriptor
	values?: Record<string, string | number | boolean>
}

export type NagDefinitions = Partial<
	Record<Labrinth.Projects.v3.NormalizedProjectNagKind, NagDefinition>
>
