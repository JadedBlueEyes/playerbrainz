use std::str::FromStr;

use async_graphql_axum::{GraphQLRequest, rejection::GraphQLRejection};
use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};

use http_body_util::{BodyExt, Full};
use httpsig::prelude::PublicKey as HttpSigPublicKey;
use playerbrainz_entities::{RemoteServerKey, remote_server_keys};
use playerbrainz_signatures::KeyAlgorithm;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

#[derive(Clone, Debug, Default)]
pub struct RequestSignature {
    pub status: SignatureStatus,
}

#[derive(Clone, Debug, Default)]
pub enum SignatureStatus {
    /// The request did not include signature headers.
    #[default]
    Unsigned,

    /// The request included signature headers and the signature verified.
    Verified(VerifiedSignature),
}

#[derive(Clone, Debug)]
pub struct VerifiedSignature {
    pub key_id: String,
    pub server_href: String,
}

/// A GraphQL request extractor that:
///
/// - verifies HTTP Message Signatures **when present**
/// - exposes the resulting [`RequestSignature`]
/// - still parses the GraphQL body into an [`async_graphql::Request`]
///
/// If signature headers are present but verification fails, this rejects with `401 Unauthorized`.
#[derive(Debug)]
pub struct SignedGraphQLRequest {
    pub signature: RequestSignature,
    pub request: async_graphql::Request,
}

impl SignedGraphQLRequest {
    fn unauthorized(message: impl Into<String>) -> Response {
        let message = message.into();
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(message))
            .expect("unauthorized response")
    }

    fn internal_error(message: impl Into<String>) -> Response {
        let message = message.into();
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(message))
            .expect("internal error response")
    }

    async fn lookup_remote_server_keys(
        db: &sea_orm::DatabaseConnection,
        href: &str,
    ) -> Result<Vec<(HttpSigPublicKey, Option<&'static str>)>, Response> {
        let now = chrono::Utc::now().fixed_offset();

        let keys = RemoteServerKey::find()
            .filter(remote_server_keys::Column::ServerHref.eq(href))
            .filter(remote_server_keys::Column::Algorithm.eq(KeyAlgorithm::Ed25519.as_ref()))
            .filter(remote_server_keys::Column::ValidUntil.gt(now))
            .all(db)
            .await
            .map_err(|e| Self::internal_error(format!("DB error looking up signature key: {e}")))?;

        let keys: Vec<(HttpSigPublicKey, _)> = keys
            .into_iter()
            .filter_map(|k| Self::verifying_key_from_row(&k).ok())
            .collect();

        match keys.len() {
            0 => Err(Self::unauthorized(format!("No keys found for '{href}'"))),
            _ => Ok(keys),
        }
    }

    #[allow(clippy::result_large_err)]
    fn verifying_key_from_row(
        row: &remote_server_keys::Model,
    ) -> Result<(HttpSigPublicKey, Option<&'static str>), Response> {
        let algo = KeyAlgorithm::from_str(&row.algorithm).map_err(|_| {
            Self::unauthorized(format!("Invalid key algorithm {:?}", &row.algorithm))
        })?;

        match algo {
            KeyAlgorithm::Ed25519 => {
                let pk =
                    ed25519_compact::PublicKey::from_slice(&row.public_key_bytes).map_err(|e| {
                        Self::unauthorized(format!("Invalid Ed25519 public key bytes: {e}"))
                    })?;
                Ok((
                    HttpSigPublicKey::Ed25519(pk),
                    Some(KeyAlgorithm::Ed25519.as_ref()),
                ))
            }
        }
    }
}

impl FromRequest<sea_orm::DatabaseConnection> for SignedGraphQLRequest {
    type Rejection = Response;

    async fn from_request(
        req: Request<Body>,
        db: &sea_orm::DatabaseConnection,
    ) -> Result<Self, Self::Rejection> {
        let has_signatures = req.headers().contains_key("signature")
            && req.headers().contains_key("signature-input");

        if !has_signatures {
            let gql_req = GraphQLRequest::<GraphQLRejection>::from_request(req, db)
                .await
                .map_err(IntoResponse::into_response)?
                .into_inner();

            return Ok(Self {
                signature: RequestSignature {
                    status: SignatureStatus::Unsigned,
                },
                request: gql_req,
            });
        }

        let origin = req
            .headers()
            .get("origin")
            .ok_or_else(|| Self::unauthorized("Signed request missing origin"))?
            .to_str()
            .map_err(|_| Self::unauthorized("Signed request has invalid origin"))?
            .to_string();
        let keys = Self::lookup_remote_server_keys(db, &origin).await?;

        // yay fun type conversions
        let key_refs: Vec<(&HttpSigPublicKey, Option<&str>)> =
            keys.iter().map(|(k, alg)| (k, alg.as_deref())).collect();

        let (parts, body) = req.into_parts();
        let bytes = body
            .collect()
            .await
            .map_err(|e| Self::internal_error(format!("Failed to read request body: {e}")))?
            .to_bytes();

        let full_body = Full::new(bytes.clone());
        let hyper_req = Request::from_parts(parts, full_body);

        use httpsig_hyper::MessageSignatureReq;

        // dbg!(hyper_req.extract_signatures());

        let verify_result = hyper_req.verify_message_signatures(&key_refs).await;

        let (parts, _body) = hyper_req.into_parts();
        let req = Request::from_parts(parts, Body::from(bytes));

        // Extract key id using the httpsig parser (do not manually parse Signature-Input).
        let signature_header = req
            .headers()
            .get("signature")
            .ok_or_else(|| Self::unauthorized("Signed request missing signature"))?
            .to_str()
            .map_err(|_| Self::unauthorized("Signed request has invalid signature"))?;

        let signature_input_header = req
            .headers()
            .get("signature-input")
            .ok_or_else(|| Self::unauthorized("Signed request missing signature-input"))?
            .to_str()
            .map_err(|_| Self::unauthorized("Signed request has invalid signature-input"))?;

        let parsed = httpsig::prelude::HttpSignatureHeaders::try_parse(
            signature_header,
            signature_input_header,
        )
        .map_err(|e| Self::unauthorized(format!("Invalid signature headers: {e}")))?;

        let key_id = parsed
            .iter()
            .next()
            .and_then(|(_name, headers)| headers.signature_params().keyid.clone())
            .ok_or_else(|| Self::unauthorized("Signed request missing keyid"))?;

        match verify_result {
            Ok(_res) => {
                let gql_req = GraphQLRequest::<GraphQLRejection>::from_request(req, db)
                    .await
                    .map_err(IntoResponse::into_response)?
                    .into_inner();

                Ok(Self {
                    signature: RequestSignature {
                        status: SignatureStatus::Verified(VerifiedSignature {
                            key_id,
                            server_href: origin.clone(),
                        }),
                    },
                    request: gql_req,
                })
            }
            Err(e) => Err(Self::unauthorized(format!(
                "Signature verification failed: {e}"
            ))),
        }
    }
}

/// Convenience helper for "must be signed" GraphQL endpoints.
///
/// You can use this from resolvers or handlers by checking that
/// [`RequestSignature::status`] is [`SignatureStatus::Verified`].
impl RequestSignature {
    pub fn verified(&self) -> Option<&VerifiedSignature> {
        match &self.status {
            SignatureStatus::Verified(v) => Some(v),
            SignatureStatus::Unsigned => None,
        }
    }
}

impl VerifiedSignature {
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn server_href(&self) -> &str {
        &self.server_href
    }
}

impl SignatureStatus {
    pub fn is_verified(&self) -> bool {
        matches!(self, SignatureStatus::Verified(_))
    }
}

impl RequestSignature {
    pub fn is_verified(&self) -> bool {
        self.status.is_verified()
    }
}
