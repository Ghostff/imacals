import { api } from '@/services/api';

export interface EligibleRole {
  id:                   string;
  name:                 string;
  title:                string;
  description:          string;
  organization_id:      string | null;
  system_user_eligible: boolean;
  created_at:           string;
  updated_at:           string;
}

export interface DomainSystemUser {
  id:              string;
  domain_id:       string;
  domain_name:     string;
  user_id:         string;
  user_first_name: string;
  user_last_name:  string;
  user_email:      string;
  user_role_id:    string;
  role_name:       string;
  role_title:      string;
  created_by:      string;
  created_at:      string;
  updated_at:      string;
}

export interface UpsertSystemUserPayload {
  domain_id:    string;
  user_id:      string;
  user_role_id: string;
}

export const systemUserService = {
  index:         (): Promise<DomainSystemUser[]>     => api.get<DomainSystemUser[]>('/domain-system-users'),
  eligibleRoles: (): Promise<EligibleRole[]>          => api.get<EligibleRole[]>('/domain-system-users/eligible-roles'),
  upsert:        (payload: UpsertSystemUserPayload): Promise<DomainSystemUser> =>
    api.post<DomainSystemUser>('/domain-system-users', payload),
  delete:        (id: string): Promise<void>          => api.delete<void>(`/domain-system-users/${id}`),
};
