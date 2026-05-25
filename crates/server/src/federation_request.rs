use axum::http;
use bytes::Bytes;
use cynic::Operation;
use ed25519_compact::KeyPair;
use http_body_util::Full;
use httpsig_hyper::{ContentDigestType, HyperDigestError, RequestContentDigest};
use serde::Serialize;
use url::Url;

pub trait HttpExt {
    /// Runs a GraphQL query with the parameters in RequestBuilder, deserializes
    /// the and returns the result.
    fn run_graphql<ResponseData, Vars>(
        self,
        operation: impl AsRef<Operation<ResponseData, Vars>> + Serialize,
        // key: KeyPair,
    ) -> Result<http::request::Request<Full<Bytes>>, http::Error>
    where
        Vars: serde::Serialize,
        ResponseData: serde::de::DeserializeOwned + 'static;
}

impl HttpExt for http::request::Builder {
    fn run_graphql<ResponseData, Vars>(
        self,
        operation: impl AsRef<Operation<ResponseData, Vars>> + Serialize,
        // key: KeyPair,
    ) -> Result<http::request::Request<Full<Bytes>>, http::Error>
    where
        Vars: serde::Serialize,
        ResponseData: serde::de::DeserializeOwned + 'static,
    {
        let body = bytes::Bytes::from(serde_json::to_vec(&operation).unwrap());
        // .map_err(|never| match never {});
        self.header("content-type", "application/json")
            .body(Full::new(body))
    }
}

pub const COVERED_COMPONENTS: &[&str] = &[
    "@method",
    "@target-uri",
    "origin",
    "content-type",
    "content-digest",
];

pub async fn build_request(
    kp: KeyPair,
    graph_endpoint: Url,
    local_href: &Url,
) -> http::Request<http_body_util::combinators::BoxBody<bytes::Bytes, HyperDigestError>> {
    use cynic::QueryBuilder;
    use httpsig_hyper::MessageSignatureReq;
    use httpsig_hyper::prelude::*;
    let operation = crate::remote_server_sync::gql::ServerInfoQuery::build(());
    let req = http::Request::builder()
        .method("POST")
        .uri(graph_endpoint.as_str())
        .header("Origin", local_href.as_str())
        .run_graphql(operation)
        .unwrap();
    let mut req = req
        .set_content_digest(&ContentDigestType::Sha512)
        .await
        .unwrap();

    let covered_components = COVERED_COMPONENTS
        .iter()
        .map(|v| message_component::HttpMessageComponentId::try_from(*v))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut signature_params = HttpSignatureParams::try_new(&covered_components).unwrap();

    // set signing/verifying key information, alg and keyid with ed25519
    // let key_id = playerbrainz_signatures::key::PublicKey::Ed25519(kp.pk).key_id();
    let secret_key = SecretKey::Ed25519(kp.sk.clone());
    signature_params.set_key_info(&secret_key);
    use playerbrainz_signatures::KeyAlgorithm;

    req.set_message_signature(
        &signature_params,
        &secret_key,
        Some(KeyAlgorithm::Ed25519.as_ref()),
    )
    .await
    .unwrap();

    req
}
