<template>
  <div>
    <section class="universal-card payout-history">
      <Breadcrumbs
        current-title="Transfer history"
        :link-stack="[{ href: '/dashboard/revenue', label: 'Revenue' }]"
      />
      <h2>Transfer history</h2>
      <p>All of your withdrawals from your Modrinth balance will be listed here:</p>
      <div class="input-group">
        <DropdownSelect
          v-model="selectedYear"
          :options="years"
          :display-name="(x) => (x === 'all' ? 'All years' : x)"
          name="Year filter"
        />
        <DropdownSelect
          v-model="selectedMethod"
          :options="methods"
          :display-name="
            (x) => (x === 'all' ? 'Any method' : x === 'paypal' ? 'PayPal' : capitalizeString(x))
          "
          name="Method filter"
        />
      </div>
      <p>
        {{
          selectedYear !== "all"
            ? selectedMethod !== "all"
              ? formatMessage(messages.transfersTotalYearMethod, {
                  amount: $formatMoney(totalAmount),
                  year: selectedYear,
                  method: selectedMethod,
                })
              : formatMessage(messages.transfersTotalYear, {
                  amount: $formatMoney(totalAmount),
                  year: selectedYear,
                })
            : selectedMethod !== "all"
              ? formatMessage(messages.transfersTotalMethod, {
                  amount: $formatMoney(totalAmount),
                  method: selectedMethod,
                })
              : formatMessage(messages.transfersTotal, { amount: $formatMoney(totalAmount) })
        }}
      </p>
      <div
        v-for="payout in filteredPayouts"
        :key="payout.id"
        class="universal-card recessed payout"
      >
        <div class="platform">
          <PayPalIcon v-if="payout.method === 'paypal'" />
          <TremendousIcon v-else-if="payout.method === 'tremendous'" />
          <VenmoIcon v-else-if="payout.method === 'venmo'" />
          <UnknownIcon v-else />
        </div>
        <div class="payout-info">
          <div>
            <strong>
              {{ $dayjs(payout.created).format("MMMM D, YYYY [at] h:mm A") }}
            </strong>
          </div>
          <div>
            <span class="amount">{{ $formatMoney(payout.amount) }}</span>
            <template v-if="payout.fee">⋅ Fee {{ $formatMoney(payout.fee) }}</template>
          </div>
          <div class="payout-status">
            <span>
              <Badge v-if="payout.status === 'success'" color="green" type="Success" />
              <Badge v-else-if="payout.status === 'cancelling'" color="yellow" type="Cancelling" />
              <Badge v-else-if="payout.status === 'cancelled'" color="red" type="Cancelled" />
              <Badge v-else-if="payout.status === 'failed'" color="red" type="Failed" />
              <Badge v-else-if="payout.status === 'in-transit'" color="yellow" type="In transit" />
              <Badge v-else :type="payout.status" />
            </span>
            <template v-if="payout.method">
              <span>⋅</span>
              <span>{{ $formatWallet(payout.method) }} ({{ payout.method_address }})</span>
            </template>
          </div>
        </div>
        <div class="input-group">
          <button
            v-if="payout.status === 'in-transit'"
            class="iconified-button raised-button"
            @click="cancelPayout(payout.id)"
          >
            <XIcon /> Cancel payment
          </button>
        </div>
      </div>
    </section>
  </div>
</template>
<script setup>
import { XIcon, PayPalIcon, UnknownIcon } from "@modrinth/assets";
import { capitalizeString } from "@modrinth/utils";
import { Badge, Breadcrumbs, DropdownSelect } from "@modrinth/ui";
import dayjs from "dayjs";
import TremendousIcon from "~/assets/images/external/tremendous.svg?component";
import VenmoIcon from "~/assets/images/external/venmo-small.svg?component";

const vintl = useVIntl();
const { formatMessage } = vintl;

useHead({
  title: "Transfer history - Modrinth",
});

const data = await useNuxtApp();
const auth = await useAuth();

const { data: payouts, refresh } = await useAsyncData(`payout`, () =>
  useBaseFetch(`payout`, {
    apiVersion: 3,
  }),
);

const sortedPayouts = computed(() =>
  [...payouts.value].sort((a, b) => dayjs(b.created) - dayjs(a.created)),
);

const years = computed(() => {
  const values = sortedPayouts.value.map((x) => dayjs(x.created).year());
  return ["all", ...new Set(values)];
});

const selectedYear = ref("all");

const methods = computed(() => {
  const values = sortedPayouts.value.filter((x) => x.method).map((x) => x.method);
  return ["all", ...new Set(values)];
});

const selectedMethod = ref("all");

const filteredPayouts = computed(() =>
  sortedPayouts.value
    .filter((x) => selectedYear.value === "all" || dayjs(x.created).year() === selectedYear.value)
    .filter((x) => selectedMethod.value === "all" || x.method === selectedMethod.value),
);

const totalAmount = computed(() =>
  filteredPayouts.value.reduce((sum, payout) => sum + payout.amount, 0),
);

async function cancelPayout(id) {
  startLoading();
  try {
    await useBaseFetch(`payout/${id}`, {
      method: "DELETE",
      apiVersion: 3,
    });
    await refresh();
    await useAuth(auth.value.token);
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data.description,
      type: "error",
    });
  }
  stopLoading();
}

const messages = defineMessages({
  transfersTotal: {
    id: "revenue.transfers.total",
    defaultMessage: "You have withdrawn {amount} in total.",
  },
  transfersTotalYear: {
    id: "revenue.transfers.total.year",
    defaultMessage: "You have withdrawn {amount} in {year}.",
  },
  transfersTotalMethod: {
    id: "revenue.transfers.total.method",
    defaultMessage: "You have withdrawn {amount} through {method}.",
  },
  transfersTotalYearMethod: {
    id: "revenue.transfers.total.year_method",
    defaultMessage: "You have withdrawn {amount} in {year} through {method}.",
  },
});
</script>
<style lang="scss" scoped>
.payout {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;

  .platform {
    display: flex;
    padding: 0.75rem;
    background-color: var(--color-raised-bg);
    width: fit-content;
    height: fit-content;
    border-radius: 20rem;

    svg {
      width: 2rem;
      height: 2rem;
    }
  }

  .payout-status {
    display: flex;
    gap: 0.5ch;
  }

  .amount {
    color: var(--color-heading);
    font-weight: 500;
  }

  @media screen and (min-width: 800px) {
    flex-direction: row;
    align-items: center;

    .input-group {
      margin-left: auto;
    }
  }
}
</style>
