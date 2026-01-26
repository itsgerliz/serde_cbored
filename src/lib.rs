//! A CBOR implementation for the serde framework

#[cfg(not(any(feature = "ser", feature = "de")))]
compile_error!("You must enable either \"ser\" or \"de\" features");

#[cfg(feature = "de")]
pub mod de;
pub mod error;
#[cfg(feature = "ser")]
pub mod ser;
