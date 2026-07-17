// use super::models::{IpBinding, CertInfo};

use mikrotik_rs::MikrotikDevice;
use std::sync::{Arc, Mutex};
use super::tls::TofuVerifier;

pub struct MikrotikClient {
    pub device: MikrotikDevice,
}

impl MikrotikClient {
    pub async fn connect(address: &str, username: &str, password: &str, _secure: bool, rejected_cert_hash: Arc<Mutex<Option<String>>>) -> Result<Self, String> {
        let addr = if address.contains(':') {
            address.to_string()
        } else {
            if _secure {
                format!("{}:8729", address)
            } else {
                format!("{}:8728", address)
            }
        };
        
        let pass = if password.is_empty() { None } else { Some(password) };
        
        let builder = MikrotikDevice::builder(&addr)
            .credentials(username, pass.as_deref());
            
        let device = if _secure {
            let verifier = Arc::new(TofuVerifier::new(rejected_cert_hash.clone()));
            let tls_config = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(verifier)
                .with_no_client_auth();
            
            let server_name = rustls::pki_types::ServerName::try_from("mikrotik").unwrap();
                
            builder.tls_config(Arc::new(tls_config), server_name)
                .connect()
                .await
                .map_err(|e| {
                    if let Some(hash) = rejected_cert_hash.lock().unwrap().clone() {
                        format!("UNTRUSTED_CERT:{}", hash)
                    } else {
                        format!("Connection failed: {}", e)
                    }
                })?
        } else {
            builder.connect()
                .await
                .map_err(|e| format!("Connection failed: {}", e))?
        };
            
        Ok(Self { device })
    }
}
