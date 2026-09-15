<template>
	<Transition name="validation-message">
		<div v-if="validations.length > 0" v-bind="$attrs" class="grid w-full grid-rows-[1fr]">
			<div class="flex min-h-0 w-full flex-col gap-1.5 overflow-hidden">
				<div
					v-for="(validation, index) in validations"
					:key="validation.code ?? validation.message?.id ?? index"
					class="flex w-full items-start gap-1.5"
					:class="{
						'text-red': validation.severity === 'error',
						'text-orange': validation.severity === 'warning',
						'text-purple': validation.severity === 'suggestion',
					}"
				>
					<component
						:is="
							validation.severity === 'error'
								? XCircleIcon
								: validation.severity === 'suggestion'
									? LightBulbIcon
									: TriangleAlertIcon
						"
						class="mt-0.5 shrink-0"
					/>
					{{
						validation.message ? formatMessage(validation.message, validation.values) : undefined
					}}
				</div>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { LightBulbIcon, TriangleAlertIcon, XCircleIcon } from '@modrinth/assets'
import type { FieldValidationMessage } from '@modrinth/moderation'
import { injectProjectPageContext, useVIntl } from '@modrinth/ui'
import { computed, onScopeDispose, ref, shallowRef, watch } from 'vue'

defineOptions({ inheritAttrs: false })

type ValidationCheck = Omit<FieldValidationMessage, 'code'> & { code?: string }

type ValidationCheckInput = ValidationCheck | ValidationCheck[] | null

const props = withDefaults(
	defineProps<{
		check?: ValidationCheckInput
		debounce?: number
		projectField?: unknown
		currentField?: unknown
	}>(),
	{
		check: null,
		debounce: 300,
	},
)

const { formatMessage } = useVIntl()
const { projectValidationLoading } = injectProjectPageContext()
const displayedCheck = shallowRef<ValidationCheckInput>(props.check)
const validationIsStale = ref(props.projectField !== props.currentField)
let debounceTimer: ReturnType<typeof setTimeout> | undefined
let validationRefreshCompleted = false

function updateDisplayedCheck(check: ValidationCheckInput) {
	clearTimeout(debounceTimer)
	const update = () => {
		displayedCheck.value = check
		if (
			validationRefreshCompleted &&
			!projectValidationLoading.value &&
			props.projectField === props.currentField
		) {
			validationIsStale.value = false
			validationRefreshCompleted = false
		}
	}

	if (props.debounce <= 0) {
		update()
		return
	}

	debounceTimer = setTimeout(update, props.debounce)
}

watch(
	() => [props.projectField, props.currentField] as const,
	([projectField, currentField], [previousProjectField]) => {
		if (projectField !== currentField) {
			validationIsStale.value = true
			validationRefreshCompleted = false
		} else if (projectField === previousProjectField) {
			validationIsStale.value = false
			validationRefreshCompleted = false
		}
	},
)

watch(projectValidationLoading, (loading, wasLoading) => {
	if (!loading && wasLoading) {
		validationRefreshCompleted = true
		updateDisplayedCheck(props.check)
	}
})

watch(() => props.check, updateDisplayedCheck)

onScopeDispose(() => clearTimeout(debounceTimer))

const validations = computed(() => {
	if (validationIsStale.value) return []

	return Array.isArray(displayedCheck.value)
		? displayedCheck.value
		: displayedCheck.value
			? [displayedCheck.value]
			: []
})
</script>

<style scoped>
.validation-message-enter-active,
.validation-message-leave-active {
	transition:
		grid-template-rows 150ms ease,
		opacity 150ms ease,
		transform 150ms ease;
}

.validation-message-enter-from,
.validation-message-leave-to {
	grid-template-rows: 0fr;
	opacity: 0;
	transform: translateY(-0.25rem);
}

@media (prefers-reduced-motion: reduce) {
	.validation-message-enter-active,
	.validation-message-leave-active {
		transition: none;
	}
}
</style>
