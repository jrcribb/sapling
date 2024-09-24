/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

pub use anyhow::anyhow;
pub use anyhow::Result;
pub use async_trait::async_trait;
pub use cas_client::CasClient;
pub use once_cell::sync::OnceCell;
pub use tracing;
pub use types::Blake3;
pub use types::CasDigest;
pub use types::CasDigestType;

#[macro_export]
macro_rules! re_client {
    ( $struct:tt ) => {
use re_client_lib::DownloadRequest;
use re_client_lib::TCode;
use re_client_lib::TDigest;
use re_client_lib::THashAlgo;

impl $struct {
    fn client(&self) -> Result<&REClient> {
        self.client.get_or_try_init(|| self.build())
    }
}

fn to_re_digest(d: &$crate::CasDigest) -> TDigest {
    TDigest {
        hash: d.hash.to_hex(),
        size_in_bytes: d.size as i64,
        hash_algo: Some(THashAlgo::BLAKE3),
        ..Default::default()
    }
}

fn from_re_digest(d: &TDigest) -> $crate::Result<$crate::CasDigest> {
    Ok($crate::CasDigest {
        hash: $crate::Blake3::from_hex(d.hash.as_bytes())?,
        size: d.size_in_bytes as u64,
    })
}

#[$crate::async_trait]
impl $crate::CasClient for $struct {
    async fn fetch(
        &self,
        digests: &[$crate::CasDigest],
        log_name: $crate::CasDigestType,
    ) -> $crate::Result<Vec<($crate::CasDigest, Result<Option<Vec<u8>>>)>> {

        $crate::tracing::debug!(target: "cas", concat!(stringify!($struct), " fetching {} {}(s)"), digests.len(), log_name);

        let request = DownloadRequest {
            inlined_digests: Some(digests.iter().map(to_re_digest).collect()),
            throw_on_error: false,
            ..Default::default()
        };

        self.client()?
            .download(self.metadata.clone(), request)
            .await?
            .inlined_blobs
            .unwrap_or_default()
            .into_iter()
            .map(|blob| {
                let digest = from_re_digest(&blob.digest)?;
                match blob.status.code {
                    TCode::OK => Ok((digest, Ok(Some(blob.blob)))),
                    TCode::NOT_FOUND => Ok((digest, Ok(None))),
                    _ => Ok((
                        digest,
                        Err($crate::anyhow!(
                            "bad status (code={}, message={}, group={})",
                            blob.status.code,
                            blob.status.message,
                            blob.status.group
                        )),
                    )),
                }
            })
            .collect::<$crate::Result<Vec<_>>>()
    }
}
};
}

pub fn split_up_to_max_bytes(
    digests: &[CasDigest],
    max_bytes: u64,
) -> impl Iterator<Item = &[CasDigest]> {
    struct Iter<'a> {
        pub digests: &'a [CasDigest],
        pub max_bytes: u64,
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a [CasDigest];

        fn next(&mut self) -> Option<&'a [CasDigest]> {
            if self.digests.is_empty() {
                return None;
            }
            let mut split_at = 0;
            let mut sum = 0;
            for (i, digest) in self.digests.iter().enumerate() {
                sum += digest.size;
                if sum > self.max_bytes {
                    break;
                }
                split_at = i + 1;
            }
            if split_at == 0 {
                // We didn't find a split point meaning that there is a single file above the threshold,
                // so just return this first item.
                split_at = 1;
            }
            let (batch, rest) = self.digests.split_at(split_at);
            self.digests = rest;
            Some(batch)
        }
    }
    Iter { digests, max_bytes }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn split_and_format_results(digests: &[CasDigest], max_bytes: u64) -> String {
        split_up_to_max_bytes(digests, max_bytes)
            .map(|v| {
                v.iter()
                    .map(|d| d.size.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join("|")
    }

    #[test]
    fn test_split_up_to_max_bytes() {
        let hash =
            Blake3::from_str("2078b4229b5353de0268efc7f64b68f3c99fb8829e9c052117b4e1e090b2603a")
                .unwrap();
        let digests = vec![
            CasDigest { hash, size: 200 },
            CasDigest { hash, size: 200 },
            CasDigest { hash, size: 400 },
        ];

        assert_eq!(split_and_format_results(&digests, 200), "200|200|400");
        assert_eq!(split_and_format_results(&digests, 400), "200,200|400");
        assert_eq!(split_and_format_results(&digests, 500), "200,200|400");
        assert_eq!(split_and_format_results(&digests, 5000), "200,200,400");
        assert_eq!(split_and_format_results(&digests, 10), "200|200|400");
    }
}
