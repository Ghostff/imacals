import { api } from '@/services/api';

export interface OrganizationUserRole {
  id: string;
  name: string;
  title: string;
}

export const organizationUserRoleService = {
  index: (): Promise<OrganizationUserRole[]> => api.get<OrganizationUserRole[]>('/user-roles'),
};
