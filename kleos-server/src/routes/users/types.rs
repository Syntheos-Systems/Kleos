//! Request and query types for the /users endpoints.

use serde::Deserialize;

/// Body for POST /users -- creates a new user account.
#[derive(Deserialize)]
pub struct CreateUserBody {
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    /// Role label stored on the user record. Defaults to "user" in the handler.
    #[serde(default)]
    pub role: Option<String>,
}

/// Query parameters for GET /users.
#[derive(Deserialize)]
pub struct ListUsersParams {
    /// When true, includes soft-deleted users in the response.
    #[serde(default)]
    pub include_inactive: Option<bool>,
}
