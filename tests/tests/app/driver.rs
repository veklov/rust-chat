use std::env;
use std::process::Command;

use anyhow::{Context, Result};
use url::Url;

use crate::process::server::ServerProcess;
use crate::webdriver::client::WebDriver;
use crate::webdriver::mgmt::WebDriverManager;
use crate::webdriver::process::WebDriverProcess;

pub struct ApplicationDriver {
    webdriver_client: WebDriver,
    #[allow(dead_code)]
    webdriver_proc: WebDriverProcess,
    application: ServerProcess,
}

impl ApplicationDriver {
    pub fn new() -> ApplicationDriver {
        Self::_new()
            // Top level test methods panic on error by deisgn
            .unwrap()
    }

    pub fn app_url(&self) -> &Url {
        self.application.get_url()
    }

    pub fn webdriver(&self) -> &WebDriver {
        &self.webdriver_client
    }

    fn _new() -> Result<ApplicationDriver> {
        let application = Self::start_application()
            .context("Could not start application process")?;

        let driver_info = WebDriverManager::select_or_install()?;

        let demo_mode = Self::demo_mode();

        let webdriver_proc = WebDriverProcess::new(driver_info.location())
            .context("Could not determine/start webdriver process")?;

        let webdriver_client = WebDriver::new(driver_info.browser(), demo_mode, webdriver_proc.get_url())
            .context("Could not create webdriver client")?;

        Ok(ApplicationDriver { webdriver_client, webdriver_proc, application })
    }

    fn start_application() -> Result<ServerProcess> {
        ServerProcess::new("application", |port| {
            let mut cmd = Command::new("../target/debug/backend");
            cmd.env("PORT", port.to_string());
            cmd.env("STATIC_ASSETS", "../frontend/dist");
            cmd
        })
    }

    fn demo_mode() -> bool {
        env::var("DEMO_MODE").map_or(false, |v| v == "1" || v.to_lowercase() == "true")
    }
}
