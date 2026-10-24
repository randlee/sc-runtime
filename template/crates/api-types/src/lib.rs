//! Widget documents shared by the service and its adapters.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Body of `widget.create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct CreateWidget {
    /// Caller-supplied widget name. Blank names are rejected.
    pub name: String,
}

/// Body of `widget.get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct GetWidget {
    /// Identifier returned by `widget.create`.
    pub id: String,
}

/// One stored widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct Widget {
    /// Stable identifier assigned by the store.
    pub id: String,
    /// Name stored for this widget.
    pub name: String,
}
