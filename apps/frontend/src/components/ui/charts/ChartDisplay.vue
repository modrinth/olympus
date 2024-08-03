<template>
  <div>
    <div v-if="analytics.error.value" class="universal-card">
      <h2>
        <span class="label__title">Error</span>
      </h2>
      <div>
        {{ analytics.error.value }}
      </div>
    </div>
    <div v-else class="graphs">
      <div class="graphs__vertical-bar">
        <client-only>
          <CompactChart
            v-if="analytics.formattedData.value.downloads"
            ref="tinyDownloadChart"
            :title="`Downloads`"
            color="var(--color-brand)"
            :value="formatNumber(analytics.formattedData.value.downloads.sum, false)"
            :data="analytics.formattedData.value.downloads.chart.sumData"
            :labels="analytics.formattedData.value.downloads.chart.labels"
            suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
            :class="`clickable chart-button-base button-base ${
              selectedChart === 'downloads'
                ? 'chart-button-base__selected button-base__selected'
                : ''
            }`"
            :onclick="() => (selectedChart = 'downloads')"
            role="button"
          />
        </client-only>
        <client-only>
          <CompactChart
            v-if="analytics.formattedData.value.views"
            ref="tinyViewChart"
            :title="`Views`"
            color="var(--color-blue)"
            :value="formatNumber(analytics.formattedData.value.views.sum, false)"
            :data="analytics.formattedData.value.views.chart.sumData"
            :labels="analytics.formattedData.value.views.chart.labels"
            suffix="<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z'/><circle cx='12' cy='12' r='3'/></svg>"
            :class="`clickable chart-button-base button-base ${
              selectedChart === 'views' ? 'chart-button-base__selected button-base__selected' : ''
            }`"
            :onclick="() => (selectedChart = 'views')"
            role="button"
          />
        </client-only>
        <client-only>
          <CompactChart
            v-if="analytics.formattedData.value.revenue"
            ref="tinyRevenueChart"
            :title="`Revenue`"
            color="var(--color-purple)"
            :value="formatMoney(analytics.formattedData.value.revenue.sum, false)"
            :data="analytics.formattedData.value.revenue.chart.sumData"
            :labels="analytics.formattedData.value.revenue.chart.labels"
            is-money
            :class="`clickable chart-button-base button-base ${
              selectedChart === 'revenue' ? 'chart-button-base__selected button-base__selected' : ''
            }`"
            :onclick="() => (selectedChart = 'revenue')"
            role="button"
          />
        </client-only>
      </div>
      <div class="graphs__main-graph">
        <div class="universal-card">
          <div class="chart-controls">
            <h2>
              <span class="label__title">
                {{ formatCategoryHeader(selectedChart) }}
              </span>
              <span class="label__subtitle">
                {{ formattedCategorySubtitle }}
              </span>
            </h2>
            <div class="chart-controls__buttons">
              <Button v-tooltip="'Toggle project colors'" icon-only @click="onToggleColors">
                <PaletteIcon />
              </Button>
              <Button v-tooltip="'Download this data as CSV'" icon-only @click="onDownloadSetAsCSV">
                <DownloadIcon />
              </Button>
              <Button v-tooltip="'Refresh the chart'" icon-only @click="resetCharts">
                <UpdatedIcon />
              </Button>
              <DropdownSelect
                class="range-dropdown"
                v-model="selectedRange"
                :options="ranges"
                name="Time range"
                :display-name="
                  (o: RangeObject) => o?.getLabel([startDate, endDate]) ?? 'Loading...'
                "
              />
            </div>
          </div>
          <div class="chart-area">
            <div class="chart">
              <client-only>
                <Chart
                  v-if="analytics.formattedData.value.downloads && selectedChart === 'downloads'"
                  ref="downloadsChart"
                  type="line"
                  name="Download data"
                  :hide-legend="true"
                  :data="analytics.formattedData.value.downloads.chart.data"
                  :labels="analytics.formattedData.value.downloads.chart.labels"
                  suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
                  :colors="
                    isUsingProjectColors
                      ? analytics.formattedData.value.downloads.chart.colors
                      : analytics.formattedData.value.downloads.chart.defaultColors
                  "
                />
                <Chart
                  v-if="analytics.formattedData.value.views && selectedChart === 'views'"
                  ref="viewsChart"
                  type="line"
                  name="View data"
                  :hide-legend="true"
                  :data="analytics.formattedData.value.views.chart.data"
                  :labels="analytics.formattedData.value.views.chart.labels"
                  suffix="<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z'/><circle cx='12' cy='12' r='3'/></svg>"
                  :colors="
                    isUsingProjectColors
                      ? analytics.formattedData.value.views.chart.colors
                      : analytics.formattedData.value.views.chart.defaultColors
                  "
                />
                <Chart
                  v-if="analytics.formattedData.value.revenue && selectedChart === 'revenue'"
                  ref="revenueChart"
                  type="line"
                  name="Revenue data"
                  :hide-legend="true"
                  :data="analytics.formattedData.value.revenue.chart.data"
                  :labels="analytics.formattedData.value.revenue.chart.labels"
                  is-money
                  suffix="<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><line x1='12' y1='2' x2='12' y2='22'></line><path d='M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6'></path></svg>"
                  :colors="
                    isUsingProjectColors
                      ? analytics.formattedData.value.revenue.chart.colors
                      : analytics.formattedData.value.revenue.chart.defaultColors
                  "
                />
              </client-only>
            </div>
            <div class="legend">
              <div class="legend__items">
                <template v-for="project in selectedDataSetProjects" :key="project">
                  <button
                    v-tooltip="project.title"
                    :class="`legend__item button-base btn-transparent ${
                      !projectIsOnDisplay(project.id) ? 'btn-dimmed' : ''
                    }`"
                    @click="
                      () =>
                        projectIsOnDisplay(project.id) &&
                        analytics.validProjectIds.value.includes(project.id)
                          ? removeProjectFromDisplay(project.id)
                          : addProjectToDisplay(project.id)
                    "
                  >
                    <div
                      :style="{
                        '--color-brand': isUsingProjectColors
                          ? intToRgba(project.color, project.id, theme.active ?? undefined)
                          : getDefaultColor(project.id),
                      }"
                      class="legend__item__color"
                    ></div>
                    <div class="legend__item__text">{{ project.title }}</div>
                  </button>
                </template>
              </div>
            </div>
          </div>
        </div>
        <div class="country-data">
          <Card
            v-if="
              analytics.formattedData.value?.downloadsByCountry &&
              selectedChart === 'downloads' &&
              analytics.formattedData.value.downloadsByCountry.data.length > 0
            "
            class="country-downloads"
          >
            <label>
              <span class="label__title">Downloads by region</span>
            </label>
            <div class="country-values">
              <div
                v-for="[name, count] in analytics.formattedData.value.downloadsByCountry.data"
                :key="name"
                class="country-value"
              >
                <div class="country-flag-container">
                  <img
                    :src="
                      name.toLowerCase() === 'xx' || !name
                        ? 'https://cdn.modrinth.com/placeholder-banner.svg'
                        : countryCodeToFlag(name)
                    "
                    alt="Hidden country"
                    class="country-flag"
                  />
                </div>
                <div class="country-text">
                  <strong class="country-name"
                    ><template v-if="name.toLowerCase() === 'xx' || !name">Hidden</template>
                    <template v-else>{{ countryCodeToName(name) }}</template>
                  </strong>
                  <span class="data-point">{{ formatNumber(count) }}</span>
                </div>
                <div
                  v-tooltip="
                    formatPercent(count, analytics.formattedData.value.downloadsByCountry.sum)
                  "
                  class="percentage-bar"
                >
                  <span
                    :style="{
                      width: formatPercent(
                        count,
                        analytics.formattedData.value.downloadsByCountry.sum,
                      ),
                      backgroundColor: 'var(--color-brand)',
                    }"
                  ></span>
                </div>
              </div>
            </div>
          </Card>
          <Card
            v-if="
              analytics.formattedData.value?.viewsByCountry &&
              selectedChart === 'views' &&
              analytics.formattedData.value.viewsByCountry.data.length > 0
            "
            class="country-downloads"
          >
            <label>
              <span class="label__title">Page views by region</span>
            </label>
            <div class="country-values">
              <div
                v-for="[name, count] in analytics.formattedData.value.viewsByCountry.data"
                :key="name"
                class="country-value"
              >
                <div class="country-flag-container">
                  <img
                    :src="
                      name.toLowerCase() === 'xx' || !name
                        ? 'https://cdn.modrinth.com/placeholder-banner.svg'
                        : countryCodeToFlag(name)
                    "
                    alt="Hidden country"
                    class="country-flag"
                  />
                </div>
                <div class="country-text">
                  <strong class="country-name">
                    <template v-if="name.toLowerCase() === 'xx' || !name">Hidden</template>
                    <template v-else>{{ countryCodeToName(name) }}</template>
                  </strong>
                  <span class="data-point">{{ formatNumber(count) }}</span>
                </div>
                <div
                  v-tooltip="
                    `${
                      Math.round(
                        (count / analytics.formattedData.value.viewsByCountry.sum) * 10000,
                      ) / 100
                    }%`
                  "
                  class="percentage-bar"
                >
                  <span
                    :style="{
                      width: `${(count / analytics.formattedData.value.viewsByCountry.sum) * 100}%`,
                      backgroundColor: 'var(--color-blue)',
                    }"
                  ></span>
                </div>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Button, Card, DropdownSelect } from "@modrinth/ui";
import { formatMoney, formatNumber, formatCategoryHeader } from "@modrinth/utils";
import { UpdatedIcon, DownloadIcon } from "@modrinth/assets";
import dayjs from "dayjs";
import { computed } from "vue";

import { analyticsSetToCSVString, intToRgba } from "~/utils/analytics.js";

import { UiChartsCompactChart as CompactChart, UiChartsChart as Chart } from "#components";

import PaletteIcon from "~/assets/icons/palette.svg?component";

const router = useNativeRouter();
const theme = useTheme();

const props = withDefaults(
  defineProps<{
    projects?: any[];
    /**
     * @deprecated Use `ranges` instead
     */
    resoloutions?: Record<string, number>;
    ranges?: RangeObject[];
    personal?: boolean;
  }>(),
  {
    projects: undefined,
    resoloutions: () => defaultResoloutions,
    ranges: () => defaultRanges,
    personal: false,
  },
);

const projects = ref(props.projects || []);

// const selectedChart = ref('downloads')
const selectedChart = computed({
  get: () => {
    const id = (router.currentRoute.value.query?.chart as string | undefined) || "downloads";
    // if the id is anything but the 3 charts we have or undefined, throw an error
    if (!["downloads", "views", "revenue"].includes(id)) {
      throw new Error(`Unknown chart ${id}`);
    }
    return id;
  },
  set: (chart) => {
    router.push({
      query: {
        ...router.currentRoute.value.query,
        chart,
      },
    });
  },
});

// Chart refs
const downloadsChart = ref();
const viewsChart = ref();
const revenueChart = ref();
const tinyDownloadChart = ref();
const tinyViewChart = ref();
const tinyRevenueChart = ref();

const selectedDisplayProjects = ref(props.projects || []);

const removeProjectFromDisplay = (id: string) => {
  selectedDisplayProjects.value = selectedDisplayProjects.value.filter((p) => p.id !== id);
};

const addProjectToDisplay = (id: string) => {
  selectedDisplayProjects.value = [
    ...selectedDisplayProjects.value,
    props.projects?.find((p) => p.id === id),
  ].filter(Boolean);
};

const projectIsOnDisplay = (id: string) => {
  return selectedDisplayProjects.value?.some((p) => p.id === id) ?? false;
};

const resetCharts = () => {
  downloadsChart.value?.resetChart();
  viewsChart.value?.resetChart();
  revenueChart.value?.resetChart();

  tinyDownloadChart.value?.resetChart();
  tinyViewChart.value?.resetChart();
  tinyRevenueChart.value?.resetChart();
};

const isUsingProjectColors = computed({
  get: () => {
    return (
      router.currentRoute.value.query?.colors === "true" ||
      router.currentRoute.value.query?.colors === undefined
    );
  },
  set: (newValue) => {
    router.push({
      query: {
        ...router.currentRoute.value.query,
        colors: newValue ? "true" : "false",
      },
    });
  },
});

const startDate = ref(dayjs().startOf("day"));
const endDate = ref(dayjs().endOf("day"));
const timeResolution = ref(30);

onBeforeMount(() => {
  // Load cached data and range from localStorage - cache.
  if (import.meta.client) {
    const rangeLabel = localStorage.getItem("analyticsSelectedRange");
    if (rangeLabel) {
      const range = props.ranges.find((r) => r.getLabel([dayjs(), dayjs()]) === rangeLabel)!;

      if (range !== undefined) {
        internalRange.value = range;
        const ranges = range.getDates(dayjs());
        timeResolution.value = range.timeResolution;
        startDate.value = ranges.startDate;
        endDate.value = ranges.endDate;
      }
    }
  }
});

onMounted(() => {
  if (internalRange.value === null) {
    internalRange.value = props.ranges.find(
      (r) => r.getLabel([dayjs(), dayjs()]) === "Previous 30 days",
    )!;
  }

  const ranges = selectedRange.value.getDates(dayjs());
  startDate.value = ranges.startDate;
  endDate.value = ranges.endDate;
  timeResolution.value = selectedRange.value.timeResolution;
});

const internalRange: Ref<RangeObject> = ref(null as unknown as RangeObject);

const selectedRange = computed({
  get: () => {
    return internalRange.value;
  },
  set: (newRange) => {
    const ranges = newRange.getDates(dayjs());
    startDate.value = ranges.startDate;
    endDate.value = ranges.endDate;
    timeResolution.value = newRange.timeResolution;

    internalRange.value = newRange;

    if (import.meta.client) {
      localStorage.setItem(
        "analyticsSelectedRange",
        internalRange.value?.getLabel([dayjs(), dayjs()]) ?? "Previous 30 days",
      );
    }
  },
});

const analytics = useFetchAllAnalytics(
  resetCharts,
  projects,
  selectedDisplayProjects,
  props.personal,
  startDate,
  endDate,
  timeResolution,
);

const formattedCategorySubtitle = computed(() => {
  return (
    selectedRange.value?.getLabel([dayjs(startDate.value), dayjs(endDate.value)]) ?? "Loading..."
  );
});

const selectedDataSet = computed(() => {
  switch (selectedChart.value) {
    case "downloads":
      return analytics.totalData.value.downloads;
    case "views":
      return analytics.totalData.value.views;
    case "revenue":
      return analytics.totalData.value.revenue;
    default:
      throw new Error(`Unknown chart ${selectedChart.value}`);
  }
});
const selectedDataSetProjects = computed(() => {
  return selectedDataSet.value.projectIds
    .map((id) => props.projects?.find((p) => p?.id === id))
    .filter(Boolean);
});

const downloadSelectedSetAsCSV = () => {
  const selectedChartName = selectedChart.value;

  const csv = analyticsSetToCSVString(selectedDataSet.value);

  const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });

  const link = document.createElement("a");
  const url = URL.createObjectURL(blob);
  link.setAttribute("href", url);
  link.setAttribute("download", `${selectedChartName}-data.csv`);
  link.style.visibility = "hidden";
  document.body.appendChild(link);

  link.click();
};

const onDownloadSetAsCSV = useClientTry(async () => await downloadSelectedSetAsCSV());
const onToggleColors = () => {
  isUsingProjectColors.value = !isUsingProjectColors.value;
};
</script>

<script lang="ts">
/**
 * @deprecated Use `ranges` instead
 */
const defaultResoloutions: Record<string, number> = {
  "5 minutes": 5,
  "30 minutes": 30,
  "An hour": 60,
  "12 hours": 720,
  "A day": 1440,
  "A week": 10080,
};

type DateRange = { startDate: dayjs.Dayjs; endDate: dayjs.Dayjs };

type RangeObject = {
  getLabel: (dateRange: [dayjs.Dayjs, dayjs.Dayjs]) => string;
  getDates: (currentDate: dayjs.Dayjs) => DateRange;
  // A time resolution in minutes.
  timeResolution: number;
};

const defaultRanges: RangeObject[] = [
  {
    getLabel: () => "Previous 30 minutes",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(30, "minute"),
      endDate: currentDate,
    }),
    timeResolution: 1,
  },
  {
    getLabel: () => "Previous hour",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "hour"),
      endDate: currentDate,
    }),
    timeResolution: 5,
  },
  {
    getLabel: () => "Previous 12 hours",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(12, "hour"),
      endDate: currentDate,
    }),
    timeResolution: 12,
  },
  {
    getLabel: () => "Previous 24 hours",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "day"),
      endDate: currentDate,
    }),
    timeResolution: 30,
  },
  {
    getLabel: () => "Today",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("day"),
      endDate: currentDate,
    }),
    timeResolution: 30,
  },
  {
    getLabel: () => "Yesterday",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "day").startOf("day"),
      endDate: dayjs(currentDate).startOf("day").subtract(1, "second"),
    }),
    timeResolution: 30,
  },
  {
    getLabel: () => "This week",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("week").add(1, "hour"),
      endDate: currentDate,
    }),
    timeResolution: 360,
  },
  {
    getLabel: () => "Last week",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "week").startOf("week").add(1, "hour"),
      endDate: dayjs(currentDate).startOf("week").subtract(1, "second"),
    }),
    timeResolution: 1440,
  },
  {
    getLabel: () => "Previous 7 days",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("day").subtract(7, "day").add(1, "hour"),
      endDate: currentDate.startOf("day"),
    }),
    timeResolution: 720,
  },
  {
    getLabel: () => "This month",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("month").add(1, "hour"),
      endDate: currentDate,
    }),
    timeResolution: 1440,
  },
  {
    getLabel: () => "Last month",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "month").startOf("month").add(1, "hour"),
      endDate: dayjs(currentDate).startOf("month").subtract(1, "second"),
    }),
    timeResolution: 1440,
  },
  {
    getLabel: () => "Previous 30 days",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("day").subtract(30, "day").add(1, "hour"),
      endDate: currentDate.startOf("day"),
    }),
    timeResolution: 1440,
  },
  {
    getLabel: () => "This quarter",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("quarter").add(1, "hour"),
      endDate: currentDate,
    }),
    timeResolution: 1440,
  },
  {
    getLabel: () => "Last quarter",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "quarter").startOf("quarter").add(1, "hour"),
      endDate: dayjs(currentDate).startOf("quarter").subtract(1, "second"),
    }),
    timeResolution: 1440,
  },
  {
    getLabel: () => "This year",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).startOf("year"),
      endDate: currentDate,
    }),
    timeResolution: 20160,
  },
  {
    getLabel: () => "Last year",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "year").startOf("year"),
      endDate: dayjs(currentDate).startOf("year").subtract(1, "second"),
    }),
    timeResolution: 20160,
  },
  {
    getLabel: () => "Previous year",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(1, "year"),
      endDate: dayjs(currentDate),
    }),
    timeResolution: 40320,
  },
  {
    getLabel: () => "Previous two years",
    getDates: (currentDate: dayjs.Dayjs) => ({
      startDate: dayjs(currentDate).subtract(2, "year"),
      endDate: currentDate,
    }),
    timeResolution: 40320,
  },
];
</script>

<style scoped lang="scss">
.chart-controls {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--gap-md);

  .chart-controls__buttons {
    display: flex;
    flex-direction: row;
    gap: var(--gap-xs);

    * {
      width: auto;
      min-height: auto;
    }
  }

  h2 {
    display: flex;
    flex-direction: column;

    .label__subtitle {
      font-size: var(--font-size-sm);
      color: var(--color-text-secondary);
    }
  }
}

.range-dropdown {
  font-size: var(--font-size-sm);
}

.chart-area {
  display: flex;
  flex-direction: row;
  gap: var(--gap-md);

  height: 100%;

  .chart {
    flex-grow: 1;
    flex-shrink: 1;

    display: flex;
    flex-direction: column;
    gap: var(--gap-md);
  }

  .legend {
    margin-top: 24px;
    overflow: hidden;

    max-width: 26ch;
    width: fit-content;

    .legend__items {
      display: flex;
      flex-direction: column;
      gap: var(--gap-xs);

      .legend__item {
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: var(--gap-xs);
        font-size: var(--font-size-sm);
        width: 100%;

        .legend__item__text {
          white-space: nowrap;
          text-overflow: ellipsis;
        }

        .legend__item__color {
          height: var(--font-size-xs);
          width: var(--font-size-xs);
          border-radius: var(--radius-sm);
          background-color: var(--color-brand);

          flex-grow: 0;
          flex-shrink: 0;
        }
      }
    }
  }
}

.btn-transparent {
  background-color: transparent;
  border: none;
  cursor: pointer;

  color: var(--text-color);
  font-weight: var(--font-weight-regular);
}

.btn-dimmed {
  opacity: 0.5;
}

.chart-button-base {
  overflow: hidden;
}

.chart-button-base__selected {
  color: var(--color-contrast);
  background-color: var(--color-brand-highlight);
  box-shadow:
    inset 0 0 0 transparent,
    0 0 0 2px var(--color-brand);

  &:hover {
    background-color: var(--color-brand-highlight);
  }
}

.graphs {
  // Pages clip so we need to add a margin
  margin-left: 0.25rem;
  margin-top: 0.25rem;

  display: flex;
  flex-direction: column;

  .graphs__vertical-bar {
    flex-grow: 0;
    flex-shrink: 0;
    gap: 0.75rem;
    display: flex;
    margin-right: 0.1rem;
  }
}

.country-flag-container {
  width: 40px;
  height: 27px;

  display: flex;
  justify-content: center;
  align-items: center;

  overflow: hidden;

  border: 1px solid var(--color-divider);
  border-radius: var(--radius-xs);
}

.country-flag {
  object-fit: cover;

  min-width: 100%;
  min-height: 100%;
}

.spark-data {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--gap-md);
}

.country-data {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--gap-md);
}

.country-values {
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg);
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-button-bg);
  gap: var(--gap-md);
  padding: var(--gap-md);
  margin-top: var(--gap-md);
  overflow-y: auto;
  max-height: 24rem;
}

.country-value {
  display: grid;
  grid-template-areas: "flag text bar";
  grid-template-columns: auto 1fr 10rem;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: var(--gap-sm);

  .country-text {
    grid-area: text;
    display: flex;
    flex-direction: column;
    gap: var(--gap-xs);
  }

  .percentage-bar {
    grid-area: bar;
    width: 100%;
    height: 1rem;
    background-color: var(--color-raised-bg);
    border: 1px solid var(--color-button-bg);
    border-radius: var(--radius-sm);
    overflow: hidden;

    span {
      display: block;
      height: 100%;
    }
  }
}

@media (max-width: 768px) {
  .chart-area {
    flex-direction: column;
    gap: var(--gap-md);
  }

  .chart-controls {
    flex-direction: column;
    gap: var(--gap-md);
  }

  .chart {
    flex-direction: column;
    gap: var(--gap-md);
  }

  .legend {
    margin-top: 0px;
    width: 100%;
    max-width: 100%;
  }

  .graphs {
    margin-left: 0px;
    margin-top: 0px;

    .graphs__vertical-bar {
      flex-direction: column;
      gap: 0;
      margin-right: 0px;
    }
  }

  .country-data {
    display: block;
  }

  .country-value {
    grid-template-columns: auto 1fr 5rem;
  }
}
</style>
