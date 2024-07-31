<template>
  <div>
    <section class="universal-card">
      <div class="adjacent-input">
        <label for="license-multiselect">
          <span class="label__title size-card-header">License</span>
          <span class="label__description">
            It is very important to choose a proper license for your
            {{ $formatProjectType(project.project_type).toLowerCase() }}. You may choose one from
            our list or provide a custom license. You may also provide a custom URL to your chosen
            license; otherwise, the license text will be displayed.
            <span v-if="license && license.friendly === 'Custom'" class="label__subdescription">
              Enter a valid
              <a href="https://spdx.org/licenses/" target="_blank" rel="noopener" class="text-link">
                SPDX license identifier</a
              >
              in the marked area. If your license does not have a SPDX identifier (for example, if
              you created the license yourself or if the license is Minecraft-specific), simply
              check the box and enter the name of the license instead.
            </span>
            <span class="label__subdescription">
              Confused? See our
              <a
                href="https://blog.modrinth.com/licensing-guide/"
                target="_blank"
                rel="noopener"
                class="text-link"
              >
                licensing guide</a
              >
              for more information.
            </span>
          </span>
        </label>
        <div class="input-stack">
          <Multiselect
            id="license-multiselect"
            v-model="license"
            placeholder="Select license..."
            track-by="short"
            label="friendly"
            :options="defaultLicenses"
            :searchable="true"
            :close-on-select="true"
            :show-labels="false"
            :class="{
              'known-error': license?.short === '' && showKnownErrors,
            }"
            :disabled="!hasPermission"
          />
          <Checkbox
            v-if="license?.requiresOnlyOrLater"
            v-model="allowOrLater"
            :disabled="!hasPermission"
            description="Allow later editions of this license"
          >
            Allow later editions of this license
          </Checkbox>
          <Checkbox
            v-if="license?.friendly === 'Custom'"
            v-model="nonSpdxLicense"
            :disabled="!hasPermission"
            description="License does not have a SPDX identifier"
          >
            License does not have a SPDX identifier
          </Checkbox>
          <input
            v-if="license?.friendly === 'Custom'"
            v-model="license.short"
            type="text"
            maxlength="2048"
            :placeholder="nonSpdxLicense ? 'License name' : 'SPDX identifier'"
            :class="{
              'known-error': license.short === '' && showKnownErrors,
            }"
            :disabled="!hasPermission"
          />
          <input
            v-model="licenseUrl"
            type="url"
            maxlength="2048"
            placeholder="License URL (optional)"
            :disabled="!hasPermission || licenseId === 'LicenseRef-Unknown'"
          />
        </div>
      </div>
      <div class="input-stack">
        <button
          type="button"
          class="iconified-button brand-button"
          :disabled="!hasChanges || license === null"
          @click="saveChanges()"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
    </section>
  </div>
</template>

<script>
import Multiselect from "vue-multiselect";
import { Checkbox } from "@modrinth/ui";
import SaveIcon from "~/assets/images/utils/save.svg?component";

export default defineNuxtComponent({
  components: {
    Multiselect,
    Checkbox,
    SaveIcon,
  },
  props: {
    project: {
      type: Object,
      default() {
        return {};
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
      licenseUrl: "",
      license: { friendly: "", short: "", requiresOnlyOrLater: false },
      allowOrLater: this.project.license.id.includes("-or-later"),
      nonSpdxLicense: this.project.license.id.includes("LicenseRef-"),
      showKnownErrors: false,
    };
  },
  async setup(props) {
    const defaultLicenses = shallowRef([
      { friendly: "Custom", short: "" },
      {
        friendly: "All Rights Reserved/No License",
        short: "All-Rights-Reserved",
      },
      { friendly: "Apache License 2.0", short: "Apache-2.0" },
      {
        friendly: 'BSD 2-Clause "Simplified" License',
        short: "BSD-2-Clause",
      },
      {
        friendly: 'BSD 3-Clause "New" or "Revised" License',
        short: "BSD-3-Clause",
      },
      {
        friendly: "CC Zero (Public Domain equivalent)",
        short: "CC0-1.0",
      },
      { friendly: "CC-BY 4.0", short: "CC-BY-4.0" },
      {
        friendly: "CC-BY-SA 4.0",
        short: "CC-BY-SA-4.0",
      },
      {
        friendly: "CC-BY-NC 4.0",
        short: "CC-BY-NC-4.0",
      },
      {
        friendly: "CC-BY-NC-SA 4.0",
        short: "CC-BY-NC-SA-4.0",
      },
      {
        friendly: "CC-BY-ND 4.0",
        short: "CC-BY-ND-4.0",
      },
      {
        friendly: "CC-BY-NC-ND 4.0",
        short: "CC-BY-NC-ND-4.0",
      },
      {
        friendly: "GNU Affero General Public License v3",
        short: "AGPL-3.0",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU Lesser General Public License v2.1",
        short: "LGPL-2.1",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU Lesser General Public License v3",
        short: "LGPL-3.0",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU General Public License v2",
        short: "GPL-2.0",
        requiresOnlyOrLater: true,
      },
      {
        friendly: "GNU General Public License v3",
        short: "GPL-3.0",
        requiresOnlyOrLater: true,
      },
      { friendly: "ISC License", short: "ISC" },
      { friendly: "MIT License", short: "MIT" },
      { friendly: "Mozilla Public License 2.0", short: "MPL-2.0" },
      { friendly: "zlib License", short: "Zlib" },
    ]);

    const licenseUrl = ref(props.project.license.url);

    const licenseId = props.project.license.id;
    const trimmedLicenseId = licenseId
      .replaceAll("-only", "")
      .replaceAll("-or-later", "")
      .replaceAll("LicenseRef-", "");

    const license = ref(
      defaultLicenses.value.find((x) => x.short === trimmedLicenseId) ?? {
        friendly: "Custom",
        short: licenseId.replaceAll("LicenseRef-", ""),
      },
    );

    if (licenseId === "LicenseRef-Unknown") {
      license.value = {
        friendly: "Unknown",
        short: licenseId.replaceAll("LicenseRef-", ""),
      };
    }

    return {
      defaultLicenses,
      licenseUrl,
      license,
    };
  },
  computed: {
    hasPermission() {
      const EDIT_DETAILS = 1 << 2;
      return (this.currentMember.permissions & EDIT_DETAILS) === EDIT_DETAILS;
    },
    licenseId() {
      let id = "";
      if (this.license === null) return id;
      if (
        (this.nonSpdxLicense && this.license.friendly === "Custom") ||
        this.license.short === "All-Rights-Reserved" ||
        this.license.short === "Unknown"
      ) {
        id += "LicenseRef-";
      }
      id += this.license.short;
      if (this.license.requiresOnlyOrLater) {
        id += this.allowOrLater ? "-or-later" : "-only";
      }
      if (this.nonSpdxLicense && this.license.friendly === "Custom") {
        id = id.replaceAll(" ", "-");
      }
      return id;
    },
    patchData() {
      const data = {};

      if (this.licenseId !== this.project.license.id) {
        data.license_id = this.licenseId;
        data.license_url = this.licenseUrl ? this.licenseUrl : null;
      } else if (this.licenseUrl !== this.project.license.url) {
        data.license_url = this.licenseUrl ? this.licenseUrl : null;
      }

      return data;
    },
    hasChanges() {
      return Object.keys(this.patchData).length > 0;
    },
  },
  methods: {
    saveChanges() {
      if (this.hasChanges) {
        this.patchProject(this.patchData);
      }
    },
  },
});
</script>
