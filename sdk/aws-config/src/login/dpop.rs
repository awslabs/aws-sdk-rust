/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
use crate::login::token::LoginTokenError;
use aws_sdk_signin::config::auth::Params;
use aws_smithy_json::serialize::JsonObjectWriter;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::auth::{
    AuthScheme, AuthSchemeEndpointConfig, AuthSchemeId, AuthSchemeOption, AuthSchemeOptionsFuture,
    Sign,
};
use aws_smithy_runtime_api::client::identity::{
    Identity, IdentityCacheLocation, IdentityCachePartition, IdentityFuture, ResolveIdentity,
    SharedIdentityResolver,
};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::runtime_components::{GetIdentityResolver, RuntimeComponents};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::Number;
use p256::ecdsa::signature::RandomizedSigner;
use p256::ecdsa::{Signature, SigningKey};
use p256::elliptic_curve::sec1::ToSec1Point;
use p256::SecretKey;
use std::sync::{Arc, LazyLock};
use std::time::SystemTime;

pub(super) struct Header {
    pub x_b64: String,
    pub y_b64: String,
}

impl Header {
    fn to_json(&self) -> String {
        let mut header = String::new();
        let mut writer = JsonObjectWriter::new(&mut header);
        writer.key("typ").string("dpop+jwt");
        writer.key("alg").string("ES256");
        let mut jwk = writer.key("jwk").start_object();
        jwk.key("kty").string("EC");
        jwk.key("x").string(&self.x_b64);
        jwk.key("y").string(&self.y_b64);
        jwk.key("crv").string("P-256");
        jwk.finish();
        writer.finish();
        header
    }
}

pub(super) struct Payload {
    pub jti: String,
    pub iat: u64,
    pub htu: String,
}

impl Payload {
    fn to_json(&self) -> String {
        let mut payload = String::new();
        let mut writer = JsonObjectWriter::new(&mut payload);
        writer.key("jti").string(&self.jti);
        writer.key("htm").string("POST");
        writer.key("htu").string(&self.htu);
        writer.key("iat").number(Number::PosInt(self.iat));
        writer.finish();
        payload
    }
}

fn header(private_key: &SecretKey) -> Result<Header, LoginTokenError> {
    let public_key = private_key.public_key();
    let point = public_key.to_sec1_point(false);

    let x_bytes = point
        .x()
        .ok_or_else(|| LoginTokenError::other("invalid private key: x coordinate", None))?;
    let y_bytes = point
        .y()
        .ok_or_else(|| LoginTokenError::other("invalid private key: y coordinate", None))?;

    Ok(Header {
        x_b64: base64_simd::URL_SAFE_NO_PAD.encode_to_string(x_bytes),
        y_b64: base64_simd::URL_SAFE_NO_PAD.encode_to_string(y_bytes),
    })
}

pub(super) fn payload(jti: String, iat: u64, htu: &str) -> Payload {
    Payload {
        jti,
        iat,
        htu: htu.to_string(),
    }
}

fn build_message(header: &Header, payload: &Payload) -> String {
    let header_json = header.to_json();
    let payload_json = payload.to_json();

    let header_b64 = base64_simd::URL_SAFE_NO_PAD.encode_to_string(header_json.as_bytes());
    let payload_b64 = base64_simd::URL_SAFE_NO_PAD.encode_to_string(payload_json.as_bytes());
    format!("{header_b64}.{payload_b64}")
}

fn sign(message: &str, private_key: &SecretKey) -> Result<String, LoginTokenError> {
    let signing_key = SigningKey::from(private_key);
    let mut rng: rand::rngs::StdRng = rand::make_rng();
    let signature: Signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
    let signature_b64 = base64_simd::URL_SAFE_NO_PAD.encode_to_string(signature.to_bytes());
    Ok(format!("{message}.{signature_b64}"))
}

/// Calculate DPoP HTTP header using the private key.
///
/// See [RFC 9449: OAuth 2.0 Demonstrating Proof of Possession (DPoP)](https://datatracker.ietf.org/doc/html/rfc9449)
pub(super) fn calculate(
    private_key: &SecretKey,
    endpoint: &str,
    now: SystemTime,
) -> Result<String, LoginTokenError> {
    let header = header(private_key)?;
    let jti = uuid::Uuid::new_v4().to_string();
    let iat = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| LoginTokenError::other("system time before UNIX epoch", Some(e.into())))?
        .as_secs();
    let payload = payload(jti, iat, endpoint);
    let message = build_message(&header, &payload);
    sign(&message, private_key)
}

/// Auth scheme ID for DPoP
const SCHEME_ID: AuthSchemeId = AuthSchemeId::new("dpop");

#[derive(Debug, Default)]
pub(super) struct DPoPAuthSchemeOptionResolver;

impl aws_sdk_signin::config::auth::ResolveAuthScheme for DPoPAuthSchemeOptionResolver {
    fn resolve_auth_scheme<'a>(
        &'a self,
        _params: &'a Params,
        _cfg: &'a ConfigBag,
        _runtime_components: &'a RuntimeComponents,
    ) -> AuthSchemeOptionsFuture<'a> {
        let options = vec![AuthSchemeOption::builder()
            .scheme_id(SCHEME_ID)
            .build()
            .expect("valid dpop auth option")];
        AuthSchemeOptionsFuture::ready(Ok(options))
    }
}

/// DPoP auth scheme.
#[derive(Debug)]
pub(super) struct DPoPAuthScheme {
    signer: DPoPSigner,
    private_key: Arc<SecretKey>,
}

impl DPoPAuthScheme {
    /// Creates a new `DPoPAuthScheme` that uses the given private key as the identity.
    pub(super) fn new(private_key_pem: &str) -> Result<Self, LoginTokenError> {
        let private_key = SecretKey::from_sec1_pem(private_key_pem)
            .map(Arc::new)
            .map_err(|e| LoginTokenError::other("invalid secret key", Some(e.into())))?;
        let signer = DPoPSigner;
        Ok(Self {
            signer,
            private_key,
        })
    }
}

impl AuthScheme for DPoPAuthScheme {
    fn scheme_id(&self) -> AuthSchemeId {
        SCHEME_ID
    }

    fn identity_resolver(
        &self,
        _identity_resolvers: &dyn GetIdentityResolver,
    ) -> Option<SharedIdentityResolver> {
        Some(SharedIdentityResolver::new(DPoPIdentityResolver(
            self.private_key.clone(),
        )))
    }

    fn signer(&self) -> &dyn Sign {
        &self.signer
    }
}

/// DPoP Signer calculates the DPoP header and "signs" the request with it
#[derive(Debug, Default)]
struct DPoPSigner;

/// DPoP identity resolver which yields the secret key to sign with
///
/// NOTE: We store the secret key on creation which avoids parsing the token file multiple times.
#[derive(Debug)]
struct DPoPIdentityResolver(Arc<SecretKey>);

// We override the cache partition because by default SharedIdentityResolver will claim a new
// partition everytime it's created. We aren't caching this identity anyway so avoid claiming cache
// partitions unnecessarily
static DPOP_PCACHE_PARTITION: LazyLock<IdentityCachePartition> =
    LazyLock::new(IdentityCachePartition::new);

impl ResolveIdentity for DPoPIdentityResolver {
    fn resolve_identity<'a>(
        &'a self,
        _runtime_components: &'a RuntimeComponents,
        _config_bag: &'a ConfigBag,
    ) -> IdentityFuture<'a> {
        let identity = Identity::new(DPoPPrivateKey(self.0.clone()), None);
        IdentityFuture::ready(Ok(identity))
    }

    fn cache_location(&self) -> IdentityCacheLocation {
        IdentityCacheLocation::IdentityResolver
    }

    fn cache_partition(&self) -> Option<IdentityCachePartition> {
        Some(*DPOP_PCACHE_PARTITION)
    }
}

impl Sign for DPoPSigner {
    fn sign_http_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        _auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
        runtime_components: &RuntimeComponents,
        _config_bag: &ConfigBag,
    ) -> Result<(), BoxError> {
        let private_key = identity
            .data::<DPoPPrivateKey>()
            .ok_or_else(|| LoginTokenError::WrongIdentityType(identity.clone()))?;

        let endpoint = request.uri().to_string();
        let now = runtime_components
            .time_source()
            .ok_or("A timesource must be provided")?
            .now();
        let dpop_header = calculate(&private_key.0, &endpoint, now)?;
        request.headers_mut().insert("dpop", dpop_header);
        Ok(())
    }
}

#[derive(Debug)]
struct DPoPPrivateKey(Arc<SecretKey>);

#[cfg(test)]
pub(super) mod test_util {
    use p256::ecdsa::signature::hazmat::PrehashVerifier;
    use p256::ecdsa::{Signature, VerifyingKey};
    use serde_json::Value;
    use sha2::{Digest, Sha256};

    pub(in crate::login) fn decode(part: &str) -> Vec<u8> {
        base64_simd::URL_SAFE_NO_PAD
            .decode_to_vec(part)
            .expect("DPoP proof parts are unpadded base64url")
    }

    /// Checks an ES256 signature over `signing_input` against the `jwk` in a proof header. The
    /// SHA-256 digest is computed here rather than by `p256`, so the check pins the hash ES256 uses.
    pub(in crate::login) fn verify_signature(
        header: &Value,
        signing_input: &[u8],
        signature: &[u8],
    ) -> Result<(), p256::ecdsa::Error> {
        let mut public_key = vec![0x04]; // SEC1 uncompressed point
        public_key.extend(decode(header["jwk"]["x"].as_str().expect("jwk.x")));
        public_key.extend(decode(header["jwk"]["y"].as_str().expect("jwk.y")));
        VerifyingKey::from_sec1_bytes(&public_key)?.verify_prehash(
            &Sha256::digest(signing_input),
            &Signature::from_slice(signature)?,
        )
    }

    /// Checks everything about a DPoP proof that a server checks without the request (RFC 9449
    /// section 4.3), and returns its decoded header and payload.
    pub(in crate::login) fn verify_proof(proof: &str) -> (Value, Value) {
        let (signing_input, signature) = proof.rsplit_once('.').expect("JWS has three parts");
        let (header, payload) = signing_input.split_once('.').expect("JWS has three parts");
        let header: Value = serde_json::from_slice(&decode(header)).expect("header is JSON");
        let payload: Value = serde_json::from_slice(&decode(payload)).expect("payload is JSON");
        assert_eq!("dpop+jwt", header["typ"]);
        assert_eq!("ES256", header["alg"]);
        assert!(
            header["jwk"].get("d").is_none(),
            "jwk leaks the private key"
        );
        assert_eq!("POST", payload["htm"]);
        assert!(payload["jti"].is_string(), "jti is required");
        assert!(payload["iat"].is_u64(), "iat is required");
        let signature = decode(signature);
        assert_eq!(
            64,
            signature.len(),
            "ES256 signatures are R || S (RFC 7518 section 3.4), not DER"
        );
        verify_signature(&header, signing_input.as_bytes(), &signature)
            .expect("proof verifies against the key in its header");
        (header, payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const TEST_KEY: &str = "-----BEGIN EC PRIVATE KEY-----\nMHcCAQEEIBMB/RwQERsVoqWRQG4zK8CnaAa5dfrpbm+9tFdBh3z4oAoGCCqGSM49\nAwEHoUQDQgAEWb1VLi1EA2hJaTz4yYuxSELvY+1GAfL+8rUTCAdiFid87Bf6GY+s\n2+1RpqDv0RpZiDIMCrZrsAh+RK9S3QCaGA==\n-----END EC PRIVATE KEY-----\n";

    #[test]
    fn test_header_extracts_public_coordinates() {
        let private_key = SecretKey::from_sec1_pem(TEST_KEY).unwrap();
        let h = header(&private_key).unwrap();
        assert_eq!(h.x_b64, "Wb1VLi1EA2hJaTz4yYuxSELvY-1GAfL-8rUTCAdiFic");
        assert_eq!(h.y_b64, "fOwX-hmPrNvtUaag79EaWYgyDAq2a7AIfkSvUt0Amhg");
    }

    #[test]
    fn test_build_message() {
        let h = Header {
            x_b64: "test_x".to_string(),
            y_b64: "test_y".to_string(),
        };
        let p = payload(
            "test-jti".to_string(),
            1651516560,
            "https://example.com/token",
        );
        let message = build_message(&h, &p);
        let parts: Vec<&str> = message.split('.').collect();
        assert_eq!(parts.len(), 2);

        let header_json = String::from_utf8(
            base64_simd::URL_SAFE_NO_PAD
                .decode_to_vec(parts[0])
                .unwrap(),
        )
        .unwrap();
        assert!(header_json.contains("dpop+jwt"));
        assert!(header_json.contains("test_x"));

        let payload_json = String::from_utf8(
            base64_simd::URL_SAFE_NO_PAD
                .decode_to_vec(parts[1])
                .unwrap(),
        )
        .unwrap();
        assert!(payload_json.contains("test-jti"));
        assert!(payload_json.contains("https://example.com/token"));
    }

    #[test]
    fn test_calculate_valid_key() {
        let endpoint = "https://signin.aws.amazon.com/v1/token";
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1651516560);

        let private_key = SecretKey::from_sec1_pem(TEST_KEY).unwrap();
        let result = calculate(&private_key, endpoint, now);
        assert!(result.is_ok());

        let dpop = result.unwrap();
        let parts: Vec<&str> = dpop.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_calculate_produces_verifiable_es256_proof() {
        let endpoint = "https://us-east-1.signin.aws.amazon.com/v1/token";
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1651516560);
        let private_key = SecretKey::from_sec1_pem(TEST_KEY).unwrap();

        let proof = calculate(&private_key, endpoint, now).unwrap();
        let (header, payload) = test_util::verify_proof(&proof);

        // Coordinates come from `openssl ec -pubout`, not from `p256`
        assert_eq!(
            serde_json::json!({
                "typ": "dpop+jwt",
                "alg": "ES256",
                "jwk": {
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "Wb1VLi1EA2hJaTz4yYuxSELvY-1GAfL-8rUTCAdiFic",
                    "y": "fOwX-hmPrNvtUaag79EaWYgyDAq2a7AIfkSvUt0Amhg",
                },
            }),
            header
        );
        let jti = payload["jti"].as_str().expect("jti is a string");
        assert_eq!(
            Some(uuid::Version::Random),
            uuid::Uuid::parse_str(jti).unwrap().get_version()
        );
        assert_eq!(
            serde_json::json!({ "jti": jti, "htm": "POST", "htu": endpoint, "iat": 1651516560 }),
            payload
        );

        // The verification above only means something if it rejects a forgery
        let (signing_input, signature) = proof.rsplit_once('.').unwrap();
        let mut signature = test_util::decode(signature);
        let (header_b64, _) = signing_input.split_once('.').unwrap();
        let altered_claims = Payload {
            jti: jti.to_string(),
            iat: 1651516561,
            htu: endpoint.to_string(),
        };
        let altered_input = format!(
            "{header_b64}.{}",
            base64_simd::URL_SAFE_NO_PAD.encode_to_string(altered_claims.to_json())
        );
        assert!(
            test_util::verify_signature(&header, altered_input.as_bytes(), &signature).is_err()
        );
        signature[63] ^= 1;
        assert!(
            test_util::verify_signature(&header, signing_input.as_bytes(), &signature).is_err()
        );
    }

    #[test]
    fn test_calculate_never_reuses_jti() {
        let endpoint = "https://us-east-1.signin.aws.amazon.com/v1/token";
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1651516560);
        let private_key = SecretKey::from_sec1_pem(TEST_KEY).unwrap();

        // Same key, endpoint and time, as a retried request with a frozen clock would have
        let first = test_util::verify_proof(&calculate(&private_key, endpoint, now).unwrap()).1;
        let second = test_util::verify_proof(&calculate(&private_key, endpoint, now).unwrap()).1;
        assert_ne!(first["jti"], second["jti"]);
    }

    /// A private key whose leading zero byte was dropped, as OpenSSL's `BN_bn2bin` does for about
    /// one key in 256. Generated with LibreSSL 3.3.6.
    const SHORT_SCALAR_KEY: &str = "-----BEGIN EC PRIVATE KEY-----\nMHYCAQEEH9rgRH8XRJdiB7jN5UayfeoW9aXKRNGW2HalEJ5SQSegCgYIKoZIzj0D\nAQehRANCAAS25vmt4QTwRksw1z+rgQMF6EXqhGsZefNfj1mfFrB2EMkR8ndGgPKq\n12D3WwW2yokPgIds1jOOu5FbZMNlMHOi\n-----END EC PRIVATE KEY-----\n";

    #[test]
    fn test_dpop_key_shapes_in_shared_cache_are_accepted() {
        // Coordinates come from `openssl ec -pubout`, not from `p256`
        let test_key = (
            "Wb1VLi1EA2hJaTz4yYuxSELvY-1GAfL-8rUTCAdiFic",
            "fOwX-hmPrNvtUaag79EaWYgyDAq2a7AIfkSvUt0Amhg",
        );
        let short_scalar_key = (
            "tub5reEE8EZLMNc_q4EDBehF6oRrGXnzX49ZnxawdhA",
            "yRHyd0aA8qrXYPdbBbbKiQ-Ah2zWM467kVtkw2Uwc6I",
        );
        let cases = [
            ("canonical", TEST_KEY.to_string(), test_key),
            (
                "31-byte private key",
                SHORT_SCALAR_KEY.to_string(),
                short_scalar_key,
            ),
            (
                "CRLF line endings",
                TEST_KEY.replace('\n', "\r\n"),
                test_key,
            ),
            (
                "no trailing newline",
                TEST_KEY.trim_end().to_string(),
                test_key,
            ),
        ];
        for (shape, pem, (x, y)) in cases {
            let scheme = DPoPAuthScheme::new(&pem).unwrap_or_else(|e| panic!("{shape}: {e}"));
            let header = header(&scheme.private_key).unwrap();
            assert_eq!(
                (x, y),
                (header.x_b64.as_str(), header.y_b64.as_str()),
                "{shape}"
            );
        }
    }

    #[test]
    fn test_calculate_invalid_key() {
        let result = DPoPAuthScheme::new("invalid_key");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid secret key"));
    }
}
