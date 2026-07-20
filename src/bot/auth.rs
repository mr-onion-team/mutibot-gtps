use crate::constants::{GAME_VER, PROTOCOL};
use crate::dashboard::get_dashboard_proxied;
use crate::login::{LoginError, get_legacy_token_proxied};
use crate::server_data::{LoginInfo, get_server_data_proxied};
use std::net::{SocketAddr, ToSocketAddrs};

use super::shared::{GtpsConfig, Socks5Config};

pub(super) struct Credentials {
    pub ltoken: String,
    pub meta: String,
    pub addr: SocketAddr,
}

pub(super) fn fetch_credentials(
    username: &str,
    password: &str,
    proxy: Option<&Socks5Config>,
    gtps: Option<&GtpsConfig>,
    log: &mut dyn FnMut(String),
) -> Credentials {
    let proxy_url = proxy.map(|p| p.to_url());
    let proxy_url = proxy_url.as_deref();

    let login_info = LoginInfo {
        protocol: PROTOCOL,
        game_version: GAME_VER.into(),
    };

    // GTPS mode: skip official login, use direct connection
    if let Some(cfg) = gtps {
        if !cfg.host.is_empty() {
            log("[Bot] GTPS mode detected - using direct connection".to_string());
            let server_data = get_server_data_proxied(false, &login_info, proxy_url, gtps)
                .expect("Failed to get GTPS server data");
            
            let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
                .to_socket_addrs()
                .expect("Failed to resolve GTPS server address")
                .filter(|a| a.is_ipv4())
                .next()
                .or_else(|| format!("{}:{}", server_data.server, server_data.port)
                    .to_socket_addrs().ok()?.next())
                .expect("No addresses found for GTPS server");

            // In GTPS mode, ltoken holds the password (username is stored separately)
            let ltoken = password.to_string();
            
            log(format!("[Bot] GTPS server: {}:{}", server_data.server, server_data.port));
            return Credentials {
                ltoken,
                meta: server_data.meta,
                addr,
            };
        }
    }

    // Fallback: check environment variables
    if std::env::var("GTPS_HOST").is_ok() {
        log("[Bot] GTPS mode detected (env) - using direct connection".to_string());
        let server_data = get_server_data_proxied(false, &login_info, proxy_url, None)
            .expect("Failed to get GTPS server data");
        
        let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
            .to_socket_addrs()
            .expect("Failed to resolve GTPS server address")
            .filter(|a| a.is_ipv4())
            .next()
            .or_else(|| format!("{}:{}", server_data.server, server_data.port)
                .to_socket_addrs().ok()?.next())
            .expect("No addresses found for GTPS server");

        let ltoken = password.to_string();
        
        log(format!("[Bot] GTPS server: {}:{}", server_data.server, server_data.port));
        return Credentials {
            ltoken,
            meta: server_data.meta,
            addr,
        };
    }

    let mut alternate = false;
    loop {
        log(format!(
            "[Bot] fetching server_data (alternate={alternate})..."
        ));
        let server_data = match get_server_data_proxied(alternate, &login_info, proxy_url, None) {
            Ok(s) => s,
            Err(e) => {
                alternate = !alternate;
                log(format!(
                    "[Bot] fetch: server_data failed: {e} - retrying in 5s"
                ));
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let dashboard = match get_dashboard_proxied(
            &server_data.loginurl,
            &login_info,
            &server_data.meta,
            proxy_url,
        ) {
            Ok(d) => d,
            Err(e) => {
                log(format!(
                    "[Bot] fetch: dashboard failed: {e} - retrying in 5s"
                ));
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let growtopia_url = match dashboard.growtopia {
            Some(u) => u,
            None => {
                log(format!(
                    "[Bot] fetch: no Growtopia URL in dashboard - retrying in 5s"
                ));
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let ltoken = match get_legacy_token_proxied(&growtopia_url, username, password, proxy_url)
        {
            Ok(t) => t,
            Err(e) => {
                log(format!("[Bot] fetch: login failed: {e}"));
                if matches!(e, LoginError::Exhausted) {
                    log("[Bot] login attempts exhausted - stopping".to_string());
                    panic!("[Bot] login attempts exhausted - stopping");
                }
                if matches!(e, LoginError::WrongCredentials) {
                    log("[Bot] wrong credentials - stopping".to_string());
                    panic!("[Bot] wrong credentials - stopping");
                }
                log("[Bot] retrying in 5s".to_string());
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        let addr: SocketAddr = format!("{}:{}", server_data.server, server_data.port)
            .parse()
            .expect("Invalid server address");

        log(format!("[Bot] Got token: {ltoken}"));
        return Credentials {
            ltoken,
            meta: server_data.meta,
            addr,
        };
    }
}
