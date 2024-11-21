pub mod ui;

use anyhow::{bail, Result};
use jsonwebtoken::DecodingKey;
use serde_json::Map;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Algorithm {
    /// HMAC using SHA-256
    #[default]
    HS256,
    /// HMAC using SHA-384
    HS384,
    /// HMAC using SHA-512
    HS512,

    /// RSASSA-PKCS1-v1_5 using SHA-256
    RS256,
    /// RSASSA-PKCS1-v1_5 using SHA-384
    RS384,
    /// RSASSA-PKCS1-v1_5 using SHA-512
    RS512,
}

impl From<Algorithm> for jsonwebtoken::Algorithm {
    fn from(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::HS256 => jsonwebtoken::Algorithm::HS256,
            Algorithm::HS384 => jsonwebtoken::Algorithm::HS384,
            Algorithm::HS512 => jsonwebtoken::Algorithm::HS512,
            Algorithm::RS256 => jsonwebtoken::Algorithm::RS256,
            Algorithm::RS384 => jsonwebtoken::Algorithm::RS384,
            Algorithm::RS512 => jsonwebtoken::Algorithm::RS512,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct JwtEncoderDecoder {
    pub encoded: String,
    pub decoded: String,
    pub algorithm: Algorithm,
    pub secret: String,
    pub public_key: String,
    pub private_key: String,
    pub verified: Option<bool>,
}

impl JwtEncoderDecoder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.encoded.clear();
        self.decoded.clear();
        self.algorithm = Algorithm::HS256;
        self.secret.clear();
        self.public_key.clear();
        self.private_key.clear();
        self.verified = None;
    }

    pub fn get_header(&mut self) -> Result<String> {
        let header = jsonwebtoken::decode_header(&self.encoded)?;
        Ok(serde_json::to_string(&header)?)
    }

    pub fn encode(&mut self) -> Result<()> {
        let token = match self.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => self.encode_by_hmac()?,
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => self.encode_by_rsa()?,
        };

        self.encoded = token;
        Ok(())
    }

    pub fn decode(&mut self) -> Result<()> {
        let mut validation = jsonwebtoken::Validation::new(self.algorithm.clone().into());
        validation.insecure_disable_signature_validation();
        validation.required_spec_claims.remove("exp");

        let token_data = jsonwebtoken::decode::<Map<_, _>>(
            &self.encoded,
            &DecodingKey::from_secret(&[]),
            &validation,
        )?;

        self.decoded = serde_json::to_string_pretty(&token_data.claims)?;
        Ok(())
    }

    pub fn verify(&mut self) -> Result<()> {
        match self.algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => self.verify_by_hmac(),
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512 => self.verify_by_rsa(),
        }
    }

    fn encode_by_hmac(&mut self) -> Result<String> {
        if self.secret.is_empty() {
            bail!("Secret is required");
        }

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(self.algorithm.clone().into()),
            &serde_json::from_str::<serde_json::Value>(&self.decoded)?,
            &jsonwebtoken::EncodingKey::from_secret(self.secret.trim().as_bytes()),
        )?;

        Ok(token)
    }

    fn encode_by_rsa(&mut self) -> Result<String> {
        if self.private_key.is_empty() {
            bail!("Private key is required");
        }

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(self.algorithm.clone().into()),
            &serde_json::from_str::<serde_json::Value>(&self.decoded)?,
            &jsonwebtoken::EncodingKey::from_rsa_pem(self.private_key.trim().as_bytes())?,
        )?;

        Ok(token)
    }

    fn verify_by_hmac(&mut self) -> Result<()> {
        if self.secret.is_empty() {
            bail!("Secret is required");
        }

        let mut validation = jsonwebtoken::Validation::new(self.algorithm.clone().into());
        validation.required_spec_claims.remove("exp");

        match jsonwebtoken::decode::<Map<_, _>>(
            &self.encoded,
            &jsonwebtoken::DecodingKey::from_secret(self.secret.trim().as_bytes()),
            &validation,
        ) {
            Ok(_) => {
                self.verified = Some(true);
                Ok(())
            }
            Err(err) => {
                self.verified = Some(false);
                anyhow::bail!(err)
            }
        }
    }

    fn verify_by_rsa(&mut self) -> Result<()> {
        if self.public_key.is_empty() {
            bail!("Public key is required");
        }

        let mut validation = jsonwebtoken::Validation::new(self.algorithm.clone().into());
        validation.required_spec_claims.remove("exp");

        let decoding_key =
            match jsonwebtoken::DecodingKey::from_rsa_pem(self.public_key.trim().as_bytes()) {
                Ok(key) => key,
                Err(err) => {
                    self.verified = Some(false);
                    bail!(err)
                }
            };

        match jsonwebtoken::decode::<Map<_, _>>(&self.encoded, &decoding_key, &validation) {
            Ok(_) => {
                self.verified = Some(true);
                Ok(())
            }
            Err(err) => {
                self.verified = Some(false);
                anyhow::bail!(err)
            }
        }
    }
}
