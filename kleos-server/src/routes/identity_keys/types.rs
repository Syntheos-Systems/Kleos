//! Request types for the /identity-keys endpoints.

use serde::Deserialize;

/// Body for POST /identity-keys/enroll -- registers a new signing key
/// after verifying a proof-of-possession signature.
#[derive(Deserialize)]
pub struct EnrollBody {
    pub tier: String,
    pub algo: String,
    pub pubkey_pem: String,
    pub host_label: String,
    pub label: Option<String>,
    pub serial: Option<String>,
    pub sig_hex: String,
}

/// Body for POST /identity-keys/{id}/revoke.
#[derive(Deserialize)]
pub struct RevokeBody {
    pub reason: Option<String>,
}

/// Query parameters for GET /identity-keys.
#[derive(Deserialize)]
pub struct ListParams {
    pub active_only: Option<bool>,
}

/// Body for POST /identity-keys/invite -- generates a one-time enrollment
/// token for a target user so they can register a FIDO2 security key.
#[derive(Deserialize)]
pub struct CreateInviteBody {
    /// The user who will consume this invite to enroll their key.
    pub user_id: i64,
    /// Auth method the invite is valid for (currently only "fido2").
    #[serde(default = "default_method")]
    pub method: String,
}

/// Defaults the invite method to FIDO2 when the caller omits it.
fn default_method() -> String {
    "fido2".into()
}
