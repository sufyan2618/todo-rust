use anyhow::{Context, bail};
use serde::{Deserialize, de::DeserializeOwned};
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Clone, Deserialize)]
pub struct Secrets {
    #[serde(rename = "JWT_SECRET")]
    pub jwt_secret: String,
    #[serde(rename = "RDS_HOST")]
    rds_host: Option<String>,
    #[serde(rename = "DB_NAME")]
    database_name: Option<String>,
    #[serde(rename = "RDS_PORT")]
    rds_port: Option<u16>,
}

// Supports existing RDS-managed secrets; extra AWS rotation metadata is ignored.
#[derive(Deserialize)]
struct RdsSecret {
    username: String,
    password: String,
    host: Option<String>,
    port: Option<u16>,
    dbname: Option<String>,
    engine: Option<String>,
}

// Bootstrap identifiers and public server settings only. Never store credentials here.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bootstrap {
    aws_region: String,
    rds_secret_id: String,
    app_secret_id: String,
    port: u16,
    client_origin: String,
    rds_ca_bundle: String,
}

#[derive(Clone)]
pub struct Config {
    pub secrets: Secrets,
    pub database_options: PgConnectOptions,
    pub port: u16,
    pub client_origin: String,
}

impl Config {
    pub async fn load() -> anyhow::Result<Self> {
        let path = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "config.json".into());
        let settings: Bootstrap = serde_json::from_str(
            &std::fs::read_to_string(&path).context("Cannot read bootstrap config.json")?,
        )
        .context("Invalid bootstrap config.json")?;
        if settings.aws_region.trim().is_empty()
            || settings.rds_secret_id.trim().is_empty()
            || settings.app_secret_id.trim().is_empty()
        {
            bail!("Set aws_region, rds_secret_id, and app_secret_id in config.json");
        }
        let sdk = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(settings.aws_region.clone()))
            .load()
            .await;
        let client = aws_sdk_secretsmanager::Client::new(&sdk);
        let (rds, secrets) = tokio::try_join!(
            read_secret::<RdsSecret>(&client, &settings.rds_secret_id),
            read_secret::<Secrets>(&client, &settings.app_secret_id),
        )?;
        let database_options = database_options(&rds, &secrets, &settings.rds_ca_bundle)?;
        Ok(Self {
            secrets,
            database_options,
            port: settings.port,
            client_origin: settings.client_origin,
        })
    }
}

async fn read_secret<T: DeserializeOwned>(
    client: &aws_sdk_secretsmanager::Client,
    id: &str,
) -> anyhow::Result<T> {
    let value = client
        .get_secret_value()
        .secret_id(id)
        .send()
        .await
        .context(
            "Failed to retrieve an AWS secret; check region, secret IDs, and IAM permissions",
        )?;
    // Serde errors describe fields/types, without printing the secret payload.
    serde_json::from_str(
        value
            .secret_string()
            .context("AWS secret must be a JSON SecretString")?,
    )
    .context("AWS secret has missing or invalid fields")
}

fn database_options(
    rds: &RdsSecret,
    app: &Secrets,
    ca_bundle: &str,
) -> anyhow::Result<PgConnectOptions> {
    if app.jwt_secret.len() < 32 {
        bail!("JWT_SECRET must be at least 32 bytes");
    }
    if rds.username.is_empty() || rds.password.is_empty() {
        bail!("RDS secret requires non-empty username and password");
    }
    if rds
        .engine
        .as_deref()
        .is_some_and(|engine| engine != "postgres" && engine != "postgresql")
    {
        bail!("RDS secret must describe PostgreSQL");
    }
    let host = rds
        .host
        .as_deref()
        .or(app.rds_host.as_deref())
        .filter(|value| !value.trim().is_empty())
        .context("Set host in the RDS secret or RDS_HOST in the app secret")?;
    let database = rds
        .dbname
        .as_deref()
        .or(app.database_name.as_deref())
        .filter(|value| !value.trim().is_empty())
        .context("Set dbname in the RDS secret or DB_NAME in the app secret")?;
    let port = rds.port.or(app.rds_port).unwrap_or(5432);
    if port == 0 {
        bail!("RDS port must be greater than zero");
    }
    // Structured options preserve special characters in passwords without URL encoding.
    Ok(PgConnectOptions::new()
        .host(host)
        .port(port)
        .username(&rds.username)
        .password(&rds.password)
        .database(database)
        .ssl_mode(PgSslMode::VerifyFull)
        .ssl_root_cert(ca_bundle))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> Secrets {
        serde_json::from_value(serde_json::json!({
            "JWT_SECRET": "a-random-secret-with-at-least-32-bytes",
            "RDS_HOST": "example.rds.amazonaws.com", "DB_NAME": "todo"
        }))
        .unwrap()
    }
    #[test]
    fn managed_rds_secret_uses_app_connection_fields() {
        let rds = serde_json::from_str(
            r#"{"username":"todo","password":"p@ss:/?#word","dbInstanceIdentifier":"existing"}"#,
        )
        .unwrap();
        let options = database_options(&rds, &app(), "certs/global-bundle.pem").unwrap();
        assert_eq!(options.get_host(), "example.rds.amazonaws.com");
        assert_eq!(options.get_database(), Some("todo"));
        assert!(matches!(options.get_ssl_mode(), PgSslMode::VerifyFull));
    }
    #[test]
    fn full_rds_secret_takes_precedence() {
        let rds = serde_json::from_str(r#"{"username":"todo","password":"secret","host":"actual.rds.amazonaws.com","port":5433,"dbname":"existing","engine":"postgres"}"#).unwrap();
        let options = database_options(&rds, &app(), "certs/global-bundle.pem").unwrap();
        assert_eq!(options.get_host(), "actual.rds.amazonaws.com");
        assert_eq!(options.get_port(), 5433);
        assert_eq!(options.get_database(), Some("existing"));
    }
    #[test]
    fn missing_database_and_weak_jwt_are_rejected() {
        let rds = serde_json::from_str(r#"{"username":"todo","password":"secret"}"#).unwrap();
        let mut app = app();
        app.database_name = None;
        assert!(database_options(&rds, &app, "ca.pem").is_err());
        app.database_name = Some("todo".into());
        app.jwt_secret = "weak".into();
        assert!(database_options(&rds, &app, "ca.pem").is_err());
    }
}
