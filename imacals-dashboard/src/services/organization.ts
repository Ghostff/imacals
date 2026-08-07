import { api } from '@/services/api';

export interface Org {
  id: string;
  name: string;
  slug: string;
  parent_id: string | null;
  description: string | null;
}

export const organizationService = {
  index: (): Promise<Org[]> => api.get<Org[]>('/organizations'),
};
