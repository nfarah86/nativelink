// Copyright 2021 Nathan (Blaise) Bruer.  All rights reserved.

use std::default::Default;
use std::pin::Pin;

use common::DigestInfo;
use error::{Code, Error, ResultExt};
use prost::Message;
use store::Store;

// NOTE(blaise.bruer) From some local testing it looks like action cache items are rarely greater than
// 1.2k. Giving a bit more just in case to reduce allocs.
pub const ESTIMATED_DIGEST_SIZE: usize = 2048;

/// This is more of a safety check. We are going to collect this entire message
/// into memory. If we don't bound the max size of the object we enable users
/// to use up all the memory on this machine.
const MAX_ACTION_MSG_SIZE: usize = 10 << 20; // 10mb.

/// Attempts to fetch the digest contents from a store into the associated proto.
pub async fn get_and_decode_digest<T: Message + Default>(
    store: Pin<&dyn Store>,
    digest: &DigestInfo,
) -> Result<T, Error> {
    let mut store_data_resp = store
        .get_part_unchunked(
            digest.clone(),
            0,
            Some(MAX_ACTION_MSG_SIZE),
            Some(ESTIMATED_DIGEST_SIZE),
        )
        .await;
    if let Err(err) = &mut store_data_resp {
        if err.code == Code::NotFound {
            // Trim the error code. Not Found is quite common and we don't want to send a large
            // error (debug) message for something that is common. We resize to just the last
            // message as it will be the most relevant.
            err.messages.resize_with(1, || "".to_string());
        }
    }
    let store_data = store_data_resp?;

    T::decode(store_data).err_tip_with_code(|e| (Code::NotFound, format!("Stored value appears to be corrupt: {}", e)))
}