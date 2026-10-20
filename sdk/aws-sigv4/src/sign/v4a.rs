/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::identity::Identity;
use bytes::{BufMut, BytesMut};
use crypto_bigint::{CheckedAdd, CheckedSub, Encoding, U256};
use ecdsa::hazmat::sign_prehashed_rfc6979;
use hmac::{digest::FixedOutput, Hmac, KeyInit, Mac};
use p256::{NistP256, NonZeroScalar};
use sha2::{Digest as _, Sha256};
use std::io::Write;
use std::sync::LazyLock;
use std::time::SystemTime;
use zeroize::Zeroizing;

const ALGORITHM: &[u8] = b"AWS4-ECDSA-P256-SHA256";
/// Size of a P-256 private scalar in bytes.
const P256_PRIVATE_KEY_SIZE: usize = 32;
/// Shortest input [`calculate_signature`] accepts, mirroring `elliptic_curve::SecretKey::MIN_SIZE`.
const P256_PRIVATE_KEY_MIN_SIZE: usize = 24;

static BIG_N_MINUS_2: LazyLock<U256> = LazyLock::new(|| {
    // The N value from section 3.2.1.3 of https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-186.pdf
    // Used as the N value for the algorithm described in section A.2.2 of https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.186-5.pdf
    // *(Basically a prime number blessed by the NSA for use in p256)*
    const ORDER: U256 =
        U256::from_be_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551");
    ORDER.checked_sub(&U256::from(2u32)).unwrap()
});

/// Calculates a Sigv4a signature
///
/// `signing_key` is a big-endian P-256 private scalar, as produced by
/// [`generate_signing_key`].
///
/// Shorter input is accepted down to 24 bytes and left-padded with zeroes, which is what the
/// `SecretKey` this used to go through did.
///
/// # Panics
/// Panics if `signing_key` is not a valid P-256 private key.
pub fn calculate_signature(signing_key: impl AsRef<[u8]>, string_to_sign: &[u8]) -> String {
    let signing_key = signing_key.as_ref();
    assert!(
        (P256_PRIVATE_KEY_MIN_SIZE..=P256_PRIVATE_KEY_SIZE).contains(&signing_key.len()),
        "a P-256 private scalar is {P256_PRIVATE_KEY_MIN_SIZE} to {P256_PRIVATE_KEY_SIZE} bytes, got {}",
        signing_key.len()
    );

    // Sign from the private scalar rather than from a `SigningKey`. Constructing one derives the
    // verifying key, and that costs a P-256 scalar multiplication as expensive as the signature
    // itself while signing never reads it. `NonZeroScalar::from_repr` range-checks the scalar
    // exactly as `SecretKey` does on the `SigningKey` path -- both reject zero and anything at or
    // above the group order. `from_repr` does differ in taking a fixed 32 bytes where
    // `SecretKey::from_slice` takes 24 to 32 and left-pads anything short, so the copy below pads
    // the same way rather than rejecting short input, which would have changed behaviour for a
    // caller passing its own key. The length check above is what keeps the two accepted ranges
    // identical.
    //
    // This is what `ecdsa`'s `PrehashSigner for SigningKey` does with that derivation left out,
    // so the signature is unchanged: same RFC 6979 deterministic nonce, same empty additional
    // data.
    //
    // Both copies this function owns, `repr` and `scalar`, are wiped on the way out; the
    // `signing_key` slice is the caller's and is left alone. The old code got the wipe for free:
    // `SigningKey` implements `ZeroizeOnDrop` and its `Drop` zeroizes the secret scalar, whereas
    // `NonZeroScalar` implements `Zeroize` but has no `Drop`, so dropping one leaves the scalar
    // in memory. `Zeroizing` supplies the `Drop` that type is missing. The `FieldBytes` that
    // `from_repr` takes by value is still an unwiped stack copy; that one is not reachable
    // through this API.
    let mut repr = Zeroizing::new([0u8; P256_PRIVATE_KEY_SIZE]);
    repr[P256_PRIVATE_KEY_SIZE - signing_key.len()..].copy_from_slice(signing_key);
    let scalar = Zeroizing::new(
        Option::<NonZeroScalar>::from(NonZeroScalar::from_repr((*repr).into()))
            .expect("signing key is a valid P-256 private scalar"),
    );

    let z = Sha256::digest(string_to_sign);
    let (signature, _recovery_id) = sign_prehashed_rfc6979::<NistP256, Sha256>(&scalar, &z, &[]);

    hex::encode(signature.to_der().as_bytes())
}

/// Generates a signing key for Sigv4a signing.
pub fn generate_signing_key(access_key: &str, secret_access_key: &str) -> impl AsRef<[u8]> {
    // Capacity is the secret access key length plus the length of "AWS4A"
    let mut input_key = Zeroizing::new(Vec::with_capacity(secret_access_key.len() + 5));
    write!(input_key, "AWS4A{secret_access_key}").unwrap();

    // Capacity is the access key length plus the counter byte
    let mut kdf_context = Zeroizing::new(Vec::with_capacity(access_key.len() + 1));
    let mut counter = Zeroizing::new(1u8);
    let key = loop {
        write!(kdf_context, "{access_key}").unwrap();
        kdf_context.push(*counter);

        let mut fis = ALGORITHM.to_vec();
        fis.push(0);
        fis.append(&mut kdf_context);
        fis.put_i32(256);

        let mut mac =
            Hmac::<Sha256>::new_from_slice(&input_key).expect("HMAC can take key of any size");

        let mut buf = BytesMut::new();
        buf.put_i32(1);
        buf.put_slice(&fis);
        mac.update(&buf);
        let k0 = U256::from_be_bytes(mac.finalize_fixed().into());

        // It would be more secure for this to be a constant time comparison, but because this
        // is for client usage, that's not as big a deal.
        if k0 <= *BIG_N_MINUS_2 {
            let pk = k0
                .checked_add(&U256::ONE)
                .expect("k0 is always less than U256::MAX");
            // Return the scalar itself rather than round-tripping it through a `SigningKey`.
            // Building one derives the verifying key, which costs a P-256 scalar
            // multiplication that nothing here uses: the only caller hands these bytes to
            // `calculate_signature`, and the loop's bound on `k0` already makes `pk` a valid
            // private scalar. Staying in `Zeroizing` also keeps the key out of a plain
            // `FieldBytes` that would not be wiped on drop.
            break Zeroizing::new(pk.to_be_bytes());
        }

        *counter = counter
            .checked_add(1)
            .expect("counter will never get to 255");
    };

    key
}

/// Parameters to use when signing.
#[derive(Debug)]
#[non_exhaustive]
pub struct SigningParams<'a, S> {
    /// The identity to use when signing a request
    pub(crate) identity: &'a Identity,

    /// Region set to sign for.
    pub(crate) region_set: &'a str,
    /// Service Name to sign for.
    ///
    /// NOTE: Endpoint resolution rules may specify a name that differs from the typical service name.
    pub(crate) name: &'a str,
    /// Timestamp to use in the signature (should be `SystemTime::now()` unless testing).
    pub(crate) time: SystemTime,

    /// Additional signing settings. These differ between HTTP and Event Stream.
    pub(crate) settings: S,
}

pub(crate) const ECDSA_256: &str = "AWS4-ECDSA-P256-SHA256";

impl<S> SigningParams<'_, S> {
    /// Returns the region that will be used to sign SigV4a requests
    pub fn region_set(&self) -> &str {
        self.region_set
    }

    /// Returns the service name that will be used to sign requests
    pub fn name(&self) -> &str {
        self.name
    }

    /// Return the name of the algorithm used to sign requests
    pub fn algorithm(&self) -> &'static str {
        ECDSA_256
    }
}

impl<'a, S: Default> SigningParams<'a, S> {
    /// Returns a builder that can create new `SigningParams`.
    pub fn builder() -> signing_params::Builder<'a, S> {
        Default::default()
    }
}

/// Builder and error for creating [`SigningParams`]
pub mod signing_params {
    use super::SigningParams;
    use aws_smithy_runtime_api::client::identity::Identity;
    use std::error::Error;
    use std::fmt;
    use std::time::SystemTime;

    /// [`SigningParams`] builder error
    #[derive(Debug)]
    pub struct BuildError {
        reason: &'static str,
    }
    impl BuildError {
        fn new(reason: &'static str) -> Self {
            Self { reason }
        }
    }

    impl fmt::Display for BuildError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.reason)
        }
    }

    impl Error for BuildError {}

    /// Builder that can create new [`SigningParams`]
    #[derive(Debug, Default)]
    pub struct Builder<'a, S> {
        identity: Option<&'a Identity>,
        region_set: Option<&'a str>,
        name: Option<&'a str>,
        time: Option<SystemTime>,
        settings: Option<S>,
    }

    impl<'a, S> Builder<'a, S> {
        builder_methods!(
            set_identity,
            identity,
            &'a Identity,
            "Sets the identity (required)",
            set_region_set,
            region_set,
            &'a str,
            "Sets the region set (required)",
            set_name,
            name,
            &'a str,
            "Sets the name (required)",
            set_time,
            time,
            SystemTime,
            "Sets the time to be used in the signature (required)",
            set_settings,
            settings,
            S,
            "Sets additional signing settings (required)"
        );

        /// Builds an instance of [`SigningParams`]. Will yield a [`BuildError`] if
        /// a required argument was not given.
        pub fn build(self) -> Result<SigningParams<'a, S>, BuildError> {
            Ok(SigningParams {
                identity: self
                    .identity
                    .ok_or_else(|| BuildError::new("identity is required"))?,
                region_set: self
                    .region_set
                    .ok_or_else(|| BuildError::new("region_set is required"))?,
                name: self
                    .name
                    .ok_or_else(|| BuildError::new("name is required"))?,
                time: self
                    .time
                    .ok_or_else(|| BuildError::new("time is required"))?,
                settings: self
                    .settings
                    .ok_or_else(|| BuildError::new("settings are required"))?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_signature, generate_signing_key, P256_PRIVATE_KEY_MIN_SIZE, P256_PRIVATE_KEY_SIZE,
    };
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::{DerSignature, SigningKey};
    use p256::elliptic_curve::sec1::ToSec1Point;

    /// `elliptic_curve::SecretKey::MIN_SIZE` is a private associated constant, so the 24 in
    /// [`P256_PRIVATE_KEY_MIN_SIZE`] is a copy this crate cannot reference. Pin both ends of the
    /// range against the `SigningKey` path: if upstream moves that constant, the length check
    /// above stops matching what `SecretKey::from_slice` accepts, and this fails rather than
    /// silently diverging.
    #[test]
    fn the_accepted_length_range_matches_secret_key() {
        let key = generate_signing_key("AKIDEXAMPLE", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY");
        let full = key.as_ref();

        assert!(
            SigningKey::from_slice(&full[..P256_PRIVATE_KEY_MIN_SIZE]).is_ok(),
            "{P256_PRIVATE_KEY_MIN_SIZE} bytes should be the shortest accepted key"
        );
        assert!(
            SigningKey::from_slice(&full[..P256_PRIVATE_KEY_MIN_SIZE - 1]).is_err(),
            "one byte below {P256_PRIVATE_KEY_MIN_SIZE} should be refused"
        );

        let too_long = [full, full].concat();
        assert!(
            SigningKey::from_slice(&too_long[..P256_PRIVATE_KEY_SIZE + 1]).is_err(),
            "one byte above {P256_PRIVATE_KEY_SIZE} should be refused"
        );
    }

    /// `SecretKey::from_slice` accepts 24 to 32 bytes and left-pads anything short, so the
    /// `SigningKey` path signed a short key rather than rejecting it. Signing from the scalar
    /// takes a fixed 32 bytes, so this function pads instead. Every accepted length has to still
    /// produce what the old path produced, since a caller may pass its own key.
    #[test]
    fn short_keys_match_the_signing_key_path() {
        let key = generate_signing_key("AKIDEXAMPLE", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY");
        let string_to_sign = b"AWS4-ECDSA-P256-SHA256\n20260101T000000Z";

        for len in P256_PRIVATE_KEY_MIN_SIZE..=P256_PRIVATE_KEY_SIZE {
            let short = &key.as_ref()[..len];
            let expected: DerSignature = SigningKey::from_slice(short)
                .expect("a left-padded short key is a valid P-256 private key")
                .sign(string_to_sign);
            assert_eq!(
                hex::encode(expected.as_bytes()),
                calculate_signature(short, string_to_sign),
                "signature diverged for a {len}-byte key"
            );
        }
    }

    const CREDENTIALS: &[(&str, &str)] = &[
        (
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        ),
        ("AKIDEXAMPLE", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY"),
        (
            "AKIAIOSFODNN7EXAMPLF",
            "je7MtGbClwBF/2Zp9Utk/h3yCo8nvbEXAMPLEKEY",
        ),
    ];

    const STRINGS_TO_SIGN: &[&[u8]] = &[
        b"",
        b"AWS4-ECDSA-P256-SHA256\n20260101T000000Z\n20260101/lambda/aws4_request\n\
          a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90",
        &[0xff; 64],
    ];

    /// Signing from the private scalar has to agree with signing through a `SigningKey`, byte for
    /// byte, across every credential and string to sign below. That equality is the whole
    /// argument for skipping the verifying-key derivation: the only thing dropped is a scalar
    /// multiplication whose result signing never reads.
    ///
    /// It says nothing about the derivation itself, since both sides start from the same
    /// `generate_signing_key` output and a change there moves both;
    /// `derived_key_matches_the_published_test_vector` is what pins that. Nor does it cover short
    /// keys, which `short_keys_match_the_signing_key_path` does.
    #[test]
    fn signature_matches_the_signing_key_path() {
        for (access_key, secret_key) in CREDENTIALS {
            let key = generate_signing_key(access_key, secret_key);
            for string_to_sign in STRINGS_TO_SIGN {
                let expected: DerSignature = SigningKey::from_slice(key.as_ref())
                    .expect("derived key is a valid P-256 private key")
                    .sign(string_to_sign);
                assert_eq!(
                    hex::encode(expected.as_bytes()),
                    calculate_signature(&key, string_to_sign),
                    "signature diverged for access key {access_key}"
                );
            }
        }
    }

    /// Pins the key derivation to the suite's published key.
    ///
    /// Every `aws-signing-test-suite/v4a/*/public-key.json` in this crate holds the same public
    /// key, and the scalar the KDF derives for `AKIDEXAMPLE` is its private half. The X and Y
    /// asserted below are the values in those files; the private scalar is not published
    /// anywhere, so it is asserted as the value that derives them.
    ///
    /// Nothing else in the crate pins this. The suite tests in `http_request::test` verify each
    /// signature with a key derived from this same output, so a change to the derivation moves
    /// both sides and they still pass; that file carries a standing `TODO(sigv4a)` about using
    /// `public-key.json` as the verifying key instead. A round trip through
    /// `SigningKey::from_slice` and `to_bytes` cannot pin it either: that pair returns what it is
    /// given, so it compares the key against itself.
    #[test]
    fn derived_key_matches_the_published_test_vector() {
        let key = generate_signing_key("AKIDEXAMPLE", "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY");

        assert_eq!(
            "7efc8c0e65a324242818c5a50c891c6060b6a00717b7ba3cbe3c5d765be9259c",
            hex::encode(key.as_ref()),
            "the derived signing key changed"
        );

        let point = SigningKey::from_slice(key.as_ref())
            .expect("derived key is a valid P-256 private key")
            .verifying_key()
            .as_affine()
            .to_sec1_point(false);
        assert_eq!(
            "b6618f6a65740a99e650b33b6b4b5bd0d43b176d721a3edfea7e7d2d56d936b1",
            hex::encode(point.x().expect("uncompressed point has an x coordinate")),
            "public key no longer matches the test suite's public-key.json"
        );
        assert_eq!(
            "865ed22a7eadc9c5cb9d2cbaca1b3699139fedc5043dc6661864218330c8e518",
            hex::encode(point.y().expect("uncompressed point has a y coordinate")),
            "public key no longer matches the test suite's public-key.json"
        );
    }
}
