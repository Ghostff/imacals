import { api } from '@/services/api';

export interface PolygonCoord {
  lat: number;
  lng: number;
}

export interface SavedPolygon {
  id: string;
  created_by: string;
  coordinates: PolygonCoord[];
  city_id: string | null;
  polygon_zone_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreatePolygonPayload {
  coordinates: PolygonCoord[];
  city_id?: string;
}

export interface UpdatePolygonPayload {
  coordinates: PolygonCoord[];
}

export const polygonService = {
  index:      ():                                           Promise<SavedPolygon[]> => api.get('/polygons'),
  create:     (payload: CreatePolygonPayload):              Promise<SavedPolygon>   => api.post('/polygons', payload),
  update:     (id: string, payload: UpdatePolygonPayload):  Promise<SavedPolygon>   => api.put(`/polygons/${id}`, payload),
  delete:     (id: string):                                 Promise<void>           => api.delete(`/polygons/${id}`),
  assignPolygonZone: (id: string, polygon_zone_id: string | null): Promise<SavedPolygon> => api.put(`/polygons/${id}/polygon-zone`, { polygon_zone_id }),
};
