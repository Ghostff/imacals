<script setup lang="ts">
import { SITE } from '@/site';
</script>

<template>
  <div class="page">
    <header class="head">
      <p class="eyebrow">Delivery</p>
      <h1 class="section-title">Everything ships from Aba</h1>
      <p class="lede">
        Our base distribution warehouse is in {{ SITE.warehouse.city }}, {{ SITE.warehouse.state }}.
        Orders are picked there and go out on our own vehicles or a vetted haulier, depending on
        the destination and the size of the load.
      </p>
    </header>

    <section class="block">
      <h2 class="block-title">Coverage and lead times</h2>
      <table class="table">
        <thead>
          <tr>
            <th class="th">Area</th>
            <th class="th">Typical delivery</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in SITE.coverage" :key="row.area">
            <td class="td">{{ row.area }}</td>
            <td class="td td-eta">{{ row.eta }}</td>
          </tr>
        </tbody>
      </table>
      <p class="note">
        Lead times are working days from the point the order is confirmed and paid. Large or
        multi-drop loads are scheduled on the call.
      </p>
    </section>

    <section class="block">
      <h2 class="block-title">Two ways to order</h2>
      <div class="cols">
        <div class="col">
          <p class="eyebrow">Online</p>
          <p class="col-copy">
            Build your order in the catalogue and check out. We call to confirm the load, the
            delivery fee and the payment details before anything is dispatched.
          </p>
        </div>
        <div class="col">
          <p class="eyebrow">By phone</p>
          <p class="col-copy">
            Call <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
            {{ SITE.hours.toLowerCase() }} and read out your list. The desk enters it for you and
            you get the same reference number as an online order.
          </p>
        </div>
      </div>
    </section>

    <section class="block">
      <h2 class="block-title">Collecting instead</h2>
      <p class="note">
        You are welcome to collect from the warehouse. Place the order first so it is picked and
        waiting — {{ SITE.warehouse.line1 }}, {{ SITE.warehouse.city }}, {{ SITE.warehouse.state }}.
      </p>
    </section>
  </div>
</template>

<style scoped>
.head {
  margin-bottom: var(--spacing-lg);
}

.lede {
  max-width: 60ch;
  color: var(--color-secondary);
  margin-top: var(--spacing-md);
}

.block {
  margin-top: var(--spacing-lg);
  padding-top: var(--spacing-lg);
  border-top: 1px solid var(--color-divider);
}

.block-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
  margin-bottom: var(--spacing-md);
}

.table {
  width: 100%;
  border-collapse: collapse;
}

.th {
  text-align: left;
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--color-secondary);
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border);
}

.td {
  padding: 12px 0;
  border-bottom: 1px solid var(--color-divider);
  font-size: 0.9rem;
}

.td-eta {
  font-family: var(--font-label);
}

.note {
  margin-top: var(--spacing-md);
  font-size: 0.85rem;
  color: var(--color-secondary);
  max-width: 60ch;
}

.cols {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: var(--spacing-lg);
}

.col-copy {
  margin-top: var(--spacing-sm);
  font-size: 0.9rem;
  color: var(--color-secondary);
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
