use std::time::Duration;
use ureq::tls::TlsConfig;

use crate::bot::GtpsConfig;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct LoginInfo {
    pub protocol: u32,
    pub game_version: String,
}

impl LoginInfo {
    pub fn to_form_data(&self) -> String {
        format!("protocol={}&version={}", self.protocol, self.game_version)
    }
}

#[derive(Debug, Default)]
pub struct ServerData {
    pub server: String,
    pub port: u16,
    pub loginurl: String,
    pub server_type: u8,
    pub beta_server: String,
    pub beta_loginurl: String,
    pub beta_port: u16,
    pub beta_type: u8,
    pub beta2_server: String,
    pub beta2_loginurl: String,
    pub beta2_port: u16,
    pub beta2_type: u8,
    pub beta3_server: String,
    pub beta3_loginurl: String,
    pub beta3_port: u16,
    pub beta3_type: u8,
    pub type2: u8,
    pub maint: Option<String>,
    pub meta: String,
}

impl ServerData {
    pub fn parse_from_response(response: &str) -> Result<Self> {
        let mut data = ServerData::default();

        for line in response.lines() {
            if line.starts_with("RTENDMARKERBS1001") {
                break;
            }
            let Some((key, value)) = line.split_once('|') else {
                continue;
            };
            let value = value.trim();
            match key.trim() {
                "server" => data.server = value.into(),
                "port" => data.port = value.parse()?,
                "loginurl" => data.loginurl = value.into(),
                "type" => data.server_type = value.parse()?,
                "beta_server" => data.beta_server = value.into(),
                "beta_loginurl" => data.beta_loginurl = value.into(),
                "beta_port" => data.beta_port = value.parse()?,
                "beta_type" => data.beta_type = value.parse()?,
                "beta2_server" => data.beta2_server = value.into(),
                "beta2_loginurl" => data.beta2_loginurl = value.into(),
                "beta2_port" => data.beta2_port = value.parse()?,
                "beta2_type" => data.beta2_type = value.parse()?,
                "beta3_server" => data.beta3_server = value.into(),
                "beta3_loginurl" => data.beta3_loginurl = value.into(),
                "beta3_port" => data.beta3_port = value.parse()?,
                "beta3_type" => data.beta3_type = value.parse()?,
                "type2" => data.type2 = value.parse()?,
                "#maint" => data.maint = Some(value.into()),
                "meta" => data.meta = value.into(),
                _ => {}
            }
        }

        Ok(data)
    }
}

pub fn get_server_data(
    alternate: bool,
    login_info: &LoginInfo,
    gtps: Option<&GtpsConfig>,
) -> Result<ServerData> {
    get_server_data_proxied(alternate, login_info, None, gtps)
}

pub fn get_server_data_proxied(
    alternate: bool,
    login_info: &LoginInfo,
    proxy_url: Option<&str>,
    gtps: Option<&GtpsConfig>,
) -> Result<ServerData> {
    // Check for custom GTPS server via shared config or environment variables
    if let Some(cfg) = gtps {
        if !cfg.host.is_empty() {
            println!("[server_data] Using custom GTPS server: {}:{}", cfg.host, cfg.port);
            return Ok(ServerData {
                server: cfg.host.clone(),
                port: cfg.port,
                loginurl: String::new(),
                server_type: cfg.server_type,
                beta_server: String::new(),
                beta_loginurl: String::new(),
                beta_port: 0,
                beta_type: 0,
                beta2_server: String::new(),
                beta2_loginurl: String::new(),
                beta2_port: 0,
                beta2_type: 0,
                beta3_server: String::new(),
                beta3_loginurl: String::new(),
                beta3_port: 0,
                beta3_type: 0,
                type2: cfg.type2,
                maint: None,
                meta: cfg.meta.clone(),
            });
        }
    }

    // Fallback: check environment variables
    if let Ok(host) = std::env::var("GTPS_HOST") {
        let port: u16 = std::env::var("GTPS_PORT")
            .unwrap_or_else(|_| "17091".to_string())
            .parse()
            .unwrap_or(17091);
        let server_type: u8 = std::env::var("GTPS_TYPE")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        let type2: u8 = std::env::var("GTPS_TYPE2")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap_or(1);
        let meta = std::env::var("GTPS_META")
            .unwrap_or_else(|_| "localhost".to_string());
        
        println!("[server_data] Using custom GTPS server (env): {}:{}", host, port);
        return Ok(ServerData {
            server: host,
            port,
            loginurl: String::new(),
            server_type,
            beta_server: String::new(),
            beta_loginurl: String::new(),
            beta_port: 0,
            beta_type: 0,
            beta2_server: String::new(),
            beta2_loginurl: String::new(),
            beta2_port: 0,
            beta2_type: 0,
            beta3_server: String::new(),
            beta3_loginurl: String::new(),
            beta3_port: 0,
            beta3_type: 0,
            type2,
            maint: None,
            meta,
        });
    }

    let url = if alternate {
        "https://www.growtopia2.com/growtopia/server_data.php"
    } else {
        "https://www.growtopia1.com/growtopia/server_data.php"
    };

    println!("[server_data] proxy_url={:?}", proxy_url);
    let agent = if let Some(p) = proxy_url {
        let proxy = ureq::Proxy::new(p)?;
        ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .proxy(Some(proxy))
                .tls_config(TlsConfig::builder().disable_verification(true).build())
                .timeout_global(Some(Duration::from_secs(20)))
                .build(),
        )
    } else {
        ureq::Agent::new_with_config(
            ureq::config::Config::builder()
                .tls_config(TlsConfig::builder().disable_verification(true).build())
                .timeout_global(Some(Duration::from_secs(20)))
                .build(),
        )
    };
    let body = agent
        .post(url)
        .header(
            "User-Agent",
            "UbiServices_SDK_2022.Release.9_PC64_ansi_static",
        )
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send(format!(
            "platform=0&protocol={}&version={}",
            login_info.protocol, login_info.game_version
        ))?
        .body_mut()
        .read_to_string()?;

    ServerData::parse_from_response(&body)
}
