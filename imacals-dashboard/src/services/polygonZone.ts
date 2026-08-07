import { api } from '@/services/api';

export interface PolygonZone {
  id: string;
  name: string;
  color: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface CreatePolygonZonePayload {
  name: string;
  color: string;
}

export const polygonZoneService = {
  index:  ():                                   Promise<PolygonZone[]> => api.get('/polygon-zones'),
  create: (payload: CreatePolygonZonePayload):  Promise<PolygonZone>   => api.post('/polygon-zones', payload),
  update: (id: string, payload: CreatePolygonZonePayload): Promise<PolygonZone> => api.put(`/polygon-zones/${id}`, payload),
};
