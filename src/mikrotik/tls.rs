use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

fn get_known_certs_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".bandeng");
    fs::create_dir_all(&path).unwrap_or_default();
    path.push("known_certs.json");
    path
}

fn load_known_certs() -> HashSet<String> {
    let path = get_known_certs_path();
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(certs) = serde_json::from_str::<HashSet<String>>(&data) {
            return certs;
        }
    }
    HashSet::new()
}

pub fn save_known_cert(hash: String) {
    let mut certs = load_known_certs();
    certs.insert(hash);
    let path = get_known_certs_path();
    if let Ok(data) = serde_json::to_string_pretty(&certs) {
        let _ = fs::write(path, data);
    }
}

pub fn calculate_hash(cert: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cert);
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in result {
        use std::fmt::Write;
        write!(&mut hex, "{:02x}", byte).unwrap();
    }
    hex
}

#[derive(Debug)]
pub struct TofuVerifier {
    pub known_hashes: HashSet<String>,
    pub rejected_cert_hash: Arc<Mutex<Option<String>>>,
}

impl TofuVerifier {
    pub fn new(rejected_cert_hash: Arc<Mutex<Option<String>>>) -> Self {
        Self {
            known_hashes: load_known_certs(),
            rejected_cert_hash,
        }
    }
}

impl ServerCertVerifier for TofuVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        let hash = calculate_hash(end_entity.as_ref());
        if self.known_hashes.contains(&hash) {
            Ok(ServerCertVerified::assertion())
        } else {
            *self.rejected_cert_hash.lock().unwrap() = Some(hash);
            Err(Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}
