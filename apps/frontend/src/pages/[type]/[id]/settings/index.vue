<template>
  <div>
    <ModalConfirm
      ref="modal_confirm"
      title="Are you sure you want to delete this project?"
      description="If you proceed, all versions and any attached data will be removed from our servers. This may break other projects, so be careful."
      :has-to-type="true"
      :confirmation-text="project.title"
      proceed-label="Delete"
      @proceed="deleteProject"
    />
    <section class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Project information</span>
        </h3>
      </div>
      <label for="project-icon">
        <span class="label__title">Icon</span>
      </label>
      <div class="input-group">
        <Avatar
          :src="deletedIcon ? null : previewImage ? previewImage : project.icon_url"
          :alt="project.title"
          size="md"
          class="project__icon"
        />
        <div class="input-stack">
          <FileInput
            id="project-icon"
            :max-size="262144"
            :show-icon="true"
            accept="image/png,image/jpeg,image/gif,image/webp"
            class="choose-image iconified-button"
            prompt="Upload icon"
            :disabled="!hasPermission"
            @change="showPreviewImage"
          >
            <UploadIcon />
          </FileInput>
          <button
            v-if="!deletedIcon && (previewImage || project.icon_url)"
            class="iconified-button"
            :disabled="!hasPermission"
            @click="markIconForDeletion"
          >
            <TrashIcon />
            Remove icon
          </button>
        </div>
      </div>

      <label for="project-name">
        <span class="label__title">Name</span>
      </label>
      <input
        id="project-name"
        v-model="name"
        maxlength="2048"
        type="text"
        :disabled="!hasPermission"
      />

      <label for="project-slug">
        <span class="label__title">URL</span>
      </label>
      <div class="text-input-wrapper">
        <div class="text-input-wrapper__before">
          https://modrinth.com/{{ $getProjectTypeForUrl(project.project_type, project.loaders) }}/
        </div>
        <input
          id="project-slug"
          v-model="slug"
          type="text"
          maxlength="64"
          autocomplete="off"
          :disabled="!hasPermission"
        />
      </div>

      <label for="project-summary">
        <span class="label__title">Summary</span>
      </label>
      <div class="textarea-wrapper summary-input">
        <textarea
          id="project-summary"
          v-model="summary"
          maxlength="256"
          :disabled="!hasPermission"
        />
      </div>
      <template
        v-if="
          project.versions?.length !== 0 &&
          project.project_type !== 'resourcepack' &&
          project.project_type !== 'plugin' &&
          project.project_type !== 'shader' &&
          project.project_type !== 'datapack'
        "
      >
        <div class="adjacent-input">
          <label for="project-env-client">
            <span class="label__title">Client-side</span>
            <span class="label__description">
              Select based on if the
              {{ $formatProjectType(project.project_type).toLowerCase() }} has functionality on the
              client side. Just because a mod works in Singleplayer doesn't mean it has actual
              client-side functionality.
            </span>
          </label>
          <Multiselect
            id="project-env-client"
            v-model="clientSide"
            class="small-multiselect"
            placeholder="Select one"
            :options="sideTypes"
            :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
            :disabled="!hasPermission"
          />
        </div>
        <div class="adjacent-input">
          <label for="project-env-server">
            <span class="label__title">Server-side</span>
            <span class="label__description">
              Select based on if the
              {{ $formatProjectType(project.project_type).toLowerCase() }} has functionality on the
              <strong>logical</strong> server. Remember that Singleplayer contains an integrated
              server.
            </span>
          </label>
          <Multiselect
            id="project-env-server"
            v-model="serverSide"
            class="small-multiselect"
            placeholder="Select one"
            :options="sideTypes"
            :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
            :disabled="!hasPermission"
          />
        </div>
      </template>
      <div class="adjacent-input">
        <label for="project-visibility">
          <span class="label__title">Visibility</span>
          <div class="label__description">
            Listed and archived projects are visible in search. Unlisted projects are published, but
            not visible in search or on user profiles. Private projects are only accessible by
            members of the project.

            <p>If approved by the moderators:</p>
            <ul class="visibility-info">
              <li>
                <CheckIcon
                  v-if="visibility === 'approved' || visibility === 'archived'"
                  class="good"
                />
                <ExitIcon v-else class="bad" />
                {{ hasModifiedVisibility() ? "Will be v" : "V" }}isible in search
              </li>
              <li>
                <ExitIcon
                  v-if="visibility === 'unlisted' || visibility === 'private'"
                  class="bad"
                />
                <CheckIcon v-else class="good" />
                {{ hasModifiedVisibility() ? "Will be v" : "V" }}isible on profile
              </li>
              <li>
                <CheckIcon v-if="visibility !== 'private'" class="good" />
                <IssuesIcon
                  v-else
                  v-tooltip="{
                    content:
                      visibility === 'private'
                        ? 'Only members will be able to view the project.'
                        : '',
                  }"
                  class="warn"
                />
                {{ hasModifiedVisibility() ? "Will be v" : "V" }}isible via URL
              </li>
            </ul>
          </div>
        </label>
        <Multiselect
          id="project-visibility"
          v-model="visibility"
          class="small-multiselect"
          placeholder="Select one"
          :options="tags.approvedStatuses"
          :custom-label="(value) => $formatProjectStatus(value)"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
          :disabled="!hasPermission"
        />
      </div>
      <div class="button-group">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="!hasChanges"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </section>

    <section class="universal-card">
      <div class="label">
        <h3>
          <span class="label__title size-card-header">Delete project</span>
        </h3>
      </div>
      <p>
        Removes your project from Modrinth's servers and search. Clicking on this will delete your
        project, so be extra careful!
      </p>
      <button
        type="button"
        class="iconified-button danger-button"
        :disabled="!hasDeletePermission"
        @click="$refs.modal_confirm.show()"
      >
        <TrashIcon />
        Delete project
      </button>
    </section>
  </div>
</template>

<script setup>
import { Multiselect } from "vue-multiselect";

import { Avatar, FileInput } from "@modrinth/ui";
import ModalConfirm from "~/components/ui/ModalConfirm.vue";

import UploadIcon from "~/assets/images/utils/upload.svg?component";
import SaveIcon from "~/assets/images/utils/save.svg?component";
import TrashIcon from "~/assets/images/utils/trash.svg?component";
import ExitIcon from "~/assets/images/utils/x.svg?component";
import IssuesIcon from "~/assets/images/utils/issues.svg?component";
import CheckIcon from "~/assets/images/utils/check.svg?component";

const props = defineProps({
  project: {
    type: Object,
    required: true,
    default: () => ({}),
  },
  currentMember: {
    type: Object,
    required: true,
    default: () => ({}),
  },
  patchProject: {
    type: Function,
    required: true,
    default: () => {},
  },
  patchIcon: {
    type: Function,
    required: true,
    default: () => {},
  },
  resetProject: {
    type: Function,
    required: true,
    default: () => {},
  },
});

const tags = useTags();
const router = useNativeRouter();

const name = ref(props.project.title);
const slug = ref(props.project.slug);
const summary = ref(props.project.description);
const icon = ref(null);
const previewImage = ref(null);
const clientSide = ref(props.project.client_side);
const serverSide = ref(props.project.server_side);
const deletedIcon = ref(false);
const visibility = ref(
  tags.value.approvedStatuses.includes(props.project.status)
    ? props.project.status
    : props.project.requested_status,
);

const hasPermission = computed(() => {
  const EDIT_DETAILS = 1 << 2;
  return (props.currentMember.permissions & EDIT_DETAILS) === EDIT_DETAILS;
});

const hasDeletePermission = computed(() => {
  const DELETE_PROJECT = 1 << 7;
  return (props.currentMember.permissions & DELETE_PROJECT) === DELETE_PROJECT;
});

const sideTypes = ["required", "optional", "unsupported"];

const patchData = computed(() => {
  const data = {};

  if (name.value !== props.project.title) {
    data.title = name.value.trim();
  }
  if (slug.value !== props.project.slug) {
    data.slug = slug.value.trim();
  }
  if (summary.value !== props.project.description) {
    data.description = summary.value.trim();
  }
  if (clientSide.value !== props.project.client_side) {
    data.client_side = clientSide.value;
  }
  if (serverSide.value !== props.project.server_side) {
    data.server_side = serverSide.value;
  }
  if (tags.value.approvedStatuses.includes(props.project.status)) {
    if (visibility.value !== props.project.status) {
      data.status = visibility.value;
    }
  } else if (visibility.value !== props.project.requested_status) {
    data.requested_status = visibility.value;
  }

  return data;
});

const hasChanges = computed(() => {
  return Object.keys(patchData.value).length > 0 || deletedIcon.value || icon.value;
});

const hasModifiedVisibility = () => {
  const originalVisibility = tags.value.approvedStatuses.includes(props.project.status)
    ? props.project.status
    : props.project.requested_status;

  return originalVisibility !== visibility.value;
};

const saveChanges = async () => {
  if (hasChanges.value) {
    await props.patchProject(patchData.value);
  }

  if (deletedIcon.value) {
    await deleteIcon();
    deletedIcon.value = false;
  } else if (icon.value) {
    await props.patchIcon(icon.value);
    icon.value = null;
  }
};

const showPreviewImage = (files) => {
  const reader = new FileReader();
  icon.value = files[0];
  deletedIcon.value = false;
  reader.readAsDataURL(icon.value);
  reader.onload = (event) => {
    previewImage.value = event.target.result;
  };
};

const deleteProject = async () => {
  await useBaseFetch(`project/${props.project.id}`, {
    method: "DELETE",
  });
  await initUserProjects();
  await router.push("/dashboard/projects");
  addNotification({
    group: "main",
    title: "Project deleted",
    text: "Your project has been deleted.",
    type: "success",
  });
};

const markIconForDeletion = () => {
  deletedIcon.value = true;
  icon.value = null;
  previewImage.value = null;
};

const deleteIcon = async () => {
  await useBaseFetch(`project/${props.project.id}/icon`, {
    method: "DELETE",
  });
  await props.resetProject();
  addNotification({
    group: "main",
    title: "Project icon removed",
    text: "Your project's icon has been removed.",
    type: "success",
  });
};
</script>
<style lang="scss" scoped>
.visibility-info {
  padding: 0;
  list-style: none;

  li {
    display: flex;
    align-items: center;
    gap: var(--spacing-card-xs);
  }
}

svg {
  &.good {
    color: var(--color-brand-green);
  }

  &.bad {
    color: var(--color-red);
  }

  &.warn {
    color: var(--color-orange);
  }
}

.summary-input {
  min-height: 8rem;
  max-width: 24rem;
}

.small-multiselect {
  max-width: 15rem;
}
</style>
