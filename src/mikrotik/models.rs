#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct IpBinding {
    pub id: String,
    pub address: String,
    pub mac_address: String,
    pub binding_type: String,
    pub comment: String,
    pub server: String,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CertInfo {
    pub subject: String,
    pub issuer: String,
    pub valid_from: String,
    pub valid_to: String,
    pub fingerprint: String,
}
