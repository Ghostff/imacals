import { api } from '@/services/api';

export interface PolygonNeighbor {
  polygon_id: string;
  neighbor_polygon_id: string;
}

export interface CreateNeighborPayload {
  polygon_id: string;
  neighbor_polygon_id: string;
}

export const neighborService = {
  index:  ():                               Promise<PolygonNeighbor[]>   => api.get('/polygon-neighbors'),
  create: (payload: CreateNeighborPayload): Promise<{ message: string }> => api.post('/polygon-neighbors', payload),
  delete: (polygonId: string, neighborId: string): Promise<void>         => api.delete(`/polygon-neighbors/${polygonId}/${neighborId}`),
};
