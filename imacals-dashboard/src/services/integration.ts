import { api } from '@/services/api';

// Providers imacals can send (or verify) campaign mail through. Kept in step with
// IntegrationType in imacals-api/src/models/integration.rs.
export type IntegrationType =
  | 'smtp'
  | 'log'
  | 'mailgun'
  | 'mailchimp'
  | 'google'
  | 'outlook'
  | 'zero-bounce'
  | 'custom';

// Providers within a category are interchangeable — exactly one of them is live at a time.
export type IntegrationCategory = 'email' | 'email-validation' | 'other';

export const INTEGRATION_TYPE_LABELS: Record<IntegrationType, string> = {
  smtp:          'SMTP Relay',
  log:           'Log (writes to API log)',
  mailgun:       'Mailgun',
  mailchimp:     'Mailchimp',
  google:        'Gmail',
  outlook:       'Outlook',
  'zero-bounce': 'ZeroBounce',
  custom:        'Custom',
};

export const CATEGORY_LABELS: Record<IntegrationCategory, string> = {
  email:              'Sending Providers',
  'email-validation': 'Address Verification',
  other:              'Other',
};

export const CATEGORY_HINTS: Record<IntegrationCategory, string> = {
  email:              'One provider is live at a time. Campaigns send through whichever is marked Live.',
  'email-validation': 'Runs before a send so dead addresses are skipped instead of bounced.',
  other:              'Free-form configuration that no sender resolves automatically.',
};

// The credential template for a provider type. Served by GET /integrations/provider-types so the
// form is rendered from the backend's field definitions rather than a second copy kept here.
export interface FieldDef {
  name: string;
  label: string;
  type: 'text' | 'url' | 'password';
  is_encrypted: boolean;
  is_required: boolean;
}

export interface ProviderTypeDef {
  integration_type: IntegrationType;
  integration_category: IntegrationCategory;
  fields: FieldDef[];
}

export interface Integration {
  id: string;
  organization_id: string;
  domain_id: string;
  created_by: string;
  name: string;
  slug: string;
  integration_type: IntegrationType;
  integration_category: IntegrationCategory;
  is_enabled: boolean;
  created_at: string;
  updated_at: string;
}

// `value` is null for encrypted attributes: the API withholds ciphertext, so secrets can be
// overwritten but never read back.
export interface IntegrationAttribute {
  id: string;
  created_by: string;
  attributeable_type: string;
  attributeable_id: string;
  name: string;
  value: string | null;
  type: string;
  is_encrypted: boolean;
  created_at: string;
  updated_at: string;
}

export interface InlineAttribute {
  name: string;
  value: string | null;
  type: string;
  is_encrypted: boolean;
}

export interface CreateIntegrationPayload {
  domain_id: string;
  name: string;
  slug: string;
  integration_type: IntegrationType;
  attributes?: InlineAttribute[];
}

export interface UpdateIntegrationPayload {
  domain_id?: string;
  name?: string;
  slug?: string;
  integration_type?: IntegrationType;
}

export interface CreateAttributePayload {
  attributeable_type: string;
  attributeable_id: string;
  name: string;
  value: string | null;
  type: string;
  is_encrypted: boolean;
}

export interface UpdateAttributePayload {
  name?: string;
  value?: string | null;
  type?: string;
  is_encrypted?: boolean;
}

export const integrationService = {
  index: (category?: IntegrationCategory): Promise<Integration[]> =>
    api.get<Integration[]>(category ? `/integrations?category=${category}` : '/integrations'),

  providerTypes: (): Promise<ProviderTypeDef[]> =>
    api.get<ProviderTypeDef[]>('/integrations/provider-types'),

  create: (payload: CreateIntegrationPayload): Promise<Integration> =>
    api.post<Integration>('/integrations', payload),

  update: (id: string, payload: UpdateIntegrationPayload): Promise<Integration> =>
    api.put<Integration>(`/integrations/${id}`, payload),

  // Enabling one provider disables the others in its category — server-side, in one transaction.
  setEnabled: (id: string, isEnabled: boolean): Promise<Integration> =>
    api.put<Integration>(`/integrations/${id}/enabled`, { is_enabled: isEnabled }),

  delete: (id: string): Promise<void> =>
    api.delete<void>(`/integrations/${id}`),

  getAttributes: (id: string): Promise<IntegrationAttribute[]> =>
    api.get<IntegrationAttribute[]>(`/integrations/${id}/attributes`),
};

export const attributeService = {
  create: (payload: CreateAttributePayload): Promise<IntegrationAttribute> =>
    api.post<IntegrationAttribute>('/attributes', payload),

  update: (id: string, payload: UpdateAttributePayload): Promise<IntegrationAttribute> =>
    api.put<IntegrationAttribute>(`/attributes/${id}`, payload),

  delete: (id: string): Promise<void> =>
    api.delete<void>(`/attributes/${id}`),
};
