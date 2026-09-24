import { api } from '@/services/api';

export interface OrganizationUserRole {
  id: string;
  name: string;
  title: string;
}

export const DEFAULT_JOB_ROLES: OrganizationUserRole[] = [
  { id: 'order-desk',       name: 'order-desk',       title: 'Order Desk' },
  { id: 'warehouse',        name: 'warehouse',        title: 'Warehouse Picker' },
  { id: 'dispatch',         name: 'dispatch',         title: 'Dispatch Manager' },
  { id: 'rider',            name: 'rider',            title: 'Rider / Driver' },
  { id: 'accounts',         name: 'accounts',         title: 'Accounts / Finance' },
  { id: 'sales-rep',        name: 'sales-rep',        title: 'Sales Representative' },
  { id: 'store-manager',    name: 'store-manager',    title: 'Store Manager' },
  { id: 'customer-support', name: 'customer-support', title: 'Customer Support' },
  { id: 'customer',         name: 'customer',         title: 'Customer' },
];

export const organizationUserRoleService = {
  index: async (): Promise<OrganizationUserRole[]> => {
    try {
      const roles = await api.get<OrganizationUserRole[]>('/user-roles');
      if (roles && roles.length > 0) return roles;
      return DEFAULT_JOB_ROLES;
    } catch {
      return DEFAULT_JOB_ROLES;
    }
  },
};
