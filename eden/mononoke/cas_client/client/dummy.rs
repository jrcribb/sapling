/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

use std::marker::PhantomData;

use anyhow::Error;
use bytes::Bytes;
use futures::Stream;
use mononoke_types::MononokeDigest;

use crate::CasClient;

pub struct DummyCasClient<'a> {
    _marker: PhantomData<&'a ()>,
}

/// A CasClient that does nothing. All operations are essentially a no-op.
#[async_trait::async_trait]
impl<'a> CasClient for DummyCasClient<'a> {
    async fn streaming_upload_blob(
        &self,
        _digest: &MononokeDigest,
        _bytes_stream: impl Stream<Item = Result<Bytes, Error>> + std::marker::Send,
    ) -> Result<(), Error> {
        Ok(())
    }
    async fn upload_blob(&self, _digest: &MononokeDigest, _bytes: Bytes) -> Result<(), Error> {
        Ok(())
    }

    async fn upload_blobs(&self, _blobs: Vec<(MononokeDigest, Bytes)>) -> Result<(), Error> {
        Ok(())
    }

    async fn lookup_blob(&self, _digest: &MononokeDigest) -> Result<bool, Error> {
        Ok(false)
    }
}
