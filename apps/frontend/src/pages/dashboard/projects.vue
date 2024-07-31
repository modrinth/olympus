<template>
  <div>
    <Modal ref="editLinksModal" header="Edit links">
      <div class="universal-modal links-modal">
        <p>
          Any links you specify below will be overwritten on each of the selected projects. Any you
          leave blank will be ignored. You can clear a link from all selected projects using the
          trash can button.
        </p>
        <section class="links">
          <label
            for="issue-tracker-input"
            title="A place for users to report bugs, issues, and concerns about your project."
          >
            <span class="label__title">Issue tracker</span>
          </label>
          <div class="input-group shrink-first">
            <input
              id="issue-tracker-input"
              v-model="editLinks.issues.val"
              :disabled="editLinks.issues.clear"
              type="url"
              :placeholder="
                editLinks.issues.clear ? 'Existing link will be cleared' : 'Enter a valid URL'
              "
              maxlength="2048"
            />
            <button
              v-tooltip="'Clear link'"
              aria-label="Clear link"
              class="square-button label-button"
              :data-active="editLinks.issues.clear"
              @click="editLinks.issues.clear = !editLinks.issues.clear"
            >
              <TrashIcon />
            </button>
          </div>
          <label
            for="source-code-input"
            title="A page/repository containing the source code for your project"
          >
            <span class="label__title">Source code</span>
          </label>
          <div class="input-group shrink-first">
            <input
              id="source-code-input"
              v-model="editLinks.source.val"
              :disabled="editLinks.source.clear"
              type="url"
              maxlength="2048"
              :placeholder="
                editLinks.source.clear ? 'Existing link will be cleared' : 'Enter a valid URL'
              "
            />
            <button
              v-tooltip="'Clear link'"
              aria-label="Clear link"
              class="square-button label-button"
              :data-active="editLinks.source.clear"
              @click="editLinks.source.clear = !editLinks.source.clear"
            >
              <TrashIcon />
            </button>
          </div>
          <label
            for="wiki-page-input"
            title="A page containing information, documentation, and help for the project."
          >
            <span class="label__title">Wiki page</span>
          </label>
          <div class="input-group shrink-first">
            <input
              id="wiki-page-input"
              v-model="editLinks.wiki.val"
              :disabled="editLinks.wiki.clear"
              type="url"
              maxlength="2048"
              :placeholder="
                editLinks.wiki.clear ? 'Existing link will be cleared' : 'Enter a valid URL'
              "
            />
            <button
              v-tooltip="'Clear link'"
              aria-label="Clear link"
              class="square-button label-button"
              :data-active="editLinks.wiki.clear"
              @click="editLinks.wiki.clear = !editLinks.wiki.clear"
            >
              <TrashIcon />
            </button>
          </div>
          <label for="discord-invite-input" title="An invitation link to your Discord server.">
            <span class="label__title">Discord invite</span>
          </label>
          <div class="input-group shrink-first">
            <input
              id="discord-invite-input"
              v-model="editLinks.discord.val"
              :disabled="editLinks.discord.clear"
              type="url"
              maxlength="2048"
              :placeholder="
                editLinks.discord.clear
                  ? 'Existing link will be cleared'
                  : 'Enter a valid Discord invite URL'
              "
            />
            <button
              v-tooltip="'Clear link'"
              aria-label="Clear link"
              class="square-button label-button"
              :data-active="editLinks.discord.clear"
              @click="editLinks.discord.clear = !editLinks.discord.clear"
            >
              <TrashIcon />
            </button>
          </div>
        </section>
        <p>
          Changes will be applied to
          <strong>{{ selectedProjects.length }}</strong> project{{
            selectedProjects.length > 1 ? "s" : ""
          }}.
        </p>
        <ul>
          <li
            v-for="project in selectedProjects.slice(
              0,
              editLinks.showAffected ? selectedProjects.length : 3,
            )"
            :key="project.id"
          >
            {{ project.title }}
          </li>
          <li v-if="!editLinks.showAffected && selectedProjects.length > 3">
            <strong>and {{ selectedProjects.length - 3 }} more...</strong>
          </li>
        </ul>
        <Checkbox
          v-if="selectedProjects.length > 3"
          v-model="editLinks.showAffected"
          :label="editLinks.showAffected ? 'Less' : 'More'"
          description="Show all loaders"
          :border="false"
          :collapsing-toggle-style="true"
        />
        <div class="push-right input-group">
          <button class="iconified-button" @click="$refs.editLinksModal.hide()">
            <CrossIcon />
            Cancel
          </button>
          <button class="iconified-button brand-button" @click="bulkEditLinks()">
            <SaveIcon />
            Save changes
          </button>
        </div>
      </div>
    </Modal>
    <ModalCreation ref="modal_creation" />
    <section class="universal-card">
      <div class="header__row">
        <h2 class="header__title text-2xl">Projects</h2>
        <div class="input-group">
          <button class="iconified-button brand-button" @click="$refs.modal_creation.show()">
            <PlusIcon />
            {{ formatMessage(commonMessages.createAProjectButton) }}
          </button>
        </div>
      </div>
      <p v-if="projects.length < 1">
        You don't have any projects yet. Click the green button above to begin.
      </p>
      <template v-else>
        <p>You can edit multiple projects at once by selecting them below.</p>
        <div class="input-group">
          <button
            class="iconified-button"
            :disabled="selectedProjects.length === 0"
            @click="$refs.editLinksModal.show()"
          >
            <EditIcon />
            Edit links
          </button>
          <div class="push-right">
            <div class="labeled-control-row">
              Sort by
              <Multiselect
                v-model="sortBy"
                :searchable="false"
                class="small-select"
                :options="['Name', 'Status', 'Type']"
                :close-on-select="true"
                :show-labels="false"
                :allow-empty="false"
                @update:model-value="projects = updateSort(projects, sortBy, descending)"
              />
              <button
                v-tooltip="descending ? 'Descending' : 'Ascending'"
                class="square-button"
                @click="updateDescending()"
              >
                <DescendingIcon v-if="descending" />
                <AscendingIcon v-else />
              </button>
            </div>
          </div>
        </div>
        <div class="grid-table">
          <div class="grid-table__row grid-table__header">
            <div>
              <Checkbox
                :model-value="selectedProjects === projects"
                @update:model-value="
                  selectedProjects === projects
                    ? (selectedProjects = [])
                    : (selectedProjects = projects)
                "
              />
            </div>
            <div>Icon</div>
            <div>Name</div>
            <div>ID</div>
            <div>Type</div>
            <div>Status</div>
            <div />
          </div>
          <div v-for="project in projects" :key="`project-${project.id}`" class="grid-table__row">
            <div>
              <Checkbox
                :disabled="(project.permissions & EDIT_DETAILS) === EDIT_DETAILS"
                :model-value="selectedProjects.includes(project)"
                @update:model-value="
                  selectedProjects.includes(project)
                    ? (selectedProjects = selectedProjects.filter((it) => it !== project))
                    : selectedProjects.push(project)
                "
              />
            </div>
            <div>
              <nuxt-link
                tabindex="-1"
                :to="`/${$getProjectTypeForUrl(project.project_type, project.loaders)}/${
                  project.slug ? project.slug : project.id
                }`"
              >
                <Avatar
                  :src="project.icon_url"
                  aria-hidden="true"
                  :alt="'Icon for ' + project.title"
                  no-shadow
                />
              </nuxt-link>
            </div>

            <div>
              <span class="project-title">
                <IssuesIcon
                  v-if="project.moderator_message"
                  aria-label="Project has a message from the moderators. View the project to see more."
                />

                <nuxt-link
                  class="hover-link wrap-as-needed"
                  :to="`/${$getProjectTypeForUrl(project.project_type, project.loaders)}/${
                    project.slug ? project.slug : project.id
                  }`"
                >
                  {{ project.title }}
                </nuxt-link>
              </span>
            </div>

            <div>
              <CopyCode :text="project.id" />
            </div>

            <div>
              {{ $formatProjectType($getProjectTypeForUrl(project.project_type, project.loaders)) }}
            </div>

            <div>
              <Badge v-if="project.status" :type="project.status" class="status" />
            </div>

            <div>
              <nuxt-link
                class="square-button"
                :to="`/${$getProjectTypeForUrl(project.project_type, project.loaders)}/${
                  project.slug ? project.slug : project.id
                }/settings`"
              >
                <SettingsIcon />
              </nuxt-link>
            </div>
          </div>
        </div>
      </template>
    </section>
  </div>
</template>

<script>
import { Multiselect } from "vue-multiselect";

import { Badge, Checkbox, Avatar, CopyCode } from "@modrinth/ui";
import Modal from "~/components/ui/Modal.vue";
import ModalCreation from "~/components/ui/ModalCreation.vue";

import SettingsIcon from "~/assets/images/utils/settings.svg?component";
import TrashIcon from "~/assets/images/utils/trash.svg?component";
import IssuesIcon from "~/assets/images/utils/issues.svg?component";
import PlusIcon from "~/assets/images/utils/plus.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import EditIcon from "~/assets/images/utils/edit.svg?component";
import SaveIcon from "~/assets/images/utils/save.svg?component";
import AscendingIcon from "~/assets/images/utils/sort-asc.svg?component";
import DescendingIcon from "~/assets/images/utils/sort-desc.svg?component";

export default defineNuxtComponent({
  components: {
    Avatar,
    Badge,
    SettingsIcon,
    TrashIcon,
    Checkbox,
    IssuesIcon,
    PlusIcon,
    CrossIcon,
    EditIcon,
    SaveIcon,
    Modal,
    ModalCreation,
    Multiselect,
    CopyCode,
    AscendingIcon,
    DescendingIcon,
  },
  async setup() {
    const { formatMessage } = useVIntl();

    const user = await useUser();
    await initUserProjects();
    return { formatMessage, user: ref(user) };
  },
  data() {
    return {
      projects: this.updateSort(this.user.projects, "Name"),
      versions: [],
      selectedProjects: [],
      sortBy: "Name",
      descending: false,
      editLinks: {
        showAffected: false,
        source: {
          val: "",
          clear: false,
        },
        discord: {
          val: "",
          clear: false,
        },
        wiki: {
          val: "",
          clear: false,
        },
        issues: {
          val: "",
          clear: false,
        },
      },
    };
  },
  head: {
    title: "Projects - Modrinth",
  },
  created() {
    this.UPLOAD_VERSION = 1 << 0;
    this.DELETE_VERSION = 1 << 1;
    this.EDIT_DETAILS = 1 << 2;
    this.EDIT_BODY = 1 << 3;
    this.MANAGE_INVITES = 1 << 4;
    this.REMOVE_MEMBER = 1 << 5;
    this.EDIT_MEMBER = 1 << 6;
    this.DELETE_PROJECT = 1 << 7;
  },
  methods: {
    updateDescending() {
      this.descending = !this.descending;
      this.projects = this.updateSort(this.projects, this.sortBy, this.descending);
    },
    updateSort(projects, sort, descending) {
      let sortedArray = projects;
      switch (sort) {
        case "Name":
          sortedArray = projects.slice().sort((a, b) => {
            return a.title.localeCompare(b.title);
          });
          break;
        case "Status":
          sortedArray = projects.slice().sort((a, b) => {
            if (a.status < b.status) {
              return -1;
            }
            if (a.status > b.status) {
              return 1;
            }
            return 0;
          });
          break;
        case "Type":
          sortedArray = projects.slice().sort((a, b) => {
            if (a.project_type < b.project_type) {
              return -1;
            }
            if (a.project_type > b.project_type) {
              return 1;
            }
            return 0;
          });
          break;
        default:
          break;
      }

      if (descending) {
        sortedArray = sortedArray.reverse();
      }

      return sortedArray;
    },
    async bulkEditLinks() {
      try {
        const baseData = {
          issues_url: this.editLinks.issues.clear ? null : this.editLinks.issues.val.trim(),
          source_url: this.editLinks.source.clear ? null : this.editLinks.source.val.trim(),
          wiki_url: this.editLinks.wiki.clear ? null : this.editLinks.wiki.val.trim(),
          discord_url: this.editLinks.discord.clear ? null : this.editLinks.discord.val.trim(),
        };

        if (!baseData.issues_url?.length ?? 1 > 0) {
          delete baseData.issues_url;
        }

        if (!baseData.source_url?.length ?? 1 > 0) {
          delete baseData.source_url;
        }

        if (!baseData.wiki_url?.length ?? 1 > 0) {
          delete baseData.wiki_url;
        }

        if (!baseData.discord_url?.length ?? 1 > 0) {
          delete baseData.discord_url;
        }

        await useBaseFetch(
          `projects?ids=${JSON.stringify(this.selectedProjects.map((x) => x.id))}`,
          {
            method: "PATCH",
            body: baseData,
          },
        );

        this.$refs.editLinksModal.hide();
        this.$notify({
          group: "main",
          title: "Success",
          text: "Bulk edited selected project's links.",
          type: "success",
        });
        this.selectedProjects = [];

        this.editLinks.issues.val = "";
        this.editLinks.source.val = "";
        this.editLinks.wiki.val = "";
        this.editLinks.discord.val = "";
        this.editLinks.issues.clear = false;
        this.editLinks.source.clear = false;
        this.editLinks.wiki.clear = false;
        this.editLinks.discord.clear = false;
      } catch (e) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: e,
          type: "error",
        });
      }
    },
  },
});
</script>
<style lang="scss" scoped>
.grid-table {
  display: grid;
  grid-template-columns:
    min-content min-content minmax(min-content, 2fr)
    minmax(min-content, 1fr) minmax(min-content, 1fr) minmax(min-content, 1fr) min-content;
  border-radius: var(--size-rounded-sm);
  overflow: hidden;
  margin-top: var(--spacing-card-md);
  outline: 1px solid transparent;

  .grid-table__row {
    display: contents;

    > div {
      display: flex;
      flex-direction: column;
      justify-content: center;
      padding: var(--spacing-card-sm);

      // Left edge of table
      &:first-child {
        padding-left: var(--spacing-card-bg);
      }

      // Right edge of table
      &:last-child {
        padding-right: var(--spacing-card-bg);
      }
    }

    &:nth-child(2n + 1) > div {
      background-color: var(--color-table-alternate-row);
    }

    &.grid-table__header > div {
      background-color: var(--color-bg);
      font-weight: bold;
      color: var(--color-text-dark);
      padding-top: var(--spacing-card-bg);
      padding-bottom: var(--spacing-card-bg);
    }
  }

  @media screen and (max-width: 750px) {
    display: flex;
    flex-direction: column;

    .grid-table__row {
      display: grid;
      grid-template: "checkbox icon name type settings" "checkbox icon id status settings";
      grid-template-columns:
        min-content min-content minmax(min-content, 2fr)
        minmax(min-content, 1fr) min-content;

      :nth-child(1) {
        grid-area: checkbox;
      }

      :nth-child(2) {
        grid-area: icon;
      }

      :nth-child(3) {
        grid-area: name;
      }

      :nth-child(4) {
        grid-area: id;
        padding-top: 0;
      }

      :nth-child(5) {
        grid-area: type;
      }

      :nth-child(6) {
        grid-area: status;
        padding-top: 0;
      }

      :nth-child(7) {
        grid-area: settings;
      }
    }

    .grid-table__header {
      grid-template: "checkbox settings";
      grid-template-columns: min-content minmax(min-content, 1fr);

      :nth-child(2),
      :nth-child(3),
      :nth-child(4),
      :nth-child(5),
      :nth-child(6) {
        display: none;
      }
    }
  }

  @media screen and (max-width: 560px) {
    .grid-table__row {
      display: grid;
      grid-template: "checkbox icon name settings" "checkbox icon id settings" "checkbox icon type settings" "checkbox icon status settings";
      grid-template-columns: min-content min-content minmax(min-content, 1fr) min-content;

      :nth-child(5) {
        padding-top: 0;
      }
    }

    .grid-table__header {
      grid-template: "checkbox settings";
      grid-template-columns: min-content minmax(min-content, 1fr);
    }
  }
}

.project-title {
  display: flex;
  flex-direction: row;
  gap: var(--spacing-card-xs);

  svg {
    color: var(--color-orange);
  }
}

.status {
  margin-top: var(--spacing-card-xs);
}

.hover-link:hover {
  text-decoration: underline;
}

.labeled-control-row {
  flex: 1;
  display: flex;
  flex-direction: row;
  min-width: 0;
  align-items: center;
  gap: var(--spacing-card-md);
  white-space: nowrap;
}

.small-select {
  width: -moz-fit-content;
  width: fit-content;
}

.label-button[data-active="true"] {
  --background-color: var(--color-red);
  --text-color: var(--color-brand-inverted);
}

.links-modal {
  .links {
    display: grid;
    gap: var(--spacing-card-sm);
    grid-template-columns: 1fr 2fr;

    .input-group {
      flex-wrap: nowrap;
    }

    @media screen and (max-width: 530px) {
      grid-template-columns: 1fr;
      .input-group {
        flex-wrap: wrap;
      }
    }
  }

  ul {
    margin: 0 0 var(--spacing-card-sm) 0;
  }
}
</style>
