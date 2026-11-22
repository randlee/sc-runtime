#![doc = include_str!("../README.md")]

mod app_name;
mod error;
mod layers;
mod load;
mod provenance;
mod sources;
mod standard;

#[doc(inline)]
pub use app_name::AppName;
#[doc(inline)]
pub use error::ConfigError;
#[doc(inline)]
pub use layers::{Layer, LayerSource, Layers, Resolved};
#[doc(inline)]
pub use load::{Loader, load};
#[doc(inline)]
pub use provenance::{KeyPath, Provenance, ResolvedRow, Source, SourceKind};
#[doc(inline)]
pub use sources::{EnvSource, FileSource, ValueSource};
#[doc(inline)]
pub use standard::{After, Slot, StandardContext, StandardLayers};
