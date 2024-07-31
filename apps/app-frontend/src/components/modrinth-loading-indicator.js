import { computed, defineComponent, h, onBeforeUnmount, ref, watch } from 'vue'
import { useLoading } from '@/store/state.js'

export default defineComponent({
  props: {
    throttle: {
      type: Number,
      default: 50,
    },
    duration: {
      type: Number,
      default: 500,
    },
    height: {
      type: Number,
      default: 3,
    },
    color: {
      type: [String, Boolean],
      default:
        'repeating-linear-gradient(to right, var(--color-brand) 0%, var(--color-brand) 100%)',
    },
  },
  setup(props, { slots }) {
    const indicator = useLoadingIndicator({
      duration: props.duration,
      throttle: props.throttle,
    })

    onBeforeUnmount(() => indicator.clear)

    const loading = useLoading()

    watch(loading, (newValue) => {
      if (newValue.loading) {
        indicator.start()
      } else {
        indicator.finish()
      }
    })

    return () =>
      h(
        'div',
        {
          style: {
            position: 'absolute',
            pointerEvents: 'none',
            width: `calc(100% * ${indicator.progress.value / 100})`,
            height: `${props.height}px`,
            opacity: indicator.isLoading.value ? 1 : 0,
            background: props.color || undefined,
            backgroundSize: `${(100 / indicator.progress.value) * 100}% auto`,
            transition: 'width 0.1s, height 0.4s, opacity 0.4s',
            zIndex: 6,
          },
        },
        slots,
      )
  },
})

function useLoadingIndicator(opts) {
  const progress = ref(0)
  const isLoading = ref(false)
  const step = computed(() => 10000 / opts.duration)

  let _timer = null
  let _throttle = null

  function start() {
    clear()
    progress.value = 0
    if (opts.throttle) {
      _throttle = setTimeout(() => {
        isLoading.value = true
        _startTimer()
      }, opts.throttle)
    } else {
      isLoading.value = true
      _startTimer()
    }
  }
  function finish() {
    progress.value = 100
    _hide()
  }

  function clear() {
    clearInterval(_timer)
    clearTimeout(_throttle)
    _timer = null
    _throttle = null
  }

  function _increase(num) {
    progress.value = Math.min(100, progress.value + num)
  }

  function _hide() {
    clear()
    setTimeout(() => {
      isLoading.value = false
      setTimeout(() => {
        progress.value = 0
      }, 400)
    }, 500)
  }

  function _startTimer() {
    _timer = setInterval(() => {
      _increase(step.value)
    }, 100)
  }

  return {
    progress,
    isLoading,
    start,
    finish,
    clear,
  }
}
