<template>
  <Card class="log-card">
    <div class="button-row">
      <DropdownSelect
        v-model="selectedLogIndex"
        :default-value="0"
        name="Log date"
        :options="logs.map((_, index) => index)"
        :display-name="(option) => logs[option]?.name"
        :disabled="logs.length === 0"
      />
      <div class="button-group">
        <Button :disabled="!logs[selectedLogIndex]" @click="copyLog()">
          <ClipboardCopyIcon v-if="!copied" />
          <CheckIcon v-else />
          {{ copied ? 'Copied' : 'Copy' }}
        </Button>
        <Button color="primary" :disabled="offline || !logs[selectedLogIndex]" @click="share">
          <ShareIcon />
          Share
        </Button>
        <Button
          v-if="logs[selectedLogIndex] && logs[selectedLogIndex].live === true"
          @click="clearLiveLog()"
        >
          <TrashIcon />
          Clear
        </Button>

        <Button
          v-else
          :disabled="!logs[selectedLogIndex] || logs[selectedLogIndex].live === true"
          color="danger"
          @click="deleteLog()"
        >
          <TrashIcon />
          Delete
        </Button>
      </div>
    </div>
    <div class="button-row">
      <input
        id="text-filter"
        v-model="searchFilter"
        autocomplete="off"
        type="text"
        class="text-filter"
        placeholder="Type to filter logs..."
      />
      <div class="filter-group">
        <Checkbox
          v-for="level in levels"
          :key="level.toLowerCase()"
          v-model="levelFilters[level.toLowerCase()]"
          class="filter-checkbox"
        >
          {{ level }}
        </Checkbox>
      </div>
    </div>
    <div class="log-text">
      <VirtualScroller
        ref="logContainer"
        class="scroller"
        :default-size="20"
        :items="displayProcessedLogs"
      >
        <template #item="{ ref }">
          <div class="user no-wrap">
            <span :style="{ color: ref.prefixColor, 'font-weight': ref.weight }">{{
              ref.prefix
            }}</span>
            <span :style="{ color: ref.textColor }">{{ ref.text }}</span>
          </div>
        </template>
      </VirtualScroller>
    </div>
  </Card>
  <ShareModal
    ref="shareModal"
    header="Share Log"
    share-title="Instance Log"
    share-text="Check out this log from an instance on the Modrinth App"
    :open-in-new-tab="false"
    link
  />
</template>

<script setup>
import { CheckIcon, ClipboardCopyIcon, ShareIcon, TrashIcon } from '@modrinth/assets'
import { Button, Card, ShareModal, Checkbox, DropdownSelect } from '@modrinth/ui'
import {
  delete_logs_by_filename,
  get_logs,
  get_output_by_filename,
  get_latest_log_cursor,
} from '@/helpers/logs.js'
import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'
import dayjs from 'dayjs'
import isToday from 'dayjs/plugin/isToday'
import isYesterday from 'dayjs/plugin/isYesterday'
import { get_by_profile_path } from '@/helpers/process.js'
import { useRoute } from 'vue-router'
import { process_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'
import { ofetch } from 'ofetch'
import { createVirtualScroller } from 'vue-typed-virtual-list'

dayjs.extend(isToday)
dayjs.extend(isYesterday)

const VirtualScroller = createVirtualScroller()

const route = useRoute()

const props = defineProps({
  instance: {
    type: Object,
    required: true,
  },
  offline: {
    type: Boolean,
    default: false,
  },
  playing: {
    type: Boolean,
    default: false,
  },
})

const currentLiveLog = ref(null)
const currentLiveLogCursor = ref(0)
const emptyText = ['No live game detected.', 'Start your game to proceed.']

const logs = ref([])
await setLogs()

const logsColored = true

const selectedLogIndex = ref(0)
const copied = ref(false)
const logContainer = ref(null)
const interval = ref(null)
const userScrolled = ref(false)
const isAutoScrolling = ref(false)
const shareModal = ref(null)

const levels = ['Comment', 'Error', 'Warn', 'Info', 'Debug', 'Trace']
const levelFilters = ref({})
levels.forEach((level) => {
  levelFilters.value[level.toLowerCase()] = true
})
const searchFilter = ref('')

function shouldDisplay(processedLine) {
  if (!processedLine.level) {
    return true
  }

  if (!levelFilters.value[processedLine.level.toLowerCase()]) {
    return false
  }
  if (searchFilter.value !== '') {
    if (!processedLine.text.toLowerCase().includes(searchFilter.value.toLowerCase())) {
      return false
    }
  }
  return true
}

// Selects from the processed logs which ones should be displayed (shouldDisplay)
// In addition, splits each line by \n. Each split line is given the same properties as the original line
const displayProcessedLogs = computed(() => {
  return processedLogs.value.filter((l) => shouldDisplay(l))
})

const processedLogs = computed(() => {
  // split based on newline and timestamp lookahead
  // (not just newline because of multiline messages)
  const splitPattern = /\n(?=(?:#|\[\d\d:\d\d:\d\d\]))/

  const lines = logs.value[selectedLogIndex.value]?.stdout.split(splitPattern) || []
  const processed = []
  let id = 0
  for (let i = 0; i < lines.length; i++) {
    // Then split off of \n.
    // Lines that are not the first have prefix = null
    const text = getLineText(lines[i])
    const prefix = getLinePrefix(lines[i])
    const prefixColor = getLineColor(lines[i], true)
    const textColor = getLineColor(lines[i], false)
    const weight = getLineWeight(lines[i])
    const level = getLineLevel(lines[i])
    text.split('\n').forEach((line, index) => {
      processed.push({
        id: id,
        text: line,
        prefix: index === 0 ? prefix : null,
        prefixColor: prefixColor,
        textColor: textColor,
        weight: weight,
        level: level,
      })
      id += 1
    })
  }
  return processed
})

async function getLiveStdLog() {
  if (route.params.id) {
    const processes = await get_by_profile_path(route.params.id).catch(handleError)
    let returnValue
    if (processes.length === 0) {
      returnValue = emptyText.join('\n')
    } else {
      const logCursor = await get_latest_log_cursor(
        props.instance.path,
        currentLiveLogCursor.value,
      ).catch(handleError)
      if (logCursor.new_file) {
        currentLiveLog.value = ''
      }
      currentLiveLog.value = currentLiveLog.value + logCursor.output
      currentLiveLogCursor.value = logCursor.cursor
      returnValue = currentLiveLog.value
    }
    return { name: 'Live Log', stdout: returnValue, live: true }
  }
  return null
}

async function getLogs() {
  return (await get_logs(props.instance.path, true).catch(handleError))
    .filter(
      // filter out latest_stdout.log or anything without .log in it
      (log) =>
        log.filename !== 'latest_stdout.log' &&
        log.filename !== 'latest_stdout' &&
        log.stdout !== '' &&
        (log.filename.includes('.log') || log.filename.endsWith('.txt')),
    )
    .map((log) => {
      log.name = log.filename || 'Unknown'
      log.stdout = 'Loading...'
      return log
    })
}

async function setLogs() {
  const [liveStd, allLogs] = await Promise.all([getLiveStdLog(), getLogs()])
  logs.value = [liveStd, ...allLogs]
}

const copyLog = () => {
  if (logs.value.length > 0 && logs.value[selectedLogIndex.value]) {
    navigator.clipboard.writeText(logs.value[selectedLogIndex.value].stdout)
    copied.value = true
  }
}

const share = async () => {
  if (logs.value.length > 0 && logs.value[selectedLogIndex.value]) {
    const url = await ofetch('https://api.mclo.gs/1/log', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: `content=${encodeURIComponent(logs.value[selectedLogIndex.value].stdout)}`,
    }).catch(handleError)

    shareModal.value.show(url.url)
  }
}

watch(selectedLogIndex, async (newIndex) => {
  copied.value = false
  userScrolled.value = false

  if (logs.value.length > 1 && newIndex !== 0) {
    logs.value[newIndex].stdout = 'Loading...'
    logs.value[newIndex].stdout = await get_output_by_filename(
      props.instance.path,
      logs.value[newIndex].log_type,
      logs.value[newIndex].filename,
    ).catch(handleError)
  }
})

if (logs.value.length > 1 && !props.playing) {
  selectedLogIndex.value = 1
} else {
  selectedLogIndex.value = 0
}

const deleteLog = async () => {
  if (logs.value[selectedLogIndex.value] && selectedLogIndex.value !== 0) {
    let deleteIndex = selectedLogIndex.value
    selectedLogIndex.value = deleteIndex - 1
    await delete_logs_by_filename(
      props.instance.path,
      logs.value[deleteIndex].log_type,
      logs.value[deleteIndex].filename,
    ).catch(handleError)
    await setLogs()
  }
}

const clearLiveLog = async () => {
  currentLiveLog.value = ''
  // does not reset cursor
}

const isLineLevel = (text, level) => {
  if ((text.includes('/INFO') || text.includes('[System] [CHAT]')) && level === 'info') {
    return true
  }

  if (text.includes('/WARN') && level === 'warn') {
    return true
  }

  if (text.includes('/DEBUG') && level === 'debug') {
    return true
  }

  if (text.includes('/TRACE') && level === 'trace') {
    return true
  }

  const errorTriggers = ['/ERROR', 'Exception:', ':?]', 'Error', '[thread', '	at']
  if (level === 'error') {
    for (const trigger of errorTriggers) {
      if (text.includes(trigger)) return true
    }
  }

  if (text.trim()[0] === '#' && level === 'comment') {
    return true
  }
  return false
}

const getLineWeight = (text) => {
  if (
    !logsColored ||
    isLineLevel(text, 'info') ||
    isLineLevel(text, 'debug') ||
    isLineLevel(text, 'trace')
  ) {
    return 'normal'
  }

  if (isLineLevel(text, 'error') || isLineLevel(text, 'warn')) {
    return 'bold'
  }
}

const getLineLevel = (text) => {
  for (const level of levels) {
    if (isLineLevel(text, level.toLowerCase())) {
      return level
    }
  }
}

const getLineColor = (text, prefix) => {
  if (isLineLevel(text, 'comment')) {
    return 'var(--color-green)'
  }

  if (!logsColored || text.includes('[System] [CHAT]')) {
    return 'var(--color-white)'
  }
  if (
    (isLineLevel(text, 'info') || isLineLevel(text, 'debug') || isLineLevel(text, 'trace')) &&
    prefix
  ) {
    return 'var(--color-blue)'
  }
  if (isLineLevel(text, 'warn')) {
    return 'var(--color-orange)'
  }
  if (isLineLevel(text, 'error')) {
    return 'var(--color-red)'
  }
}

const getLinePrefix = (text) => {
  if (text.includes(']:')) {
    return text.split(']:')[0] + ']:'
  }
}

const getLineText = (text) => {
  if (text.includes(']:')) {
    if (text.split(']:').length > 2) {
      return text.split(']:').slice(1).join(']:')
    }
    return text.split(']:')[1]
  } else {
    return text
  }
}

function handleUserScroll() {
  if (!isAutoScrolling.value) {
    userScrolled.value = true
  }
}

interval.value = setInterval(async () => {
  if (logs.value.length > 0) {
    logs.value[0] = await getLiveStdLog()
    const logContainerElement = logContainer.value.$el
    const scroll =
      logContainerElement.scrollHeight -
      logContainerElement.scrollTop -
      logContainerElement.clientHeight
    // const scroll = logContainer.value.$el.scrollHeight - logContainer.value.$el.scrollTop - logContainer.value.$el.clientHeight

    // Allow resetting of userScrolled if the user scrolls to the bottom
    if (selectedLogIndex.value === 0) {
      if (scroll <= 10) userScrolled.value = false
      if (!userScrolled.value) {
        await nextTick()
        isAutoScrolling.value = true
        logContainer.value.scrollTo(displayProcessedLogs.value.length - 1)
        setTimeout(() => (isAutoScrolling.value = false), 50)
      }
    }
  }
}, 250)

const unlistenProcesses = await process_listener(async (e) => {
  if (e.event === 'launched') {
    currentLiveLog.value = ''
    currentLiveLogCursor.value = 0
    selectedLogIndex.value = 0
  }
  if (e.event === 'finished') {
    currentLiveLog.value = ''
    currentLiveLogCursor.value = 0
    userScrolled.value = false
    await setLogs()
    selectedLogIndex.value = 1
  }
})

onMounted(() => {
  logContainer.value.$el.addEventListener('scroll', handleUserScroll)
})

onBeforeUnmount(() => {
  logContainer.value.$el.removeEventListener('scroll', handleUserScroll)
})
onUnmounted(() => {
  clearInterval(interval.value)
  unlistenProcesses()
})
</script>

<style scoped lang="scss">
.log-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  /*                   gaps   appbar    promotion*/
  height: calc(100vh - 3rem - 3.25rem - 47px);
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: 0.5rem;
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.log-text {
  width: 100%;
  height: 100%;
  font-family: var(--mono-font);
  background-color: var(--color-accent-contrast);
  color: var(--color-contrast);
  border-radius: var(--radius-lg);
  padding: 1.5rem;
  overflow-x: auto; /* Enables horizontal scrolling */
  overflow-y: hidden; /* Disables vertical scrolling on this wrapper */
  white-space: nowrap; /* Keeps content on a single line */
  white-space: normal;
  color-scheme: dark;
}

.filter-checkbox {
  margin-bottom: 0.3rem;
  font-size: 1rem;

  svg {
    display: flex;
    align-self: center;
    justify-self: center;
  }
}

.filter-group {
  display: flex;
  padding: 0.6rem;
  flex-direction: row;
  overflow: auto;
  gap: 0.5rem;

  &::-webkit-scrollbar-track,
  &::-webkit-scrollbar-thumb {
    border-radius: 10px;
  }
}

:deep(.vue-recycle-scroller__item-wrapper) {
  overflow: visible; /* Enables horizontal scrolling */
}

:deep(.vue-recycle-scroller) {
  &::-webkit-scrollbar-corner {
    background-color: var(--color-bg);
    border-radius: 0 0 10px 0;
  }
}

.scroller {
  height: 100%;
}

.user {
  height: 32%;
  padding: 0 12px;
  display: flex;

  align-items: center;
}
</style>
