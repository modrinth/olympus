<template>
  <section class="universal-card">
    <h2>{{ formatMessage(messages.subscriptionTitle) }}</h2>
    <p>{{ formatMessage(messages.subscriptionDescription) }}</p>
    <div class="universal-card recessed">
      <div class="flex justify-between">
        <div class="flex flex-col gap-4">
          <span v-if="midasSubscription">You're currently subscribed to:</span>
          <span v-else>Become a subscriber to Modrinth Plus!</span>
          <ModrinthPlusIcon class="h-8 w-min" />
          <div class="flex flex-col gap-2">
            <span class="font-bold">Benefits</span>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Ad-free browsing on modrinth.com and Modrinth App</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Modrinth+ badge on your profile</span>
            </div>
            <div class="flex items-center gap-2">
              <CheckCircleIcon class="h-5 w-5 text-brand" />
              <span>Support Modrinth and creators directly</span>
            </div>
          </div>
        </div>
        <div class="flex flex-col justify-between text-right">
          <div class="flex flex-col gap-1">
            <span class="text-2xl font-bold text-dark">
              <template v-if="midasSubscription">
                {{
                  formatPrice(
                    midasSubscriptionPrice.prices.intervals[midasSubscription.interval],
                    midasSubscriptionPrice.currency_code,
                  )
                }}
                /
                {{ midasSubscription.interval }}
              </template>
              <template v-else>
                {{ formatPrice(price.prices.intervals.monthly, currency) }} / month
              </template>
            </span>
            <span v-if="midasSubscription" class="text-sm text-secondary">
              Since {{ $dayjs(midasSubscription.created).format("MMMM D, YYYY") }}
            </span>
            <span v-if="midasSubscription" class="text-sm text-secondary">
              Renews {{ $dayjs(midasSubscription.expires).format("MMMM D, YYYY") }}
            </span>
            <span v-else class="text-sm text-secondary">
              Or {{ formatPrice(price.prices.intervals.yearly, currency) }} / year (save
              {{
                calculateSavings(price.prices.intervals.monthly, price.prices.intervals.yearly)
              }}%)!
            </span>
          </div>
          <button v-if="midasSubscription" class="btn ml-auto"><XIcon /> Cancel</button>
          <button
            v-else
            class="btn btn-purple btn-large ml-auto"
            @click="
              purchaseModalStep = 0;
              $refs.purchaseModal.show();
            "
          >
            <RightArrowIcon />
            Subscribe
          </button>
        </div>
      </div>
    </div>
  </section>
  <section class="universal-card">
    <ConfirmModal
      ref="modal_confirm"
      :title="formatMessage(deleteModalMessages.title)"
      :description="formatMessage(deleteModalMessages.description)"
      :proceed-label="formatMessage(deleteModalMessages.action)"
      @proceed="removePaymentMethod(removePaymentMethodIndex)"
    />
    <NewModal ref="purchaseModal">
      <template #title>
        <span class="text-contrast text-xl font-extrabold">Subscribe to Modrinth Plus!</span>
      </template>
      <div class="flex items-center gap-1 pb-4">
        <span
          :class="{
            'text-secondary': purchaseModalStep !== 0,
            'font-bold': purchaseModalStep === 0,
          }"
        >
          Select plan
        </span>
        <ChevronRightIcon class="h-5 w-5 text-secondary" />
        <span
          :class="{
            'text-secondary': purchaseModalStep !== 1,
            'font-bold': purchaseModalStep === 1,
          }"
        >
          Payment
        </span>
        <ChevronRightIcon class="h-5 w-5 text-secondary" />
        <span
          :class="{
            'text-secondary': purchaseModalStep !== 2,
            'font-bold': purchaseModalStep === 2,
          }"
        >
          Review
        </span>
      </div>
      <div v-if="purchaseModalStep === 0" class="sm:w-[600px]">
        <div>
          <p class="my-2 text-lg font-bold">Choose billing interval</p>
          <div class="flex flex-col gap-4">
            <div
              v-for="([interval, rawPrice], index) in Object.entries(price.prices.intervals)"
              :key="index"
              class="flex cursor-pointer items-center gap-2"
              @click="selectedPlan = interval"
            >
              <RadioButtonChecked v-if="selectedPlan === interval" class="h-8 w-8 text-brand" />
              <RadioButtonIcon v-else class="h-8 w-8 text-secondary" />
              <span
                class="text-lg capitalize"
                :class="{ 'text-secondary': selectedPlan !== interval }"
              >
                {{ interval }}
              </span>
              <span
                v-if="interval === 'yearly'"
                class="rounded-full bg-brand px-2 py-1 font-bold text-brand-inverted"
              >
                SAVE {{ calculateSavings(price.prices.intervals.monthly, rawPrice) }}%
              </span>
              <span
                class="ml-auto text-lg"
                :class="{ 'text-secondary': selectedPlan !== interval }"
              >
                {{ formatPrice(rawPrice, currency) }}
              </span>
            </div>
          </div>
          <div class="mt-4 flex justify-between border-0 border-t border-solid border-code-bg pt-4">
            <span class="text-xl text-secondary">Total</span>
            <div class="flex items-baseline gap-2">
              <span class="text-2xl font-extrabold text-primary">
                {{ formatPrice(price.prices.intervals[selectedPlan], currency) }}
              </span>
              <span class="text-lg text-secondary">/ {{ selectedPlan }}</span>
            </div>
          </div>

          <div class="flex items-center gap-2 pt-4">
            <InfoIcon />
            <span class="text-sm text-secondary">
              Final price and currency will be based on your selected payment method.
            </span>
          </div>
        </div>
      </div>
      <template v-if="purchaseModalStep === 1">
        <div
          v-show="loadingPaymentMethodModal !== 2"
          class="flex min-h-[16rem] items-center justify-center sm:w-[600px]"
        >
          <AnimatedLogo class="w-[80px]" />
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="min-h-[16rem] p-1 sm:w-[600px]">
          <div id="address-element"></div>
          <div id="payment-element" class="mt-4"></div>
        </div>
      </template>
      <div v-if="purchaseModalStep === 2" class="sm:w-[650px]">
        <div>
          <div class="r-4 rounded-xl bg-bg p-4">
            <p class="my-2 text-lg font-bold text-primary">Purchase details</p>
            <div class="mb-2 flex justify-between">
              <span class="text-secondary">Modrinth+ {{ selectedPlan }}</span>
              <span class="text-secondary">
                {{ formatPrice(price.prices.intervals[selectedPlan], currency) }} /
                {{ selectedPlan }}
              </span>
            </div>
            <div class="flex justify-between">
              <span class="text-secondary">Tax</span>
              <span class="text-secondary">{{ formatPrice(tax, currency) }}</span>
            </div>
            <div
              class="mt-4 flex justify-between border-0 border-t border-solid border-code-bg pt-4"
            >
              <span class="text-lg font-bold">Today's total</span>
              <span class="text-lg font-extrabold text-primary">
                {{ formatPrice(price.prices.intervals[selectedPlan], currency) }}
              </span>
            </div>
          </div>
          <p class="my-2 text-lg font-bold">Pay for it with</p>
          <multiselect
            v-model="selectedPaymentMethod"
            placeholder="Payment method"
            label="id"
            track-by="id"
            :options="selectablePaymentMethods"
            :option-height="104"
            :show-labels="false"
            :searchable="false"
            :close-on-select="true"
            :allow-empty="false"
            open-direction="top"
            class="max-w-[20rem]"
            @select="selectPaymentMethod"
          >
            <!-- TODO: Move this to component to remove duplicated code -->
            <template #singleLabel="props">
              <div class="flex items-center gap-2">
                <CardIcon v-if="props.option.type === 'card'" class="h-8 w-8" />
                <CurrencyIcon v-else-if="props.option.type === 'cashapp'" class="h-8 w-8" />
                <PayPalIcon v-else-if="props.option.type === 'paypal'" class="h-8 w-8" />

                <span v-if="props.option.type === 'card'">
                  {{
                    formatMessage(messages.paymentMethodCardDisplay, {
                      card_brand:
                        formatMessage(paymentMethodTypes[props.option.card.brand]) ??
                        formatMessage(paymentMethodTypes.unknown),
                      last_four: props.option.card.last4,
                    })
                  }}
                </span>
                <template v-else>
                  {{
                    formatMessage(paymentMethodTypes[props.option.type]) ??
                    formatMessage(paymentMethodTypes.unknown)
                  }}
                </template>

                <span v-if="props.option.type === 'cashapp'">
                  ({{ props.option.cashapp.cashtag }})
                </span>
                <span v-else-if="props.option.type === 'paypal'">
                  ({{ props.option.paypal.payer_email }})
                </span>
              </div>
            </template>
            <template #option="props">
              <div class="flex items-center gap-2">
                <template v-if="props.option.id === 'new'">
                  <PlusIcon class="h-8 w-8" />
                  <span class="text-secondary">
                    {{ formatMessage(messages.paymentMethodAdd) }}
                  </span>
                </template>
                <template v-else>
                  <CardIcon v-if="props.option.type === 'card'" class="h-8 w-8" />
                  <CurrencyIcon v-else-if="props.option.type === 'cashapp'" class="h-8 w-8" />
                  <PayPalIcon v-else-if="props.option.type === 'paypal'" class="h-8 w-8" />

                  <span v-if="props.option.type === 'card'">
                    {{
                      formatMessage(messages.paymentMethodCardDisplay, {
                        card_brand:
                          formatMessage(paymentMethodTypes[props.option.card.brand]) ??
                          formatMessage(paymentMethodTypes.unknown),
                        last_four: props.option.card.last4,
                      })
                    }}
                  </span>
                  <template v-else>
                    {{
                      formatMessage(paymentMethodTypes[props.option.type]) ??
                      formatMessage(paymentMethodTypes.unknown)
                    }}
                  </template>

                  <span v-if="props.option.type === 'cashapp'">
                    ({{ props.option.cashapp.cashtag }})
                  </span>
                  <span v-else-if="props.option.type === 'paypal'">
                    ({{ props.option.paypal.payer_email }})
                  </span>
                </template>
              </div>
            </template>
          </multiselect>
        </div>
        <p class="m-0 mt-9 text-sm text-secondary">
          <strong>By clicking "Subscribe", you are purchasing a recurring subscription.</strong>
          You'll be charged {{ formatPrice(price.prices.intervals[selectedPlan], currency) }} /
          {{ selectedPlan }} plus applicable taxes starting today, until you cancel. Cancel anytime
          from your settings page.
        </p>
      </div>
      <div class="input-group push-right pt-4">
        <template v-if="purchaseModalStep === 0">
          <button class="btn" @click="$refs.purchaseModal.hide()">
            <XIcon />
            {{ formatMessage(commonMessages.cancelButton) }}
          </button>
          <button
            class="btn btn-primary"
            :disabled="paymentLoading"
            @click="beginPurchaseFlow(true)"
          >
            <RightArrowIcon />
            Select
          </button>
        </template>
        <template v-else-if="purchaseModalStep === 1">
          <button class="btn" @click="purchaseModalStep = 0">Back</button>
          <button class="btn btn-primary" :disabled="paymentLoading" @click="validatePayment">
            <RightArrowIcon />
            Continue
          </button>
        </template>
        <template v-else-if="purchaseModalStep === 2">
          <button class="btn" @click="$refs.purchaseModal.hide()">
            <XIcon />
            {{ formatMessage(commonMessages.cancelButton) }}
          </button>
          <button class="btn btn-primary" :disabled="paymentLoading" @click="submitPayment">
            <CheckCircleIcon /> Subscribe
          </button>
        </template>
      </div>
    </NewModal>
    <NewModal ref="addPaymentMethodModal">
      <template #title>
        <span class="text-contrast text-lg font-extrabold">
          {{ formatMessage(messages.paymentMethodTitle) }}
        </span>
      </template>
      <div class="min-h-[16rem] sm:w-[600px]">
        <div
          v-show="loadingPaymentMethodModal !== 2"
          class="flex min-h-[16rem] items-center justify-center"
        >
          <AnimatedLogo class="w-[80px]" />
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="min-h-[16rem] p-1">
          <div id="address-element"></div>
          <div id="payment-element" class="mt-4"></div>
        </div>
        <div v-show="loadingPaymentMethodModal === 2" class="input-group push-right mt-auto pt-4">
          <button class="btn" @click="$refs.addPaymentMethodModal.hide()">
            <XIcon />
            {{ formatMessage(commonMessages.cancelButton) }}
          </button>
          <button class="btn btn-primary" :disabled="loadingAddMethod" @click="submit">
            <PlusIcon />
            {{ formatMessage(messages.paymentMethodAdd) }}
          </button>
        </div>
      </div>
    </NewModal>
    <div class="header__row">
      <div class="header__title">
        <h2 class="text-2xl">{{ formatMessage(messages.paymentMethodTitle) }}</h2>
      </div>
      <button class="btn" @click="addPaymentMethod">
        <PlusIcon /> {{ formatMessage(messages.paymentMethodAdd) }}
      </button>
    </div>
    <div
      v-if="!paymentMethods || paymentMethods.length === 0"
      class="universal-card recessed !mb-0"
    >
      {{ formatMessage(messages.paymentMethodNone) }}
    </div>
    <div v-else class="flex flex-col gap-4">
      <div
        v-for="(method, index) in paymentMethods"
        :key="index"
        class="universal-card recessed !mb-0 flex items-center justify-between"
      >
        <div class="flex gap-2">
          <CardIcon v-if="method.type === 'card'" class="h-8 w-8" />
          <CurrencyIcon v-else-if="method.type === 'cashapp'" class="h-8 w-8" />
          <PayPalIcon v-else-if="method.type === 'paypal'" class="h-8 w-8" />
          <div class="flex flex-col">
            <div class="flex items-center gap-2">
              <div class="text-contrast font-bold">
                <template v-if="method.type === 'card'">
                  {{
                    formatMessage(messages.paymentMethodCardDisplay, {
                      card_brand:
                        formatMessage(paymentMethodTypes[method.card.brand]) ??
                        formatMessage(paymentMethodTypes.unknown),
                      last_four: method.card.last4,
                    })
                  }}
                </template>
                <template v-else>
                  {{
                    formatMessage(paymentMethodTypes[method.type]) ??
                    formatMessage(paymentMethodTypes.unknown)
                  }}
                </template>
              </div>
              <div
                v-if="primaryPaymentMethodId === method.id"
                class="border-r-ma rounded-full bg-button-bg px-2 py-0.5 text-sm font-bold text-secondary"
              >
                {{ formatMessage(messages.paymentMethodPrimary) }}
              </div>
            </div>
            <div v-if="method.type === 'card'" class="text-secondary">
              {{
                formatMessage(messages.paymentMethodCardExpiry, {
                  month: method.card.exp_month,
                  year: method.card.exp_year,
                })
              }}
            </div>
            <div v-else-if="method.type === 'cashapp'" class="text-secondary">
              {{ method.cashapp.cashtag }}
            </div>
            <div v-else-if="method.type === 'paypal'" class="text-secondary">
              {{ method.paypal.payer_email }}
            </div>
          </div>
        </div>
        <OverflowMenu
          class="btn icon-only transparent"
          :options="
            [
              {
                id: 'primary',
                action: () => editPaymentMethod(index, true),
              },
              {
                id: 'remove',
                action: () => {
                  removePaymentMethodIndex = index;
                  $refs.modal_confirm.show();
                },
                color: 'red',
                hoverOnly: true,
              },
            ].slice(primaryPaymentMethodId === method.id ? 1 : 0, 2)
          "
        >
          <MoreVerticalIcon />
          <template #primary>
            <StarIcon />
            {{ formatMessage(messages.paymentMethodMakePrimary) }}
          </template>
          <template #edit>
            <EditIcon />
            {{ formatMessage(commonMessages.editButton) }}
          </template>
          <template #remove>
            <TrashIcon />
            {{ formatMessage(commonMessages.deleteLabel) }}
          </template>
        </OverflowMenu>
      </div>
    </div>
  </section>
</template>
<script setup>
import { Multiselect } from "vue-multiselect";
import { ConfirmModal, NewModal, OverflowMenu, AnimatedLogo } from "@modrinth/ui";
import {
  PlusIcon,
  XIcon,
  CardIcon,
  MoreVerticalIcon,
  TrashIcon,
  EditIcon,
  StarIcon,
  PayPalIcon,
  CurrencyIcon,
  CheckCircleIcon,
  RightArrowIcon,
  ModrinthPlusIcon,
  RadioButtonIcon,
  RadioButtonChecked,
  InfoIcon,
  ChevronRightIcon,
} from "@modrinth/assets";
import { products } from "~/generated/state.json";

const data = useNuxtApp();

const vintl = useVIntl();
const { formatMessage } = vintl;

const deleteModalMessages = defineMessages({
  title: {
    id: "settings.billing.modal.delete.title",
    defaultMessage: "Are you sure you want to remove this payment method?",
  },
  description: {
    id: "settings.billing.modal.delete.description",
    defaultMessage: "This will remove this payment method forever (like really forever).",
  },
  action: {
    id: "settings.billing.modal.delete.action",
    defaultMessage: "Remove this payment method",
  },
});

const messages = defineMessages({
  subscriptionTitle: {
    id: "settings.billing.subscription.title",
    defaultMessage: "Subscriptions",
  },
  subscriptionDescription: {
    id: "settings.billing.subscription.description",
    defaultMessage: "Manage your Modrinth subscriptions.",
  },
  paymentMethodTitle: {
    id: "settings.billing.payment_method.title",
    defaultMessage: "Payment methods",
  },
  paymentMethodNone: {
    id: "settings.billing.payment_method.none",
    defaultMessage: "You have not added any payment methods.",
  },
  paymentMethodAdd: {
    id: "settings.billing.payment_method.action.add",
    defaultMessage: "Add payment method",
  },
  paymentMethodPrimary: {
    id: "settings.billing.payment_method.primary",
    defaultMessage: "Primary",
  },
  paymentMethodMakePrimary: {
    id: "settings.billing.payment_method.action.primary",
    defaultMessage: "Make primary",
  },
  paymentMethodCardDisplay: {
    id: "settings.billing.payment_method.card_display",
    defaultMessage: "{card_brand} ending in {last_four}",
  },
  paymentMethodCardExpiry: {
    id: "settings.billing.payment_method.card_expiry",
    defaultMessage: "Expires {month}/{year}",
  },
});

const paymentMethodTypes = defineMessages({
  visa: {
    id: "settings.billing.payment_method_type.visa",
    defaultMessage: "Visa",
  },
  amex: { id: "settings.billing.payment_method_type.amex", defaultMessage: "American Express" },
  diners: { id: "settings.billing.payment_method_type.diners", defaultMessage: "Diners Club" },
  discover: { id: "settings.billing.payment_method_type.discover", defaultMessage: "Discover" },
  eftpos: { id: "settings.billing.payment_method_type.eftpos", defaultMessage: "EFTPOS" },
  jcb: { id: "settings.billing.payment_method_type.jcb", defaultMessage: "JCB" },
  mastercard: {
    id: "settings.billing.payment_method_type.mastercard",
    defaultMessage: "MasterCard",
  },
  unionpay: { id: "settings.billing.payment_method_type.unionpay", defaultMessage: "UnionPay" },
  paypal: { id: "settings.billing.payment_method_type.paypal", defaultMessage: "PayPal" },
  cashapp: { id: "settings.billing.payment_method_type.cashapp", defaultMessage: "Cash App" },
  amazon_pay: {
    id: "settings.billing.payment_method_type.amazon_pay",
    defaultMessage: "Amazon Pay",
  },
  unknown: {
    id: "settings.billing.payment_method_type.unknown",
    defaultMessage: "Unknown payment method",
  },
});

let stripe = null;
let elements = null;

function loadStripe() {
  try {
    stripe = Stripe(
      "pk_test_51JbFxJJygY5LJFfKV50mnXzz3YLvBVe2Gd1jn7ljWAkaBlRz3VQdxN9mXcPSrFbSqxwAb0svte9yhnsmm7qHfcWn00R611Ce7b",
    );
  } catch {}
}

useHead({
  script: [
    {
      src: "https://js.stripe.com/v3/",
      onload: loadStripe,
    },
  ],
});

onMounted(() => {
  loadStripe();
});

const [
  { data: paymentMethods, refresh: refreshPaymentMethods },
  { data: customer, refresh: refreshCustomer },
  { data: subscriptions, refresh: refreshSubscriptions },
] = await Promise.all([
  useAsyncData("billing/payment_methods", () =>
    useBaseFetch("billing/payment_methods", { internal: true }),
  ),
  useAsyncData("billing/customer", () => useBaseFetch("billing/customer", { internal: true })),
  useAsyncData("billing/subscriptions", () =>
    useBaseFetch("billing/subscriptions", { internal: true }),
  ),
]);

async function refresh() {
  await Promise.all([refreshPaymentMethods(), refreshCustomer()]);
}

const midasProduct = ref(products.find((x) => x.metadata.type === "midas"));
const midasSubscription = computed(() =>
  subscriptions.value.find((x) => midasProduct.value.prices.find((y) => y.id === x.price_id)),
);
const midasSubscriptionPrice = computed(() =>
  midasSubscription.value
    ? midasProduct.value.prices.find((x) => x.id === midasSubscription.value.price_id)
    : null,
);

// Initialize subscription with fake data if redirected from checkout
const route = useRoute();
if (route.query.priceId && route.query.plan && route.query.redirect_status) {
  subscriptions.value.push({
    id: "temp",
    price_id: route.query.priceId,
    interval: route.query.plan,
    created: Date.now(),
    expires: route.query.plan === "yearly" ? Date.now() + 31536000000 : Date.now() + 2629746000,
    status: route.query.redirect_status === "success" ? "active" : "payment_failed",
  });
}

const primaryPaymentMethodId = computed(() => {
  if (
    customer.value &&
    customer.value.invoice_settings &&
    customer.value.invoice_settings.default_payment_method
  ) {
    return customer.value.invoice_settings.default_payment_method;
  } else if (paymentMethods.value && paymentMethods.value[0] && paymentMethods.value[0].id) {
    return paymentMethods.value[0].id;
  } else {
    return null;
  }
});

const addPaymentMethodModal = ref();
const loadingPaymentMethodModal = ref(0);
async function addPaymentMethod() {
  try {
    loadingPaymentMethodModal.value = 0;
    addPaymentMethodModal.value.show();

    const result = await useBaseFetch("billing/payment_method", {
      internal: true,
      method: "POST",
    });

    createStripeElements({ clientSecret: result.client_secret });
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

const loadingAddMethod = ref(false);
async function submit() {
  startLoading();
  loadingAddMethod.value = true;
  const { error } = await stripe.confirmSetup({
    elements,
    confirmParams: {
      return_url: "http://localhost:3000/settings/billing",
    },
  });

  if (error && error.type !== "validation_error") {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: error.message,
      type: "error",
    });
  } else if (!error) {
    await refresh();
    addPaymentMethodModal.value.close();
  }
  loadingAddMethod.value = false;
  stopLoading();
}

const removePaymentMethodIndex = ref();

async function editPaymentMethod(index, primary) {
  startLoading();
  try {
    await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
      internal: true,
      method: "PATCH",
      data: {
        primary,
      },
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

async function removePaymentMethod(index) {
  startLoading();
  try {
    await useBaseFetch(`billing/payment_method/${paymentMethods.value[index].id}`, {
      internal: true,
      method: "DELETE",
    });
    await refresh();
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
  stopLoading();
}

const purchaseModal = ref();
const purchaseModalStep = ref(0);

const selectedPlan = ref("yearly");
const currency = ref("JPY");

const price = ref(midasProduct.value.prices.find((x) => x.currency_code === currency.value));

const clientSecret = ref();
const paymentIntentId = ref();
const confirmationToken = ref();
const tax = ref();
const total = ref();

const selectedPaymentMethod = ref();
const inputtedPaymentMethod = ref();
const selectablePaymentMethods = computed(() => {
  const values = [
    ...paymentMethods.value,
    {
      id: "new",
    },
  ];

  if (inputtedPaymentMethod.value) {
    values.unshift(inputtedPaymentMethod.value);
  }

  return values;
});

const paymentLoading = ref(false);

async function beginPurchaseFlow(skip = false) {
  if (primaryPaymentMethodId.value && skip) {
    paymentLoading.value = true;
    await refreshPayment(null, primaryPaymentMethodId.value);
    paymentLoading.value = false;
    purchaseModalStep.value = 2;
  } else {
    try {
      loadingPaymentMethodModal.value = 0;
      purchaseModalStep.value = 1;

      await nextTick();

      createStripeElements({
        mode: "payment",
        amount: price.value.prices.intervals[selectedPlan.value],
        currency: currency.value.toLowerCase(),
        paymentMethodCreation: "manual",
        setupFutureUsage: "off_session",
      });
    } catch (err) {
      data.$notify({
        group: "main",
        title: "An error occurred",
        text: err.data ? err.data.description : err,
        type: "error",
      });
    }
  }
}

async function createConfirmationToken() {
  const { error, confirmationToken: confirmation } = await stripe.createConfirmationToken({
    elements,
  });

  if (error) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: error.message,
      type: "error",
    });

    return;
  }

  return confirmation.id;
}

async function validatePayment() {
  paymentLoading.value = true;
  const { error: submitError } = await elements.submit();

  if (submitError) {
    return;
  }

  await refreshPayment(await createConfirmationToken());

  elements.update({ currency: price.value.currency_code.toLowerCase(), amount: total.value });

  loadingPaymentMethodModal.value = 0;
  confirmationToken.value = await createConfirmationToken();
  purchaseModalStep.value = 2;
  paymentLoading.value = false;
}

async function selectPaymentMethod(paymentMethod) {
  if (paymentMethod.id === "new") {
    await beginPurchaseFlow(false);
  } else if (inputtedPaymentMethod.value && inputtedPaymentMethod.value.id === paymentMethod.id) {
    paymentLoading.value = true;
    await refreshPayment(confirmationToken.value);
    paymentLoading.value = false;
  } else {
    paymentLoading.value = true;
    await refreshPayment(null, paymentMethod.id);
    paymentLoading.value = false;
  }
}

async function refreshPayment(confirmationId, paymentMethodId) {
  try {
    const base = confirmationId
      ? {
          type: "confirmation_token",
          token: confirmationId,
        }
      : {
          type: "payment_method",
          id: paymentMethodId,
        };

    const result = await useBaseFetch(`billing/payment`, {
      internal: true,
      method: "POST",
      body: {
        product_id: midasProduct.value.id,
        interval: selectedPlan.value,
        existing_payment_intent: paymentIntentId.value,
        ...base,
      },
    });

    if (!paymentIntentId.value) {
      paymentIntentId.value = result.payment_intent_id;
      clientSecret.value = result.client_secret;
    }

    price.value = midasProduct.value.prices.find((x) => x.id === result.price_id);
    currency.value = price.value.currency_code;
    tax.value = result.tax;
    total.value = result.total;

    if (confirmationId) {
      confirmationToken.value = confirmationId;
      inputtedPaymentMethod.value = result.payment_method;
    }

    selectedPaymentMethod.value = result.payment_method;
  } catch (err) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: err.data ? err.data.description : err,
      type: "error",
    });
  }
}

async function submitPayment() {
  paymentLoading.value = true;
  const { error } = await stripe.confirmPayment({
    clientSecret: clientSecret.value,
    confirmParams: {
      confirmation_token: confirmationToken.value,
      return_url: `http://localhost:3000/settings/billing?priceId=${price.value.id}&plan=${selectedPlan.value}`,
    },
  });

  if (error) {
    data.$notify({
      group: "main",
      title: "An error occurred",
      text: error.message,
      type: "error",
    });
  }
  paymentLoading.value = false;
}

function createStripeElements(options) {
  const styles = getComputedStyle(document.body);

  elements = stripe.elements({
    appearance: {
      variables: {
        colorPrimary: styles.getPropertyValue("--color-brand"),
        colorBackground: styles.getPropertyValue("--color-bg"),
        colorText: styles.getPropertyValue("--color-base"),
        colorTextPlaceholder: styles.getPropertyValue("--color-secondary"),
        colorDanger: styles.getPropertyValue("--color-red"),
        fontFamily: styles.getPropertyValue("--font-standard"),
        spacingUnit: "0.25rem",
        borderRadius: "1rem",
      },
    },
    loader: "never",
    ...options,
  });

  const paymentElement = elements.create("payment");
  paymentElement.mount("#payment-element");
  paymentElement.on("ready", () => {
    loadingPaymentMethodModal.value += 1;
  });

  const addressElement = elements.create("address", {
    mode: "billing",
    contacts: [
      ...new Set(
        paymentMethods.value.map((x) => ({
          address: x.billing_details.address,
          email: x.billing_details.email,
          name: x.billing_details.name,
        })),
      ),
    ],
  });
  addressElement.mount("#address-element");
  addressElement.on("ready", () => {
    loadingPaymentMethodModal.value += 1;
  });
}

function formatPrice(price, currency) {
  const formatter = new Intl.NumberFormat(vintl.locale, {
    style: "currency",
    currency,
  });

  const maxDigits = formatter.resolvedOptions().maximumFractionDigits;

  const convertedPrice = price / Math.pow(10, maxDigits);

  return formatter.format(convertedPrice);
}

function calculateSavings(monthlyPlan, annualPlan) {
  const monthlyAnnualized = monthlyPlan * 12;

  return Math.floor(((monthlyAnnualized - annualPlan) / monthlyAnnualized) * 100);
}
</script>
