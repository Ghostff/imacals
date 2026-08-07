<script setup lang="ts">
import { ref, type Ref } from 'vue';
import { orderService, type TrackedOrder } from '@/services/order';
import { formatNaira } from '@/services/catalog';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

const reference: Ref<string>          = ref('');
const order: Ref<TrackedOrder | null> = ref(null);
const loading: Ref<boolean>           = ref(false);
const error: Ref<string | null>       = ref(null);
const searched: Ref<boolean>          = ref(false);

async function track(): Promise<void> {
  if (!reference.value.trim()) return;
  error.value    = null;
  order.value    = null;
  loading.value  = true;
  searched.value = true;
  try {
    order.value = await orderService.track(reference.value.trim());
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Could not find that order.';
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="page">
    <header class="head">
      <p class="eyebrow">Track</p>
      <h1 class="section-title">Where is my order?</h1>
      <p class="lede">
        Enter the reference from your confirmation. Phone orders get the same reference — the desk
        reads it back to you on the call.
      </p>
    </header>

    <form class="form" @submit.prevent="track" novalidate>
      <label class="field-label" for="reference">Order reference</label>
      <div class="row">
        <input
          id="reference"
          v-model="reference"
          class="field-input"
          type="text"
          placeholder="IMC-000000"
        />
        <!-- The one Tertiary action on this screen. -->
        <button class="btn-primary" type="submit" :disabled="loading || !reference.trim()">
          {{ loading ? 'Checking…' : 'Track order' }}
        </button>
      </div>
    </form>

    <p v-if="error" class="state-msg state-msg--error">{{ error }}</p>

    <section v-else-if="order" class="result">
      <div class="result-head">
        <div>
          <p class="eyebrow">{{ order.reference }}</p>
          <h2 class="result-status">{{ order.status }}</h2>
        </div>
        <p class="result-total">{{ formatNaira(order.total_kobo) }}</p>
      </div>

      <ol class="timeline">
        <li v-for="event in order.history" :key="event.occurred_at" class="event">
          <p class="event-status">{{ event.status }}</p>
          <p class="event-time">{{ event.occurred_at }}</p>
          <p v-if="event.note" class="event-note">{{ event.note }}</p>
        </li>
      </ol>
    </section>

    <p v-else-if="searched && !loading" class="state-msg">
      No order found for that reference.
    </p>

    <p class="help">
      Cannot find it? Call <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
      with the phone number you ordered on and we will look it up.
    </p>
  </div>
</template>

<style scoped>
.head {
  margin-bottom: var(--spacing-lg);
}

.lede {
  max-width: 56ch;
  color: var(--color-secondary);
  margin-top: var(--spacing-md);
}

.row {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
  max-width: 480px;
}

.row .field-input {
  flex: 1;
  min-width: 200px;
}

.result {
  margin-top: var(--spacing-lg);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.result-head {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: var(--spacing-md);
  padding-bottom: var(--spacing-md);
  border-bottom: 1px solid var(--color-divider);
}

.result-status {
  font-family: var(--font-display);
  font-size: 1.5rem;
  font-weight: 500;
}

.result-total {
  font-family: var(--font-label);
}

.timeline {
  list-style: none;
  margin-top: var(--spacing-md);
}

.event {
  padding: 12px 0 12px var(--spacing-md);
  border-left: 1px solid var(--color-border);
}

.event-status {
  font-family: var(--font-label);
  font-size: 0.875rem;
}

.event-time,
.event-note {
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.help {
  margin-top: var(--spacing-lg);
  font-size: 0.85rem;
  color: var(--color-secondary);
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
