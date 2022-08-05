use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use url::Url;

use crate::process::server::ServerProcess;
use crate::webdriver::mgmt::WebDriverLocation;

pub enum WebDriverProcess {
    Local(ServerProcess),
    Remote(Url),
}

impl WebDriverProcess {
    pub fn new(locate: &WebDriverLocation) -> Result<WebDriverProcess> {
        match locate {
            WebDriverLocation::Remote(url) => Self::create_remote(url.clone()),
            WebDriverLocation::Local((path, args)) => Self::start_local(path, args.clone())
        }
    }

    pub fn get_url(&self) -> &Url {
        match self {
            Self::Remote(url) => url,
            Self::Local(process) => process.get_url(),
        }
    }

    fn create_remote(url: Url) -> Result<WebDriverProcess> {
        Ok(WebDriverProcess::Remote(url))
    }

    fn start_local(path: &PathBuf, args: Vec<String>) -> Result<WebDriverProcess> {
        let process = ServerProcess::new("webdriver", |port| {
            let mut cmd = Command::new(&path);
            cmd.args(args)
                .arg(format!("--port={}", port));
            cmd
        })?;
        Ok(WebDriverProcess::Local(process))
    }
}