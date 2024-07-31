<template>
  <div class="universal-card">
    <ConfirmModal
      ref="modal_confirm"
      :title="formatMessage(deleteModalMessages.title)"
      :description="formatMessage(deleteModalMessages.description)"
      :proceed-label="formatMessage(deleteModalMessages.action)"
      @proceed="removePat(deletePatIndex)"
    />
    <Modal
      ref="patModal"
      :header="
        editPatIndex !== null
          ? formatMessage(createModalMessages.editTitle)
          : formatMessage(createModalMessages.createTitle)
      "
    >
      <div class="universal-modal">
        <label for="pat-name">
          <span class="label__title">{{ formatMessage(createModalMessages.nameLabel) }}</span>
        </label>
        <input
          id="pat-name"
          v-model="name"
          maxlength="2048"
          type="email"
          :placeholder="formatMessage(createModalMessages.namePlaceholder)"
        />
        <label for="pat-scopes">
          <span class="label__title">{{ formatMessage(commonMessages.scopesLabel) }}</span>
        </label>
        <div id="pat-scopes" class="checkboxes">
          <Checkbox
            v-for="scope in scopeList"
            :key="scope"
            :label="scopesToLabels(getScopeValue(scope)).join(', ')"
            :model-value="hasScope(scopesVal, scope)"
            @update:model-value="scopesVal = toggleScope(scopesVal, scope)"
          />
        </div>
        <label for="pat-name">
          <span class="label__title">{{ formatMessage(createModalMessages.expiresLabel) }}</span>
        </label>
        <input id="pat-name" v-model="expires" type="date" />
        <p></p>
        <div class="input-group push-right">
          <button class="iconified-button" @click="$refs.patModal.hide()">
            <XIcon />
            {{ formatMessage(commonMessages.cancelButton) }}
          </button>
          <button
            v-if="editPatIndex !== null"
            :disabled="loading || !name || !expires"
            type="button"
            class="iconified-button brand-button"
            @click="editPat"
          >
            <SaveIcon />
            {{ formatMessage(commonMessages.saveChangesButton) }}
          </button>
          <button
            v-else
            :disabled="loading || !name || !expires"
            type="button"
            class="iconified-button brand-button"
            @click="createPat"
          >
            <PlusIcon />
            {{ formatMessage(createModalMessages.action) }}
          </button>
        </div>
      </div>
    </Modal>

    <div class="header__row">
      <div class="header__title">
        <h2 class="text-2xl">{{ formatMessage(commonSettingsMessages.pats) }}</h2>
      </div>
      <button
        class="btn btn-primary"
        @click="
          () => {
            name = null;
            scopesVal = 0;
            expires = null;
            editPatIndex = null;
            $refs.patModal.show();
          }
        "
      >
        <PlusIcon /> {{ formatMessage(messages.create) }}
      </button>
    </div>
    <p>
      <IntlFormatted :message-id="messages.description">
        <template #doc-link="{ children }">
          <a class="text-link" href="https://docs.modrinth.com">
            <component :is="() => children" />
          </a>
        </template>
      </IntlFormatted>
    </p>
    <div v-for="(pat, index) in pats" :key="pat.id" class="universal-card recessed token">
      <div>
        <div>
          <strong>{{ pat.name }}</strong>
        </div>
        <div>
          <template v-if="pat.access_token">
            <CopyCode :text="pat.access_token" />
          </template>
          <template v-else>
            <span
              v-tooltip="
                pat.last_used
                  ? formatMessage(commonMessages.dateAtTimeTooltip, {
                      date: new Date(pat.last_used),
                      time: new Date(pat.last_used),
                    })
                  : null
              "
            >
              <template v-if="pat.last_used">
                {{
                  formatMessage(tokenMessages.lastUsed, {
                    ago: formatRelativeTime(pat.last_used),
                  })
                }}
              </template>
              <template v-else>{{ formatMessage(tokenMessages.neverUsed) }}</template>
            </span>
            ⋅
            <span
              v-tooltip="
                formatMessage(commonMessages.dateAtTimeTooltip, {
                  date: new Date(pat.expires),
                  time: new Date(pat.expires),
                })
              "
            >
              <template v-if="new Date(pat.expires) > new Date()">
                {{
                  formatMessage(tokenMessages.expiresIn, {
                    inTime: formatRelativeTime(pat.expires),
                  })
                }}
              </template>
              <template v-else>
                {{
                  formatMessage(tokenMessages.expiredAgo, {
                    ago: formatRelativeTime(pat.expires),
                  })
                }}
              </template>
            </span>
            ⋅
            <span
              v-tooltip="
                formatMessage(commonMessages.dateAtTimeTooltip, {
                  date: new Date(pat.created),
                  time: new Date(pat.created),
                })
              "
            >
              {{
                formatMessage(commonMessages.createdAgoLabel, {
                  ago: formatRelativeTime(pat.created),
                })
              }}
            </span>
          </template>
        </div>
      </div>
      <div class="input-group">
        <button
          class="iconified-button raised-button"
          @click="
            () => {
              editPatIndex = index;
              name = pat.name;
              scopesVal = pat.scopes;
              expires = $dayjs(pat.expires).format('YYYY-MM-DD');
              $refs.patModal.show();
            }
          "
        >
          <EditIcon /> {{ formatMessage(tokenMessages.edit) }}
        </button>
        <button
          class="iconified-button raised-button"
          @click="
            () => {
              deletePatIndex = pat.id;
              $refs.modal_confirm.show();
            }
          "
        >
          <TrashIcon /> {{ formatMessage(tokenMessages.revoke) }}
        </button>
      </div>
    </div>
  </div>
</template>
<script setup>
import { PlusIcon, XIcon, TrashIcon, EditIcon, SaveIcon } from "@modrinth/assets";
import { Checkbox, ConfirmModal, CopyCode } from "@modrinth/ui";

import { commonSettingsMessages } from "~/utils/common-messages.ts";
import {
  hasScope,
  scopeList,
  toggleScope,
  useScopes,
  getScopeValue,
} from "~/composables/auth/scopes.ts";

import Modal from "~/components/ui/Modal.vue";

const { formatMessage } = useVIntl();

const formatRelativeTime = useRelativeTime();

const createModalMessages = defineMessages({
  createTitle: {
    id: "settings.pats.modal.create.title",
    defaultMessage: "Create personal access token",
  },
  editTitle: {
    id: "settings.pats.modal.edit.title",
    defaultMessage: "Edit personal access token",
  },
  nameLabel: {
    id: "settings.pats.modal.create.name.label",
    defaultMessage: "Name",
  },
  namePlaceholder: {
    id: "settings.pats.modal.create.name.placeholder",
    defaultMessage: "Enter the PAT's name...",
  },
  expiresLabel: {
    id: "settings.pats.modal.create.expires.label",
    defaultMessage: "Expires",
  },
  action: {
    id: "settings.pats.modal.create.action",
    defaultMessage: "Create PAT",
  },
});

const deleteModalMessages = defineMessages({
  title: {
    id: "settings.pats.modal.delete.title",
    defaultMessage: "Are you sure you want to delete this token?",
  },
  description: {
    id: "settings.pats.modal.delete.description",
    defaultMessage: "This will remove this token forever (like really forever).",
  },
  action: {
    id: "settings.pats.modal.delete.action",
    defaultMessage: "Delete this token",
  },
});

const messages = defineMessages({
  description: {
    id: "settings.pats.description",
    defaultMessage:
      "PATs can be used to access Modrinth's API. For more information, see <doc-link>Modrinth's API documentation</doc-link>. They can be created and revoked at any time.",
  },
  create: {
    id: "settings.pats.action.create",
    defaultMessage: "Create a PAT",
  },
});

const tokenMessages = defineMessages({
  edit: {
    id: "settings.pats.token.action.edit",
    defaultMessage: "Edit token",
  },
  revoke: {
    id: "settings.pats.token.action.revoke",
    defaultMessage: "Revoke token",
  },
  lastUsed: {
    id: "settings.pats.token.last-used",
    defaultMessage: "Last used {ago}",
  },
  neverUsed: {
    id: "settings.pats.token.never-used",
    defaultMessage: "Never used",
  },
  expiresIn: {
    id: "settings.pats.token.expires-in",
    defaultMessage: "Expires {inTime}",
  },
  expiredAgo: {
    id: "settings.pats.token.expired-ago",
    defaultMessage: "Expired {ago}",
  },
});

definePageMeta({
  middleware: "auth",
});

useHead({
  title: `${formatMessage(commonSettingsMessages.pats)} - Modrinth`,
});

const data = useNuxtApp();
const { scopesToLabels } = useScopes();
const patModal = ref();

const editPatIndex = ref(null);

const name = ref(null);
const scopesVal = ref(BigInt(0));
const expires = ref(null);

const deletePatIndex = ref(null);

const loading = ref(false);

const { data: pats, refresh } = await useAsyncData("pat", () => useBaseFetch("pat"));

async function createPat() {
  startLoading();
  loading.value = true;
  try {
    const res = await useBaseFetch("pat", {
      method: "POST",
      body: {
        name: name.value,
        scopes: Number(scopesVal.value),
        expires: data.$dayjs(expires.value).toISOString(),
      },
    });
    pats.value.push(res);
    patModal.value.hide();
  } catch (err) {
    data.$notify({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  loading.value = false;
  stopLoading();
}

async function editPat() {
  startLoading();
  loading.value = true;
  try {
    await useBaseFetch(`pat/${pats.value[editPatIndex.value].id}`, {
      method: "PATCH",
      body: {
        name: name.value,
        scopes: Number(scopesVal.value),
        expires: data.$dayjs(expires.value).toISOString(),
      },
    });
    await refresh();
    patModal.value.hide();
  } catch (err) {
    data.$notify({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  loading.value = false;
  stopLoading();
}

async function removePat(id) {
  startLoading();
  try {
    pats.value = pats.value.filter((x) => x.id !== id);
    await useBaseFetch(`pat/${id}`, {
      method: "DELETE",
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}
</script>
<style lang="scss" scoped>
.checkboxes {
  display: grid;
  column-gap: 0.5rem;

  @media screen and (min-width: 432px) {
    grid-template-columns: repeat(2, 1fr);
  }

  @media screen and (min-width: 800px) {
    grid-template-columns: repeat(3, 1fr);
  }
}

.token {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  @media screen and (min-width: 800px) {
    flex-direction: row;
    align-items: center;

    .input-group {
      margin-left: auto;
    }
  }
}
</style>
