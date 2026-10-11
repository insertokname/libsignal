//
// Copyright 2020-2021 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

#![allow(clippy::missing_safety_doc)]
#![deny(clippy::unwrap_used)]

#[cfg(not(any(feature = "ffi", feature = "jni", feature = "node")))]
compile_error!("Feature \"ffi\", \"jni\", or \"node\" must be enabled for this crate.");

#[cfg(feature = "ffi")]
pub use libsignal_bridge_types::ffi;
pub use libsignal_bridge_types::support::IllegalArgumentError;
pub use libsignal_bridge_types::{
    bridge_as_handle, bridge_deserialize, bridge_fixed_length_serializable_fns, bridge_get,
    bridge_handle_fns, bridge_serializable_handle_fns, describe_panic, io,
};
#[cfg(feature = "jni")]
pub use libsignal_bridge_types::{jni, jni_args};
#[cfg(feature = "node")]
pub use libsignal_bridge_types::{node, node_register};

pub mod logging;

pub mod crypto;
pub mod protocol;

// Desktop does not make use of device transfer certificates
#[cfg(any(feature = "jni", feature = "ffi"))]
pub mod device_transfer;

#[cfg(not(target_family = "wasm"))] // WEB-FORK: no networking on wasm
mod cds2;
#[cfg(not(target_family = "wasm"))] // WEB-FORK: no networking on wasm
mod hsm_enclave;
#[cfg(not(target_family = "wasm"))] // WEB-FORK: no networking on wasm
mod sgx_session;

pub mod zkgroup;

#[cfg(not(target_family = "wasm"))] // WEB-FORK: no networking on wasm
pub mod net;

mod account_keys;

#[cfg(not(target_family = "wasm"))] // WEB-FORK: no networking on wasm
mod svr2;

pub mod incremental_mac;
pub mod message_backup;
pub mod usernames;

#[cfg(feature = "signal-media")]
pub mod media;

pub mod support;
