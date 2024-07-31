<template>
  <Chips v-if="false" v-model="viewMode" :items="['open', 'archived']" />
  <ReportInfo
    v-for="report in reports.filter(
      (x) =>
        (moderation || x.reporterUser.id === auth.user.id) &&
        (viewMode === 'open' ? x.open : !x.open),
    )"
    :key="report.id"
    :report="report"
    :thread="report.thread"
    :moderation="moderation"
    raised
    :auth="auth"
    class="universal-card recessed"
  />
  <p v-if="reports.length === 0">You don't have any active reports.</p>
</template>
<script setup>
import { Chips } from "@modrinth/ui";
import ReportInfo from "~/components/ui/report/ReportInfo.vue";
import { addReportMessage } from "~/helpers/threads.js";

defineProps({
  moderation: {
    type: Boolean,
    default: false,
  },
  auth: {
    type: Object,
    required: true,
  },
});

const viewMode = ref("open");
const reports = ref([]);

let { data: rawReports } = await useAsyncData("report", () => useBaseFetch("report"));

rawReports = rawReports.value.map((report) => {
  report.item_id = report.item_id.replace(/"/g, "");
  return report;
});

const reporterUsers = rawReports.map((report) => report.reporter);
const reportedUsers = rawReports
  .filter((report) => report.item_type === "user")
  .map((report) => report.item_id);
const versionReports = rawReports.filter((report) => report.item_type === "version");
const versionIds = [...new Set(versionReports.map((report) => report.item_id))];
const userIds = [...new Set(reporterUsers.concat(reportedUsers))];
const threadIds = [
  ...new Set(rawReports.filter((report) => report.thread_id).map((report) => report.thread_id)),
];

const [{ data: users }, { data: versions }, { data: threads }] = await Promise.all([
  await useAsyncData(`users?ids=${JSON.stringify(userIds)}`, () =>
    useBaseFetch(`users?ids=${encodeURIComponent(JSON.stringify(userIds))}`),
  ),
  await useAsyncData(`versions?ids=${JSON.stringify(versionIds)}`, () =>
    useBaseFetch(`versions?ids=${encodeURIComponent(JSON.stringify(versionIds))}`),
  ),
  await useAsyncData(`threads?ids=${JSON.stringify(threadIds)}`, () =>
    useBaseFetch(`threads?ids=${encodeURIComponent(JSON.stringify(threadIds))}`),
  ),
]);

const reportedProjects = rawReports
  .filter((report) => report.item_type === "project")
  .map((report) => report.item_id);
const versionProjects = versions.value.map((version) => version.project_id);
const projectIds = [...new Set(reportedProjects.concat(versionProjects))];

const { data: projects } = await useAsyncData(`projects?ids=${JSON.stringify(projectIds)}`, () =>
  useBaseFetch(`projects?ids=${encodeURIComponent(JSON.stringify(projectIds))}`),
);

reports.value = rawReports.map((report) => {
  report.reporterUser = users.value.find((user) => user.id === report.reporter);
  if (report.item_type === "user") {
    report.user = users.value.find((user) => user.id === report.item_id);
  } else if (report.item_type === "project") {
    report.project = projects.value.find((project) => project.id === report.item_id);
  } else if (report.item_type === "version") {
    report.version = versions.value.find((version) => version.id === report.item_id);
    report.project = projects.value.find((project) => project.id === report.version.project_id);
  }
  if (report.thread_id) {
    report.thread = addReportMessage(
      threads.value.find((thread) => report.thread_id === thread.id),
      report,
    );
  }
  report.open = true;
  return report;
});
</script>
