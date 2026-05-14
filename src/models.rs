use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Server {
    pub id: i64,
    pub name: String,
    pub ldap_url: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub skip_tls: bool,
}
