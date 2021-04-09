use std::convert::TryInto;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use config::RawConfig;
use structopt::StructOpt;
use tokio::fs::read_to_string;

use crate::tproxy::config::Config;

mod config;

#[derive(Debug, StructOpt)]
#[structopt(name = "proxy", about = "The option of rs-proxy.")]
struct Opt {
    ///path of config file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

pub async fn get_config() -> Result<Config> {
    let opt: Opt = Opt::from_args();
    get_config_from_opt(opt.input).await
}

pub async fn get_config_from_opt(path_buf: PathBuf) -> Result<Config> {
    let buffer = read_to_string(&path_buf).await?;
    match path_buf.extension().and_then(|ext| ext.to_str()) {
        Some("json") => Ok(serde_json::from_str::<RawConfig>(&buffer)?.try_into()?),
        Some("yaml") => Ok(serde_yaml::from_str::<RawConfig>(&buffer)?.try_into()?),
        _ => Err(anyhow!("invalid file extension")),
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use anyhow::Result;

    use super::config::{RawActions, RawConfig, RawRule, RawSelector, RawTarget};
    #[test]
    fn test_serde_util() -> Result<()> {
        let conf = RawConfig {
            listen_port: Some(58080),
            proxy_ports: vec![80],
            proxy_mark: Some(255),
            ignore_mark: Some(255),
            route_table: Some(100),
            rules: Some(vec![
                RawRule {
                    target: RawTarget::Request,
                    selector: RawSelector {
                        port: None,
                        path: Some("/rs-tproxy".to_string()),
                        method: Some("GET".to_string()),
                        headers: Some(
                            [("aname", "avalue")]
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect(),
                        ),
                        code: None,
                        response_headers: None,
                    },
                    actions: RawActions {
                        abort: None,
                        delay: Some(Duration::from_secs(1)),
                        append: None,
                        replace: None,
                    },
                },
                RawRule {
                    target: RawTarget::Response,
                    selector: RawSelector {
                        port: None,
                        path: Some("/rs-tproxy".to_string()),
                        method: Some("GET".to_string()),
                        headers: Some(
                            [("aname", "avalue")]
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect(),
                        ),
                        code: Some(80),
                        response_headers: Some(
                            [("server", "nginx")]
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                                .collect(),
                        ),
                    },
                    actions: RawActions {
                        abort: Some(true),
                        delay: Some(Duration::from_secs(1)),
                        append: None,
                        replace: None,
                    },
                },
            ]),
        };
        let json = serde_json::to_string(&conf)?;
        println!("{}", json);
        let conf_json_out: RawConfig = serde_json::from_str(&json)?;
        assert_eq!(conf_json_out, conf);
        let yaml = serde_yaml::to_string(&conf)?;
        println!("{}", yaml);
        let conf_yaml_out: RawConfig = serde_yaml::from_str(&yaml)?;
        assert_eq!(conf_yaml_out, conf);
        Ok(())
    }
}
