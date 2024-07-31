<template>
  <div>
    <div class="universal-card">
      <div class="markdown-disclaimer">
        <h2>Description</h2>
        <span class="label__description">
          You can type an extended description of your mod here.
          <span class="label__subdescription">
            The description must clearly and honestly describe the purpose and function of the
            project. See section 2.1 of the
            <nuxt-link to="/legal/rules" class="text-link" target="_blank">Content Rules</nuxt-link>
            for the full requirements.
          </span>
        </span>
      </div>
      <MarkdownEditor
        v-model="description"
        :on-image-upload="onUploadHandler"
        :disabled="(currentMember.permissions & EDIT_BODY) !== EDIT_BODY"
      />
      <div class="input-group markdown-disclaimer">
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
    </div>
  </div>
</template>

<script>
import { MarkdownEditor, Chips } from "@modrinth/ui";
import SaveIcon from "~/assets/images/utils/save.svg?component";
import { renderHighlightedString } from "~/helpers/highlight.js";
import { useImageUpload } from "~/composables/image-upload.ts";

export default defineNuxtComponent({
  components: {
    Chips,
    SaveIcon,
    MarkdownEditor,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {};
      },
    },
    allMembers: {
      type: Array,
      default() {
        return [];
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null;
      },
    },
    patchProject: {
      type: Function,
      default() {
        return () => {
          this.$notify({
            group: "main",
            title: "An error occurred",
            text: "Patch project function not found",
            type: "error",
          });
        };
      },
    },
  },
  data() {
    return {
      description: this.project.body,
      bodyViewMode: "source",
    };
  },
  computed: {
    patchData() {
      const data = {};

      if (this.description !== this.project.body) {
        data.body = this.description;
      }

      return data;
    },
    hasChanges() {
      return Object.keys(this.patchData).length > 0;
    },
  },
  created() {
    this.EDIT_BODY = 1 << 3;
  },
  methods: {
    renderHighlightedString,
    saveChanges() {
      if (this.hasChanges) {
        this.patchProject(this.patchData);
      }
    },
    async onUploadHandler(file) {
      const response = await useImageUpload(file, {
        context: "project",
        projectID: this.project.id,
      });
      return response.url;
    },
  },
});
</script>

<style scoped>
.markdown-disclaimer {
  margin-block: 1rem;
}

.universal-card {
  margin-top: 1rem;
}
</style>
