import { api } from '@/services/api';

export interface GeoCountry {
  id: string;
  name: string;
  iso2_code: string;
  iso3_code: string;
}

export interface GeoState {
  id: string;
  country_id: string;
  name: string;
  code: string;
  latitude: number | null;
  longitude: number | null;
}

export interface GeoCity {
  id: string;
  state_id: string;
  name: string;
  latitude: number | null;
  longitude: number | null;
}

export const geo = {
  countries: (): Promise<GeoCountry[]>                      => api.get('/geo/countries'),
  states:    (countryId: string): Promise<GeoState[]>       => api.get(`/geo/countries/${countryId}/states`),
  cities:    (stateId: string): Promise<GeoCity[]>          => api.get(`/geo/states/${stateId}/cities`),
};
