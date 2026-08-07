import { api } from '@/services/api';

export interface Domain {
  id: string;
  name: string;
  slug: string;
  country_id: string;
  state_id: string | null;
  city_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateDomainPayload {
  name: string;
  slug: string;
  country_id: string;
  state_id: string | null;
  city_id: string | null;
}

export type UpdateDomainPayload = CreateDomainPayload;

export const domainService = {
  index:  (): Promise<Domain[]>                                        => api.get<Domain[]>('/domains'),
  show:   (id: string): Promise<Domain>                               => api.get<Domain>(`/domains/${id}`),
  create: (payload: CreateDomainPayload): Promise<Domain>             => api.post<Domain>('/domains', payload),
  update: (id: string, payload: UpdateDomainPayload): Promise<Domain> => api.put<Domain>(`/domains/${id}`, payload),
  delete: (id: string): Promise<void>                                 => api.delete<void>(`/domains/${id}`),
};
