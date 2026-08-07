<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import { geo, type GeoCountry, type GeoState, type GeoCity } from '@/services/geo';

const emit = defineEmits<{
  confirm: [lat: number, lng: number, cityId: string];
}>();

const countries: Ref<GeoCountry[]> = ref([]);
const states:    Ref<GeoState[]>   = ref([]);
const cities:    Ref<GeoCity[]>    = ref([]);

const selectedCountryId: Ref<string> = ref('');
const selectedStateId:   Ref<string> = ref('');
const selectedCityId:    Ref<string> = ref('');

const loadingStates: Ref<boolean> = ref(false);
const loadingCities: Ref<boolean> = ref(false);

const selectedCity = (): GeoCity | undefined =>
  cities.value.find((c: GeoCity): boolean => c.id === selectedCityId.value);

const canConfirm = (): boolean => {
  const city = selectedCity();
  return city?.latitude != null && city?.longitude != null;
};

async function loadStates(countryId: string): Promise<void> {
  loadingStates.value = true;
  states.value  = [];
  cities.value  = [];
  selectedStateId.value = '';
  selectedCityId.value  = '';
  try {
    states.value = await geo.states(countryId);
  } finally {
    loadingStates.value = false;
  }
}

async function loadCities(stateId: string): Promise<void> {
  loadingCities.value = true;
  cities.value = [];
  selectedCityId.value = '';
  try {
    cities.value = await geo.cities(stateId);
  } finally {
    loadingCities.value = false;
  }
}

async function onCountryChange(): Promise<void> {
  if (selectedCountryId.value) await loadStates(selectedCountryId.value);
}

async function onStateChange(): Promise<void> {
  if (selectedStateId.value) await loadCities(selectedStateId.value);
}

function confirm(): void {
  const city = selectedCity();
  if (city?.latitude == null || city?.longitude == null) return;
  emit('confirm', city.latitude, city.longitude, city.id);
}

onMounted(async (): Promise<void> => {
  countries.value = await geo.countries();

  // Pre-select United States
  const us = countries.value.find((c: GeoCountry): boolean => c.iso2_code === 'US');
  if (!us) return;
  selectedCountryId.value = us.id;
  await loadStates(us.id);

  // Pre-select Texas
  const tx = states.value.find((s: GeoState): boolean => s.code === 'TX');
  if (!tx) return;
  selectedStateId.value = tx.id;
  await loadCities(tx.id);
});
</script>

<template>
  <div class="overlay">
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <p class="modal-label">Admin / Map</p>
      <h2 id="modal-title" class="modal-title">Choose a location</h2>
      <p class="modal-sub">Select a city to centre the map before you start drawing.</p>

      <div class="fields">
        <div class="field">
          <label class="field-label" for="country">Country</label>
          <select
            id="country"
            v-model="selectedCountryId"
            class="field-select"
            @change="onCountryChange"
          >
            <option value="" disabled>Select country</option>
            <option v-for="c in countries" :key="c.id" :value="c.id">{{ c.name }}</option>
          </select>
        </div>

        <div class="field">
          <label class="field-label" for="state">State / Province</label>
          <select
            id="state"
            v-model="selectedStateId"
            class="field-select"
            :disabled="!selectedCountryId || loadingStates || states.length === 0"
            @change="onStateChange"
          >
            <option value="" disabled>{{ loadingStates ? 'Loading…' : 'Select state' }}</option>
            <option v-for="s in states" :key="s.id" :value="s.id">{{ s.name }}</option>
          </select>
        </div>

        <div class="field">
          <label class="field-label" for="city">City</label>
          <select
            id="city"
            v-model="selectedCityId"
            class="field-select"
            :disabled="!selectedStateId || loadingCities || cities.length === 0"
          >
            <option value="" disabled>{{ loadingCities ? 'Loading…' : 'Select city' }}</option>
            <option v-for="c in cities" :key="c.id" :value="c.id">{{ c.name }}</option>
          </select>
        </div>
      </div>

      <button class="btn-confirm" :disabled="!canConfirm()" @click="confirm">
        Go to location
      </button>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: absolute;
  inset: 0;
  background-color: rgba(26, 28, 30, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal {
  background-color: var(--color-surface);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
  width: 100%;
  max-width: 440px;
  box-shadow: 0 8px 32px rgba(26, 28, 30, 0.16);
}

.modal-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-sm);
}

.modal-title {
  font-family: var(--font-display);
  font-size: 2rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
  margin-bottom: var(--spacing-sm);
}

.modal-sub {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-lg);
}

.fields {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.field-select {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  background-color: var(--color-surface);
  border: 1px solid #E0DED9;
  border-radius: var(--rounded-md);
  padding: 10px var(--spacing-md);
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'%3E%3Cpath d='M1 1l5 5 5-5' stroke='%236C7278' stroke-width='1.5' fill='none' stroke-linecap='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right var(--spacing-md) center;
  cursor: pointer;
}

.field-select:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.field-select:focus {
  outline: none;
  border-color: var(--color-primary);
}

.btn-confirm {
  width: 100%;
  padding: 12px var(--spacing-lg);
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: #FFFFFF;
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-confirm:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-confirm:not(:disabled):hover {
  opacity: 0.88;
}
</style>
