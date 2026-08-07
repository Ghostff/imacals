use crate::models::integration::IntegrationType;

// The credential template for a provider. Drives create-time validation on the API side and the
// rendered form on the dashboard side, so both agree on what a provider needs without either
// hardcoding a field list.
pub struct FieldDef {
    pub name: &'static str,
    pub label: &'static str,
    pub field_type: &'static str,
    pub is_encrypted: bool,
    pub is_required: bool,
}

// Plain SMTP relay. In dev these point at the Mailpit catcher (imacals-mail:1025), which accepts any
// credentials over plaintext — hence username/password are optional in the template.
static SMTP_FIELDS: &[FieldDef] = &[
    FieldDef { name: "SMTP_HOST",       label: "Host",         field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "SMTP_PORT",       label: "Port",         field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "SMTP_USERNAME",   label: "Username",     field_type: "text",     is_encrypted: false, is_required: false },
    FieldDef { name: "SMTP_PASSWORD",   label: "Password",     field_type: "password", is_encrypted: true,  is_required: false },
    FieldDef { name: "SMTP_FROM_EMAIL", label: "From Address", field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "SMTP_FROM_NAME",  label: "From Name",    field_type: "text",     is_encrypted: false, is_required: false },
    FieldDef { name: "SMTP_USE_TLS",    label: "Use TLS",      field_type: "text",     is_encrypted: false, is_required: false },
];

// The Log provider writes outgoing mail to the API log instead of sending it — no credentials, no
// network. Only a From address, so rendered messages look like the real thing.
static LOG_FIELDS: &[FieldDef] = &[
    FieldDef { name: "LOG_FROM_EMAIL", label: "From Address", field_type: "text", is_encrypted: false, is_required: true },
];

// Bulk marketing wants its own display name and a Reply-To that is a real monitored mailbox,
// rather than the Mailgun sending subdomain that nobody reads.
static MAILGUN_FIELDS: &[FieldDef] = &[
    FieldDef { name: "MAILGUN_API_KEY",    label: "API Key",          field_type: "password", is_encrypted: true,  is_required: true  },
    FieldDef { name: "MAILGUN_DOMAIN",     label: "Sending Domain",   field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "MAILGUN_REGION",     label: "Region (us / eu)", field_type: "text",     is_encrypted: false, is_required: false },
    FieldDef { name: "MAILGUN_FROM_EMAIL", label: "From Address",     field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "MAILGUN_FROM_NAME",  label: "From Name",        field_type: "text",     is_encrypted: false, is_required: false },
    FieldDef { name: "MAILGUN_REPLY_TO",   label: "Reply-To Address", field_type: "text",     is_encrypted: false, is_required: false },
];

static MAILCHIMP_FIELDS: &[FieldDef] = &[
    FieldDef { name: "MAILCHIMP_API_KEY",    label: "API Key",      field_type: "password", is_encrypted: true,  is_required: true },
    FieldDef { name: "MAILCHIMP_FROM_EMAIL", label: "From Address", field_type: "text",     is_encrypted: false, is_required: true },
    FieldDef { name: "MAILCHIMP_FROM_NAME",  label: "From Name",    field_type: "text",     is_encrypted: false, is_required: true },
];

// Gmail sending over OAuth: the refresh token is the long-lived credential, exchanged for an
// access token per send. FROM_EMAIL must be the mailbox that granted consent.
static GOOGLE_FIELDS: &[FieldDef] = &[
    FieldDef { name: "GOOGLE_CLIENT_ID",     label: "OAuth Client ID",     field_type: "text",     is_encrypted: false, is_required: true },
    FieldDef { name: "GOOGLE_CLIENT_SECRET", label: "OAuth Client Secret", field_type: "password", is_encrypted: true,  is_required: true },
    FieldDef { name: "GOOGLE_REFRESH_TOKEN", label: "Refresh Token",       field_type: "password", is_encrypted: true,  is_required: true },
    FieldDef { name: "GOOGLE_FROM_EMAIL",    label: "From Address",        field_type: "text",     is_encrypted: false, is_required: true },
];

// Microsoft Graph client-credentials flow: the app authenticates with CLIENT_ID + CLIENT_SECRET
// against a TENANT, then sends on behalf of a specific mailbox (USER_ID / USERNAME).
// CLIENT_STATE is the shared secret used to verify Graph webhook callbacks.
static OUTLOOK_FIELDS: &[FieldDef] = &[
    FieldDef { name: "OUTLOOK_CLIENT_ID",     label: "App Client ID",        field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "OUTLOOK_CLIENT_SECRET", label: "App Client Secret",    field_type: "password", is_encrypted: true,  is_required: true  },
    FieldDef { name: "OUTLOOK_TENANT_ID",     label: "Azure Tenant ID",      field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "OUTLOOK_USER_ID",       label: "Mailbox User ID",      field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "OUTLOOK_USERNAME",      label: "Mailbox Username",     field_type: "text",     is_encrypted: false, is_required: true  },
    FieldDef { name: "OUTLOOK_CLIENT_STATE",  label: "Webhook Client State", field_type: "password", is_encrypted: true,  is_required: false },
];

// ZeroBounce verification. One credential; the API base URL is fixed.
static ZERO_BOUNCE_FIELDS: &[FieldDef] = &[
    FieldDef { name: "ZEROBOUNCE_API_KEY", label: "API Key", field_type: "password", is_encrypted: true, is_required: true },
];

pub fn fields_for_type(t: &IntegrationType) -> &'static [FieldDef] {
    match t {
        IntegrationType::Smtp       => SMTP_FIELDS,
        IntegrationType::Log        => LOG_FIELDS,
        IntegrationType::Mailgun    => MAILGUN_FIELDS,
        IntegrationType::Mailchimp  => MAILCHIMP_FIELDS,
        IntegrationType::Google     => GOOGLE_FIELDS,
        IntegrationType::Outlook    => OUTLOOK_FIELDS,
        IntegrationType::ZeroBounce => ZERO_BOUNCE_FIELDS,
        IntegrationType::Custom     => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every provider except Custom must declare a template, or create-time validation silently
    // accepts a credential-less row that then fails at send time.
    #[test]
    fn every_provider_except_custom_has_fields() {
        for t in [
            IntegrationType::Smtp,
            IntegrationType::Log,
            IntegrationType::Mailgun,
            IntegrationType::Mailchimp,
            IntegrationType::Google,
            IntegrationType::Outlook,
            IntegrationType::ZeroBounce,
        ] {
            assert!(!fields_for_type(&t).is_empty(), "{t:?} has no field template");
        }
        assert!(fields_for_type(&IntegrationType::Custom).is_empty());
    }

    // A secret stored in plaintext is the failure this rule exists to prevent.
    #[test]
    fn password_fields_are_always_encrypted() {
        for t in [
            IntegrationType::Smtp,
            IntegrationType::Mailgun,
            IntegrationType::Mailchimp,
            IntegrationType::Google,
            IntegrationType::Outlook,
            IntegrationType::ZeroBounce,
        ] {
            for f in fields_for_type(&t) {
                if f.field_type == "password" {
                    assert!(f.is_encrypted, "{t:?}/{} is a password but not encrypted", f.name);
                }
            }
        }
    }

    // The Log provider is the zero-config fallback — requiring a secret would defeat that.
    #[test]
    fn log_provider_needs_no_secret() {
        assert!(fields_for_type(&IntegrationType::Log).iter().all(|f| !f.is_encrypted));
    }
}
