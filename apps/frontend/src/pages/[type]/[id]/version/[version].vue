<template>
  <div v-if="version" class="version-page">
    <ModalConfirm
      v-if="currentMember"
      ref="modal_confirm"
      title="Are you sure you want to delete this version?"
      description="This will remove this version forever (like really forever)."
      :has-to-type="false"
      proceed-label="Delete"
      @proceed="deleteVersion()"
    />
    <Modal v-if="auth.user && currentMember" ref="modal_package_mod" header="Package data pack">
      <div class="modal-package-mod universal-labels">
        <div class="markdown-body">
          <p>
            Package your data pack as a mod. This will create a new version with support for the
            selected mod loaders. You will be redirected to the new version and can edit it to your
            liking.
          </p>
        </div>
        <label for="package-mod-loaders">
          <span class="label__title">Mod loaders</span>
          <span class="label__description">
            The mod loaders you would like to package your data pack for.
          </span>
        </label>
        <multiselect
          id="package-mod-loaders"
          v-model="packageLoaders"
          :options="['fabric', 'forge', 'quilt']"
          :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
          :multiple="true"
          :searchable="false"
          :show-no-results="false"
          :show-labels="false"
          placeholder="Choose loaders.."
          open-direction="top"
        />
        <div class="button-group">
          <button class="iconified-button" @click="$refs.modal_package_mod.hide()">
            <CrossIcon />
            Cancel
          </button>
          <button class="iconified-button brand-button" @click="createDataPackVersion">
            <RightArrowIcon />
            Begin packaging data pack
          </button>
        </div>
      </div>
    </Modal>
    <div class="version-page__title universal-card">
      <Breadcrumbs
        :current-title="version.name"
        :link-stack="[
          {
            href: getPreviousLink(),
            label: getPreviousLabel(),
          },
        ]"
      />
      <div class="version-header">
        <template v-if="isEditing">
          <input
            v-model="version.name"
            type="text"
            placeholder="Enter a version title..."
            maxlength="256"
          />
        </template>
        <h2 :class="{ 'sr-only': isEditing }">
          {{ version.name }}
        </h2>
        <div v-if="version.featured" class="featured">
          <StarIcon aria-hidden="true" />
          Featured
        </div>
        <div v-else-if="featuredVersions.find((x) => x.id === version.id)" class="featured">
          <StarIcon aria-hidden="true" />
          Auto-featured
        </div>
      </div>
      <div v-if="fieldErrors && showKnownErrors" class="known-errors">
        <ul>
          <li v-if="version.version_number === ''">Your version must have a version number.</li>
          <li v-if="version.game_versions.length === 0">
            Your version must have the supported Minecraft versions selected.
          </li>
          <li v-if="newFiles.length === 0 && version.files.length === 0 && !replaceFile">
            Your version must have a file uploaded.
          </li>
          <li v-if="version.loaders.length === 0 && project.project_type !== 'resourcepack'">
            Your version must have the supported mod loaders selected.
          </li>
        </ul>
      </div>
      <div v-if="isCreating" class="input-group">
        <button
          class="iconified-button brand-button"
          :disabled="shouldPreventActions"
          @click="createVersion"
        >
          <PlusIcon aria-hidden="true" />
          Create
        </button>
        <nuxt-link
          v-if="auth.user"
          :to="`/${project.project_type}/${project.slug ? project.slug : project.id}/versions`"
          class="iconified-button"
        >
          <CrossIcon aria-hidden="true" />
          Cancel
        </nuxt-link>
      </div>
      <div v-else-if="isEditing" class="input-group">
        <button
          class="iconified-button brand-button"
          :disabled="shouldPreventActions"
          @click="saveEditedVersion"
        >
          <SaveIcon aria-hidden="true" />
          Save
        </button>
        <button class="iconified-button" @click="version.featured = !version.featured">
          <StarIcon aria-hidden="true" />
          <template v-if="!version.featured"> Feature version</template>
          <template v-else> Unfeature version</template>
        </button>
        <nuxt-link
          v-if="currentMember"
          class="action iconified-button"
          :to="`/${project.project_type}/${
            project.slug ? project.slug : project.id
          }/version/${encodeURI(version.displayUrlEnding)}`"
        >
          <CrossIcon aria-hidden="true" />
          Discard changes
        </nuxt-link>
      </div>
      <div v-else class="input-group">
        <a
          v-if="primaryFile"
          v-tooltip="primaryFile.filename + ' (' + $formatBytes(primaryFile.size) + ')'"
          :href="primaryFile.url"
          class="iconified-button brand-button"
          :aria-label="`Download ${primaryFile.filename}`"
        >
          <DownloadIcon aria-hidden="true" />
          Download
        </a>
        <nuxt-link v-if="!auth.user" class="iconified-button" to="/auth/sign-in">
          <ReportIcon aria-hidden="true" />
          Report
        </nuxt-link>
        <button v-else class="iconified-button" @click="() => reportVersion(version.id)">
          <ReportIcon aria-hidden="true" />
          Report
        </button>
        <nuxt-link
          v-if="currentMember"
          class="action iconified-button"
          :to="`/${project.project_type}/${
            project.slug ? project.slug : project.id
          }/version/${encodeURI(version.displayUrlEnding)}/edit`"
        >
          <EditIcon aria-hidden="true" />
          Edit
        </nuxt-link>
        <button
          v-if="
            currentMember &&
            version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x))
          "
          class="iconified-button"
          @click="$refs.modal_package_mod.show()"
        >
          <BoxIcon aria-hidden="true" />
          Package as mod
        </button>
        <button
          v-if="currentMember"
          class="iconified-button danger-button"
          @click="$refs.modal_confirm.show()"
        >
          <TrashIcon aria-hidden="true" />
          Delete
        </button>
      </div>
    </div>
    <div class="version-page__changelog universal-card">
      <h3>Changelog</h3>
      <template v-if="isEditing">
        <div class="changelog-editor-spacing">
          <MarkdownEditor v-model="version.changelog" :on-image-upload="onImageUpload" />
        </div>
      </template>
      <div
        v-else
        class="markdown-body"
        v-html="
          version.changelog ? renderHighlightedString(version.changelog) : 'No changelog specified.'
        "
      />
    </div>
    <div
      v-if="deps.length > 0 || (isEditing && project.project_type !== 'modpack')"
      class="version-page__dependencies universal-card"
    >
      <h3>Dependencies</h3>
      <div
        v-for="(dependency, index) in deps.filter((x) => !x.file_name)"
        :key="index"
        class="dependency"
        :class="{ 'button-transparent': !isEditing }"
        @click="!isEditing ? $router.push(dependency.link) : {}"
      >
        <Avatar
          :src="dependency.project ? dependency.project.icon_url : null"
          alt="dependency-icon"
          size="sm"
        />
        <nuxt-link v-if="!isEditing" :to="dependency.link" class="info">
          <span class="project-title">
            {{ dependency.project ? dependency.project.title : "Unknown Project" }}
          </span>
          <span v-if="dependency.version" class="dep-type" :class="dependency.dependency_type">
            Version {{ dependency.version.version_number }} is
            {{ dependency.dependency_type }}
          </span>
          <span v-else class="dep-type" :class="dependency.dependency_type">
            {{ dependency.dependency_type }}
          </span>
        </nuxt-link>
        <div v-else class="info">
          <span class="project-title">
            {{ dependency.project ? dependency.project.title : "Unknown Project" }}
          </span>
          <span v-if="dependency.version" class="dep-type" :class="dependency.dependency_type">
            Version {{ dependency.version.version_number }} is
            {{ dependency.dependency_type }}
          </span>
          <span v-else class="dep-type" :class="dependency.dependency_type">
            {{ dependency.dependency_type }}
          </span>
        </div>
        <button
          v-if="isEditing && project.project_type !== 'modpack'"
          class="iconified-button"
          @click="version.dependencies.splice(index, 1)"
        >
          <TrashIcon />
          Remove
        </button>
      </div>
      <div
        v-for="(dependency, index) in deps.filter((x) => x.file_name)"
        :key="index"
        class="dependency"
      >
        <Avatar :src="null" alt="dependency-icon" size="sm" />
        <div class="info">
          <span class="project-title">
            {{ dependency.file_name }}
          </span>
          <span class="dep-type" :class="dependency.dependency_type">Added via overrides</span>
        </div>
      </div>
      <div v-if="isEditing && project.project_type !== 'modpack'" class="add-dependency">
        <h4>Add dependency</h4>
        <div class="input-group">
          <Multiselect
            v-model="dependencyAddMode"
            class="input"
            :options="['project', 'version']"
            :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
          />
          <input
            v-model="newDependencyId"
            type="text"
            :placeholder="`Enter the ${dependencyAddMode} ID${
              dependencyAddMode === 'project' ? '/slug' : ''
            }`"
            @keyup.enter="addDependency(dependencyAddMode, newDependencyId, newDependencyType)"
          />
          <Multiselect
            v-model="newDependencyType"
            class="input"
            :options="['required', 'optional', 'incompatible', 'embedded']"
            :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="true"
          />
        </div>
        <div class="input-group">
          <button
            class="iconified-button brand-button"
            @click="addDependency(dependencyAddMode, newDependencyId, newDependencyType)"
          >
            <PlusIcon />
            Add dependency
          </button>
        </div>
      </div>
    </div>
    <div class="version-page__files universal-card">
      <h3>Files</h3>
      <div v-if="isEditing && replaceFile" class="file primary">
        <FileIcon />
        <span class="filename">
          <strong>{{ replaceFile.name }}</strong>
          <span class="file-size">({{ $formatBytes(replaceFile.size) }})</span>
        </span>
        <FileInput
          class="iconified-button raised-button"
          prompt="Replace"
          :accept="acceptFileFromProjectType(project.project_type)"
          :max-size="524288000"
          should-always-reset
          @change="(x) => (replaceFile = x[0])"
        >
          <TransferIcon />
        </FileInput>
      </div>
      <div
        v-for="(file, index) in version.files"
        :key="file.hashes.sha1"
        :class="{
          file: true,
          primary: primaryFile.hashes.sha1 === file.hashes.sha1,
        }"
      >
        <FileIcon />
        <span class="filename">
          <strong>{{ file.filename }}</strong>
          <span class="file-size">({{ $formatBytes(file.size) }})</span>
          <span v-if="primaryFile.hashes.sha1 === file.hashes.sha1" class="file-type">
            Primary
          </span>
          <span
            v-else-if="file.file_type === 'required-resource-pack' && !isEditing"
            class="file-type"
          >
            Required resource pack
          </span>
          <span
            v-else-if="file.file_type === 'optional-resource-pack' && !isEditing"
            class="file-type"
          >
            Optional resource pack
          </span>
        </span>
        <multiselect
          v-if="
            version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x)) &&
            isEditing &&
            primaryFile.hashes.sha1 !== file.hashes.sha1
          "
          v-model="oldFileTypes[index]"
          class="raised-multiselect"
          placeholder="Select file type"
          :options="fileTypes"
          track-by="value"
          label="display"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
        />
        <button
          v-if="isEditing"
          :disabled="primaryFile.hashes.sha1 === file.hashes.sha1"
          class="iconified-button raised-button"
          @click="
            () => {
              deleteFiles.push(file.hashes.sha1);
              version.files.splice(index, 1);
              oldFileTypes.splice(index, 1);
            }
          "
        >
          <TrashIcon />
          Remove
        </button>
        <a
          v-else
          :href="file.url"
          class="iconified-button raised-button"
          :title="`Download ${file.filename}`"
          tabindex="0"
        >
          <DownloadIcon />
          Download
        </a>
      </div>
      <template v-if="isEditing">
        <div v-for="(file, index) in newFiles" :key="index" class="file">
          <FileIcon />
          <span class="filename">
            <strong>{{ file.name }}</strong>
            <span class="file-size">({{ $formatBytes(file.size) }})</span>
          </span>
          <multiselect
            v-if="version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x))"
            v-model="newFileTypes[index]"
            class="raised-multiselect"
            placeholder="Select file type"
            :options="fileTypes"
            track-by="value"
            label="display"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
          />
          <button
            class="iconified-button raised-button"
            @click="
              () => {
                newFiles.splice(index, 1);
                newFileTypes.splice(index, 1);
              }
            "
          >
            <TrashIcon />
            Remove
          </button>
        </div>
        <div class="additional-files">
          <h4>Upload additional files</h4>
          <span v-if="version.loaders.some((x) => tags.loaderData.dataPackLoaders.includes(x))">
            Used for additional files such as required/optional resource packs
          </span>
          <span v-else>Used for files such as sources or Javadocs.</span>
          <FileInput
            prompt="Drag and drop to upload or click to select"
            multiple
            long-style
            :accept="acceptFileFromProjectType(project.project_type)"
            :max-size="524288000"
            @change="
              (x) =>
                x.forEach((y) => {
                  newFiles.push(y);
                  newFileTypes.push(null);
                })
            "
          >
            <UploadIcon />
          </FileInput>
        </div>
      </template>
    </div>
    <div class="version-page__metadata">
      <div class="universal-card full-width-inputs">
        <h3>Metadata</h3>
        <div>
          <h4>Release channel</h4>
          <Multiselect
            v-if="isEditing"
            v-model="version.version_type"
            class="input"
            placeholder="Select one"
            :options="['release', 'beta', 'alpha']"
            :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
          />
          <template v-else>
            <Badge
              v-if="version.version_type === 'release'"
              class="value"
              type="release"
              color="green"
            />
            <Badge
              v-else-if="version.version_type === 'beta'"
              class="value"
              type="beta"
              color="orange"
            />
            <Badge
              v-else-if="version.version_type === 'alpha'"
              class="value"
              type="alpha"
              color="red"
            />
          </template>
        </div>
        <div>
          <h4>Version number</h4>
          <div v-if="isEditing" class="iconified-input">
            <label class="hidden" for="version-number">Version number</label>
            <HashIcon aria-hidden="true" />
            <input
              id="version-number"
              v-model="version.version_number"
              type="text"
              autocomplete="off"
              maxlength="54"
            />
          </div>
          <span v-else>{{ version.version_number }}</span>
        </div>
        <div v-if="project.project_type !== 'resourcepack'">
          <h4>Loaders</h4>
          <Multiselect
            v-if="isEditing"
            v-model="version.loaders"
            :options="
              tags.loaders
                .filter((x) =>
                  x.supported_project_types.includes(project.actualProjectType.toLowerCase()),
                )
                .map((it) => it.name)
            "
            :custom-label="(value) => $formatCategory(value)"
            :loading="tags.loaders.length === 0"
            :multiple="true"
            :searchable="true"
            :show-no-results="false"
            :close-on-select="false"
            :clear-on-select="false"
            :show-labels="false"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose loaders..."
          />
          <Categories v-else :categories="version.loaders" :type="project.actualProjectType" />
        </div>
        <div>
          <h4>Game versions</h4>
          <template v-if="isEditing">
            <multiselect
              v-model="version.game_versions"
              :options="
                showSnapshots
                  ? tags.gameVersions.map((x) => x.version)
                  : tags.gameVersions
                      .filter((it) => it.version_type === 'release')
                      .map((x) => x.version)
              "
              :loading="tags.gameVersions.length === 0"
              :multiple="true"
              :searchable="true"
              :show-no-results="false"
              :close-on-select="false"
              :clear-on-select="false"
              :show-labels="false"
              :limit="6"
              :hide-selected="true"
              placeholder="Choose versions..."
            />
            <Checkbox
              v-model="showSnapshots"
              label="Show all versions"
              description="Show all versions"
              style="margin-top: 0.5rem"
              :border="false"
            />
          </template>
          <span v-else>{{ $formatVersion(version.game_versions) }}</span>
        </div>
        <div v-if="!isEditing">
          <h4>Downloads</h4>
          <span>{{ version.downloads }}</span>
        </div>
        <div v-if="!isEditing">
          <h4>Publication date</h4>
          <span>
            {{ $dayjs(version.date_published).format("MMMM D, YYYY [at] h:mm A") }}
          </span>
        </div>
        <div v-if="!isEditing && version.author">
          <h4>Publisher</h4>
          <div
            class="team-member columns button-transparent"
            @click="$router.push('/user/' + version.author.user.username)"
          >
            <Avatar
              :src="version.author.avatar_url"
              :alt="version.author.user.username"
              size="sm"
              circle
            />

            <div class="member-info">
              <nuxt-link :to="'/user/' + version.author.user.username" class="name">
                <p>
                  {{ version.author.name }}
                </p>
              </nuxt-link>
              <p v-if="version.author.role" class="role">
                {{ version.author.role }}
              </p>
              <p v-else-if="version.author_id === 'GVFjtWTf'" class="role">Archivist</p>
            </div>
          </div>
        </div>
        <div v-if="!isEditing">
          <h4>Version ID</h4>
          <CopyCode :text="version.id" />
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import { MarkdownEditor, Avatar, Badge, CopyCode, Chips, Checkbox, FileInput } from "@modrinth/ui";
import { Multiselect } from "vue-multiselect";
import { acceptFileFromProjectType } from "~/helpers/fileUtils.js";
import { inferVersionInfo } from "~/helpers/infer.js";
import { createDataPackVersion } from "~/helpers/package.js";
import { renderHighlightedString } from "~/helpers/highlight.js";
import { reportVersion } from "~/utils/report-helpers.ts";
import { useImageUpload } from "~/composables/image-upload.ts";

import Breadcrumbs from "~/components/ui/Breadcrumbs.vue";
import Categories from "~/components/ui/search/Categories.vue";
import ModalConfirm from "~/components/ui/ModalConfirm.vue";

import FileIcon from "~/assets/images/utils/file.svg?component";
import TrashIcon from "~/assets/images/utils/trash.svg?component";
import EditIcon from "~/assets/images/utils/edit.svg?component";
import DownloadIcon from "~/assets/images/utils/download.svg?component";
import StarIcon from "~/assets/images/utils/star.svg?component";
import ReportIcon from "~/assets/images/utils/report.svg?component";
import SaveIcon from "~/assets/images/utils/save.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import HashIcon from "~/assets/images/utils/hash.svg?component";
import PlusIcon from "~/assets/images/utils/plus.svg?component";
import TransferIcon from "~/assets/images/utils/transfer.svg?component";
import UploadIcon from "~/assets/images/utils/upload.svg?component";
import BackIcon from "~/assets/images/utils/left-arrow.svg?component";
import BoxIcon from "~/assets/images/utils/box.svg?component";
import RightArrowIcon from "~/assets/images/utils/right-arrow.svg?component";
import Modal from "~/components/ui/Modal.vue";
import ChevronRightIcon from "~/assets/images/utils/chevron-right.svg?component";

export default defineNuxtComponent({
  components: {
    MarkdownEditor,
    Modal,
    FileInput,
    Checkbox,
    ChevronRightIcon,
    Chips,
    Categories,
    DownloadIcon,
    EditIcon,
    TrashIcon,
    StarIcon,
    FileIcon,
    ReportIcon,
    SaveIcon,
    CrossIcon,
    HashIcon,
    PlusIcon,
    TransferIcon,
    UploadIcon,
    BackIcon,
    Avatar,
    Badge,
    Breadcrumbs,
    CopyCode,
    ModalConfirm,
    Multiselect,
    BoxIcon,
    RightArrowIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {};
      },
    },
    versions: {
      type: Array,
      default() {
        return [];
      },
    },
    featuredVersions: {
      type: Array,
      default() {
        return [];
      },
    },
    members: {
      type: Array,
      default() {
        return [{}];
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null;
      },
    },
    dependencies: {
      type: Object,
      default() {
        return {};
      },
    },
    resetProject: {
      type: Function,
      required: true,
      default: () => {},
    },
  },
  async setup(props) {
    const data = useNuxtApp();
    const route = useNativeRoute();

    const auth = await useAuth();
    const tags = useTags();

    const path = route.name.split("-");
    const mode = path[path.length - 1];

    const fileTypes = [
      {
        display: "Required resource pack",
        value: "required-resource-pack",
      },
      {
        display: "Optional resource pack",
        value: "optional-resource-pack",
      },
    ];
    let oldFileTypes = [];

    let isCreating = false;
    let isEditing = false;

    let version = {};
    let primaryFile = {};
    let alternateFile = {};

    let replaceFile = null;

    if (mode === "edit") {
      isEditing = true;
    }

    if (route.params.version === "create") {
      isCreating = true;
      isEditing = true;

      version = {
        id: "none",
        project_id: props.project.id,
        author_id: props.currentMember.user.id,
        name: "",
        version_number: "",
        changelog: "",
        date_published: Date.now(),
        downloads: 0,
        version_type: "release",
        files: [],
        dependencies: [],
        game_versions: [],
        loaders: [],
        featured: false,
      };
      // For navigation from versions page / upload file prompt
      if (import.meta.client && history.state && history.state.newPrimaryFile) {
        replaceFile = history.state.newPrimaryFile;

        try {
          const inferredData = await inferVersionInfo(
            replaceFile,
            props.project,
            tags.value.gameVersions,
          );

          version = {
            ...version,
            ...inferredData,
          };
        } catch (err) {
          console.error("Error parsing version file data", err);
        }
      }
    } else if (route.params.version === "latest") {
      let versionList = props.versions;
      if (route.query.loader) {
        versionList = versionList.filter((x) => x.loaders.includes(route.query.loader));
      }
      if (route.query.version) {
        versionList = versionList.filter((x) => x.game_versions.includes(route.query.version));
      }
      version = versionList.reduce((a, b) => (a.date_published > b.date_published ? a : b));
    } else {
      version = props.versions.find((x) => x.id === route.params.version);

      if (!version) {
        version = props.versions.find((x) => x.displayUrlEnding === route.params.version);
      }
    }

    if (!version) {
      throw createError({
        fatal: true,
        statusCode: 404,
        message: "Version not found",
      });
    }

    version = JSON.parse(JSON.stringify(version));
    primaryFile = version.files.find((file) => file.primary) ?? version.files[0];
    alternateFile = version.files.find(
      (file) => file.file_type && file.file_type.includes("resource-pack"),
    );

    for (const dependency of version.dependencies) {
      dependency.version = props.dependencies.versions.find((x) => x.id === dependency.version_id);

      if (dependency.version) {
        dependency.project = props.dependencies.projects.find(
          (x) => x.id === dependency.version.project_id,
        );
      }

      if (!dependency.project) {
        dependency.project = props.dependencies.projects.find(
          (x) => x.id === dependency.project_id,
        );
      }

      dependency.link = dependency.project
        ? `/${dependency.project.project_type}/${dependency.project.slug ?? dependency.project.id}${
            dependency.version ? `/version/${encodeURI(dependency.version.version_number)}` : ""
          }`
        : "";
    }

    oldFileTypes = version.files.map((x) => fileTypes.find((y) => y.value === x.file_type));

    const title = computed(
      () => `${isCreating ? "Create Version" : version.name} - ${props.project.title}`,
    );
    const description = computed(
      () =>
        `Download ${props.project.title} ${
          version.version_number
        } on Modrinth. Supports ${data.$formatVersion(version.game_versions)} ${version.loaders
          .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
          .join(" & ")}. Published on ${data
          .$dayjs(version.date_published)
          .format("MMM D, YYYY")}. ${version.downloads} downloads.`,
    );

    useSeoMeta({
      title,
      description,
      ogTitle: title,
      ogDescription: description,
    });

    return {
      auth,
      tags,
      fileTypes: ref(fileTypes),
      oldFileTypes: ref(oldFileTypes),
      isCreating: ref(isCreating),
      isEditing: ref(isEditing),
      version: ref(version),
      primaryFile: ref(primaryFile),
      alternateFile: ref(alternateFile),
      replaceFile: ref(replaceFile),
      uploadedImageIds: ref([]),
    };
  },
  data() {
    return {
      dependencyAddMode: "project",
      newDependencyType: "required",
      newDependencyId: "",

      showSnapshots: false,

      newFiles: [],
      deleteFiles: [],
      replaceFile: null,

      newFileTypes: [],

      packageLoaders: ["forge", "fabric", "quilt"],

      showKnownErrors: false,
      shouldPreventActions: false,
    };
  },
  computed: {
    fieldErrors() {
      return (
        this.version.version_number === "" ||
        this.version.game_versions.length === 0 ||
        (this.version.loaders.length === 0 && this.project.project_type !== "resourcepack") ||
        (this.newFiles.length === 0 && this.version.files.length === 0 && !this.replaceFile)
      );
    },
    deps() {
      const order = ["required", "optional", "incompatible", "embedded"];
      return [...this.version.dependencies].sort(
        (a, b) => order.indexOf(a.dependency_type) - order.indexOf(b.dependency_type),
      );
    },
  },
  watch: {
    "$route.path"() {
      const path = this.$route.name.split("-");
      const mode = path[path.length - 1];

      this.isEditing = mode === "edit" || this.$route.params.version === "create";
    },
  },
  methods: {
    async onImageUpload(file) {
      const response = await useImageUpload(file, { context: "version" });

      this.uploadedImageIds.push(response.id);
      this.uploadedImageIds = this.uploadedImageIds.slice(-10);

      return response.url;
    },
    getPreviousLink() {
      if (this.$router.options.history.state.back) {
        if (
          this.$router.options.history.state.back.includes("/changelog") ||
          this.$router.options.history.state.back.includes("/versions")
        ) {
          return this.$router.options.history.state.back;
        }
      }
      return `/${this.project.project_type}/${
        this.project.slug ? this.project.slug : this.project.id
      }/versions`;
    },
    getPreviousLabel() {
      return this.$router.options.history.state.back &&
        this.$router.options.history.state.back.endsWith("/changelog")
        ? "Changelog"
        : "Versions";
    },
    acceptFileFromProjectType,
    renderHighlightedString,
    async addDependency(dependencyAddMode, newDependencyId, newDependencyType, hideErrors) {
      try {
        if (dependencyAddMode === "project") {
          const project = await useBaseFetch(`project/${newDependencyId}`);

          if (this.version.dependencies.some((dep) => project.id === dep.project_id)) {
            this.$notify({
              group: "main",
              title: "Dependency already added",
              text: "You cannot add the same dependency twice.",
              type: "error",
            });
          } else {
            this.version.dependencies.push({
              project,
              project_id: project.id,
              dependency_type: newDependencyType,
              link: `/${project.project_type}/${project.slug ?? project.id}`,
            });

            this.$emit("update:dependencies", {
              projects: this.dependencies.projects.concat([project]),
              versions: this.dependencies.versions,
            });
          }
        } else if (dependencyAddMode === "version") {
          const version = await useBaseFetch(`version/${this.newDependencyId}`);

          const project = await useBaseFetch(`project/${version.project_id}`);

          if (this.version.dependencies.some((dep) => version.id === dep.version_id)) {
            this.$notify({
              group: "main",
              title: "Dependency already added",
              text: "You cannot add the same dependency twice.",
              type: "error",
            });
          } else {
            this.version.dependencies.push({
              version,
              project,
              version_id: version.id,
              project_id: project.id,
              dependency_type: this.newDependencyType,
              link: `/${project.project_type}/${project.slug ?? project.id}/version/${encodeURI(
                version.version_number,
              )}`,
            });

            this.$emit("update:dependencies", {
              projects: this.dependencies.projects.concat([project]),
              versions: this.dependencies.versions.concat([version]),
            });
          }
        }

        this.newDependencyId = "";
      } catch {
        if (!hideErrors) {
          this.$notify({
            group: "main",
            title: "Invalid Dependency",
            text: "The specified dependency could not be found",
            type: "error",
          });
        }
      }
    },
    async saveEditedVersion() {
      startLoading();

      if (this.fieldErrors) {
        this.showKnownErrors = true;

        stopLoading();
        return;
      }

      try {
        if (this.newFiles.length > 0) {
          const formData = new FormData();
          const fileParts = this.newFiles.map((f, idx) => `${f.name}-${idx}`);

          formData.append(
            "data",
            JSON.stringify({
              file_types: this.newFileTypes.reduce(
                (acc, x, i) => ({
                  ...acc,
                  [fileParts[i]]: x ? x.value : null,
                }),
                {},
              ),
            }),
          );

          for (let i = 0; i < this.newFiles.length; i++) {
            formData.append(fileParts[i], new Blob([this.newFiles[i]]), this.newFiles[i].name);
          }

          await useBaseFetch(`version/${this.version.id}/file`, {
            method: "POST",
            body: formData,
            headers: {
              "Content-Disposition": formData,
            },
          });
        }

        const body = {
          name: this.version.name || this.version.version_number,
          version_number: this.version.version_number,
          changelog: this.version.changelog,
          version_type: this.version.version_type,
          dependencies: this.version.dependencies,
          game_versions: this.version.game_versions,
          loaders: this.version.loaders,
          primary_file: ["sha1", this.primaryFile.hashes.sha1],
          featured: this.version.featured,
          file_types: this.oldFileTypes.map((x, i) => {
            return {
              algorithm: "sha1",
              hash: this.version.files[i].hashes.sha1,
              file_type: x ? x.value : null,
            };
          }),
        };

        if (this.project.project_type === "modpack") {
          delete body.dependencies;
        }

        await useBaseFetch(`version/${this.version.id}`, {
          method: "PATCH",
          body,
        });

        for (const hash of this.deleteFiles) {
          await useBaseFetch(`version_file/${hash}?version_id=${this.version.id}`, {
            method: "DELETE",
          });
        }

        await this.resetProjectVersions();

        await this.$router.replace(
          `/${this.project.project_type}/${
            this.project.slug ? this.project.slug : this.project.id
          }/version/${encodeURI(
            this.versions.find((x) => x.id === this.version.id).displayUrlEnding,
          )}`,
        );
      } catch (err) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: err.data.description,
          type: "error",
        });
        window.scrollTo({ top: 0, behavior: "smooth" });
      }
      stopLoading();
    },
    reportVersion,
    async createVersion() {
      this.shouldPreventActions = true;
      startLoading();
      if (this.fieldErrors) {
        this.showKnownErrors = true;
        this.shouldPreventActions = false;

        stopLoading();
        return;
      }

      try {
        await this.createVersionRaw(this.version);
      } catch (err) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: err.data ? err.data.description : err,
          type: "error",
        });
        window.scrollTo({ top: 0, behavior: "smooth" });
      }

      stopLoading();
      this.shouldPreventActions = false;
    },
    async createVersionRaw(version) {
      const formData = new FormData();

      const fileParts = this.newFiles.map((f, idx) => `${f.name}-${idx}`);
      if (this.replaceFile) {
        fileParts.unshift(this.replaceFile.name.concat("-primary"));
      }

      if (this.project.project_type === "resourcepack") {
        version.loaders = ["minecraft"];
      }

      const newVersion = {
        project_id: version.project_id,
        file_parts: fileParts,
        version_number: version.version_number,
        version_title: version.name || version.version_number,
        version_body: version.changelog,
        dependencies: version.dependencies,
        game_versions: version.game_versions,
        loaders: version.loaders,
        release_channel: version.version_type,
        featured: version.featured,
        file_types: this.newFileTypes.reduce(
          (acc, x, i) => ({
            ...acc,
            [fileParts[this.replaceFile ? i + 1 : i]]: x ? x.value : null,
          }),
          {},
        ),
      };

      formData.append("data", JSON.stringify(newVersion));

      if (this.replaceFile) {
        formData.append(
          this.replaceFile.name.concat("-primary"),
          new Blob([this.replaceFile]),
          this.replaceFile.name,
        );
      }

      for (let i = 0; i < this.newFiles.length; i++) {
        formData.append(
          fileParts[this.replaceFile ? i + 1 : i],
          new Blob([this.newFiles[i]]),
          this.newFiles[i].name,
        );
      }

      const data = await useBaseFetch("version", {
        method: "POST",
        body: formData,
        headers: {
          "Content-Disposition": formData,
        },
      });

      await this.resetProjectVersions();

      await this.$router.push(
        `/${this.project.project_type}/${
          this.project.slug ? this.project.slug : this.project.project_id
        }/version/${data.id}`,
      );
    },
    async deleteVersion() {
      startLoading();

      await useBaseFetch(`version/${this.version.id}`, {
        method: "DELETE",
      });

      await this.resetProjectVersions();
      await this.$router.replace(`/${this.project.project_type}/${this.project.id}/versions`);
      stopLoading();
    },
    async createDataPackVersion() {
      this.shouldPreventActions = true;
      startLoading();
      try {
        const blob = await createDataPackVersion(
          this.project,
          this.version,
          this.primaryFile,
          this.members,
          this.tags.gameVersions,
          this.packageLoaders,
        );

        this.newFiles = [];
        this.newFileTypes = [];
        this.replaceFile = new File(
          [blob],
          `${this.project.slug}-${this.version.version_number}.jar`,
        );

        await this.createVersionRaw({
          project_id: this.project.id,
          author_id: this.currentMember.user.id,
          name: this.version.name,
          version_number: `${this.version.version_number}+mod`,
          changelog: this.version.changelog,
          version_type: this.version.version_type,
          dependencies: this.version.dependencies,
          game_versions: this.version.game_versions,
          loaders: this.packageLoaders,
          featured: this.version.featured,
        });

        this.$refs.modal_package_mod.hide();

        this.$notify({
          group: "main",
          title: "Packaging Success",
          text: "Your data pack was successfully packaged as a mod! Make sure to playtest to check for errors.",
          type: "success",
        });
      } catch (err) {
        this.$notify({
          group: "main",
          title: "An error occurred",
          text: err.data ? err.data.description : err,
          type: "error",
        });
      }
      stopLoading();
      this.shouldPreventActions = false;
    },
    async resetProjectVersions() {
      const [versions, featuredVersions, dependencies] = await Promise.all([
        useBaseFetch(`project/${this.version.project_id}/version`),
        useBaseFetch(`project/${this.version.project_id}/version?featured=true`),
        useBaseFetch(`project/${this.version.project_id}/dependencies`),
        this.resetProject(),
      ]);

      const newCreatedVersions = this.$computeVersions(versions, this.members);
      const featuredIds = featuredVersions.map((x) => x.id);
      this.$emit("update:versions", newCreatedVersions);
      this.$emit(
        "update:featuredVersions",
        newCreatedVersions.filter((version) => featuredIds.includes(version.id)),
      );
      this.$emit("update:dependencies", dependencies);

      return newCreatedVersions;
    },
  },
});
</script>

<style lang="scss" scoped>
.changelog-editor-spacing {
  padding-block: var(--gap-md);
}

.version-page {
  display: grid;

  grid-template:
    "title" auto
    "changelog" auto
    "dependencies" auto
    "metadata" auto
    "files" auto
    / 1fr;

  column-gap: var(--spacing-card-md);

  .version-page__title {
    grid-area: title;

    .version-header {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      margin-bottom: 1rem;
      gap: var(--spacing-card-md);

      h2,
      input[type="text"] {
        margin: 0;
        font-size: var(--font-size-2xl);
        font-weight: bold;
      }

      input[type="text"] {
        max-width: 100%;
        min-width: 0;
        flex-grow: 1;
        width: 2rem;
      }

      .featured {
        display: flex;
        align-items: center;
        gap: var(--spacing-card-xs);

        svg {
          height: 1.45rem;
        }
      }
    }

    .known-errors {
      margin-bottom: 1rem;
    }
  }

  h3 {
    font-size: var(--font-size-lg);
    margin: 0 0 0.5rem 0;
  }

  .version-page__changelog {
    grid-area: changelog;
    overflow-x: hidden;
  }

  .version-page__dependencies {
    grid-area: dependencies;

    .dependency {
      align-items: center;
      display: flex;
      gap: var(--spacing-card-sm);
      padding: var(--spacing-card-sm);

      .info {
        display: flex;
        flex-direction: column;
        gap: var(--spacing-card-xs);

        .project-title {
          font-weight: bold;
        }

        .dep-type {
          color: var(--color-text-secondary);

          &.incompatible {
            color: var(--color-red);
          }

          &::first-letter {
            text-transform: capitalize;
          }
        }
      }

      button {
        margin-left: auto;
      }
    }

    .add-dependency {
      h4 {
        margin-bottom: var(--spacing-card-sm);
      }

      .input-group {
        &:not(:last-child) {
          margin-bottom: var(--spacing-card-sm);
        }

        .multiselect {
          width: 8rem;
          flex-grow: 1;
        }

        input {
          flex-grow: 2;
        }
      }
    }
  }

  .version-page__files {
    grid-area: files;

    .file {
      --text-color: var(--color-button-text);
      --background-color: var(--color-button-bg);

      &.primary {
        --background-color: var(--color-brand-highlight);
        --text-color: var(--color-button-text-active);
      }

      display: flex;
      align-items: center;

      font-weight: 500;
      color: var(--text-color);
      background-color: var(--background-color);
      padding: var(--spacing-card-sm) var(--spacing-card-bg);
      border-radius: var(--size-rounded-sm);

      svg {
        min-width: 1.1rem;
        min-height: 1.1rem;
        margin-right: 0.5rem;
      }

      .filename {
        word-wrap: anywhere;
      }

      .file-size {
        margin-left: 1ch;
        font-weight: 400;
        white-space: nowrap;
      }

      .file-type {
        margin-left: 1ch;
        font-style: italic;
        font-weight: 300;
      }

      .raised-multiselect {
        display: none;
        margin: 0 0.5rem;
        height: 40px;
        max-height: 40px;
        min-width: 235px;
      }

      .iconified-button {
        margin-left: auto;
      }

      &:not(:nth-child(2)) {
        margin-top: 0.5rem;
      }

      // TODO: Make file type editing  work on mobile
      @media (min-width: 600px) {
        .raised-multiselect {
          display: block;
        }
      }
    }

    .additional-files {
      h4 {
        margin-bottom: 0.5rem;
      }

      label {
        margin-top: 0.5rem;
      }
    }
  }

  .version-page__metadata {
    grid-area: metadata;

    h4 {
      margin: 1rem 0 0.25rem 0;
    }

    .team-member {
      align-items: center;
      padding: 0.25rem 0.5rem;

      .member-info {
        overflow: hidden;
        margin: auto 0 auto 0.75rem;

        .name {
          font-weight: bold;
        }

        p {
          font-size: var(--font-size-sm);
          margin: 0.2rem 0;
        }
      }
    }
  }
}

@media (min-width: 1200px) {
  .version-page {
    grid-template:
      "title title" auto
      "changelog metadata" auto
      "dependencies metadata" auto
      "files metadata" auto
      "dummy metadata" 1fr
      / 1fr 20rem;
  }
}

.separator {
  margin: var(--spacing-card-sm) 0;
}

.modal-package-mod {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;

  .markdown-body {
    margin-bottom: 1rem;
  }

  .multiselect {
    max-width: 20rem;
  }
}
</style>
