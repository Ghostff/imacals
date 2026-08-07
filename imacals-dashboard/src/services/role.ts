import { api } from '@/services/api';

export interface Role {
  id: string;
  name: string;
  title: string;
}

export const roleService = {
  index: (): Promise<Role[]> => api.get<Role[]>('/roles'),
};
