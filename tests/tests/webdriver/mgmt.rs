use std::env;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use url::Url;
use wasm_pack::cache;
use wasm_pack::install::InstallMode;
use wasm_pack::test::webdriver;

#[derive(Copy, Clone)]
pub enum WebDriverType {
    Gecko,
    Safari,
    Chrome,
    Edge,
}

#[derive(Copy, Clone)]
pub enum BrowserType {
    Firefox,
    Safari,
    Chrome,
    Edge,
}

#[derive(Debug)]
pub enum WebDriverLocation {
    Local((PathBuf, Vec<String>)),
    Remote(Url),
}

pub struct WebDriverInfo {
    driver: WebDriverType,
    location: WebDriverLocation,
}

impl WebDriverInfo {
    pub fn browser(&self) -> BrowserType {
        match self.driver {
            WebDriverType::Gecko => BrowserType::Firefox,
            WebDriverType::Safari => BrowserType::Safari,
            WebDriverType::Chrome => BrowserType::Chrome,
            WebDriverType::Edge => BrowserType::Edge,
        }
    }

    pub fn location(&self) -> &WebDriverLocation {
        return &self.location;
    }
}

pub struct WebDriverManager;

impl WebDriverManager {
    /// Attempts to find an appropriate remote WebDriver server or server binary
    /// to execute tests with.
    /// Performs a number of heuristics to find one available, including:
    ///
    /// * Env vars like `GECKODRIVER_REMOTE` address of remote webdriver.
    /// * Env vars like `GECKODRIVER` point to the path to a binary to execute.
    /// * Env vars like `GECKODRIVER` contains `auto` point - download driver from its repo.
    /// * Otherwise, `PATH` is searched for an appropriate binary.
    ///
    /// In the last three cases a list of auxiliary arguments is also returned
    /// which is configured through env vars like `GECKODRIVER_ARGS` to support
    /// extra arguments to the driver's invocation.
    pub fn select_or_install() -> Result<WebDriverInfo> {
        let env_args = |name: &str| {
            env::var(format!("{}_ARGS", name.to_uppercase()))
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        };

        let drivers = [
            ("geckodriver", WebDriverType::Gecko, Some(Self::get_or_install_geckodriver as fn() -> Result<PathBuf>)),
            ("safaridriver", WebDriverType::Safari, None),
            ("chromedriver", WebDriverType::Chrome, Some(Self::get_or_install_chromedriver as fn() -> Result<PathBuf>)),
            ("msedgedriver", WebDriverType::Edge, None),
        ];

        // First up, if env vars like GECKODRIVER_REMOTE are present, use those
        // to allow forcing usage of a particular remote driver.
        for (driver_name, driver_type, ..) in drivers.iter() {
            let env = format!("{}_REMOTE", driver_name.to_uppercase());
            let url = match env::var(&env) {
                Ok(var) => match Url::parse(&var) {
                    Ok(url) => url,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };
            return Ok(WebDriverInfo { driver: *driver_type, location: WebDriverLocation::Remote(url) });
        }

        // Next, if env vars like GECKODRIVER are present, use those to
        // allow forcing usage of a particular local driver.
        // If value is AUTO, try to locate/download the driver
        for (driver_name, driver_type, locator) in drivers.iter() {
            let env = driver_name.to_uppercase();
            let path = match env::var(&env) {
                Ok(path) => path,
                Err(_) => continue,
            };
            let path_buf;
            if path.to_lowercase() == "auto" {
                path_buf = match locator {
                    None => bail!("Auto downloading is not supported for {}", driver_name),
                    Some(locator) => locator()?,
                }
            } else {
                path_buf = path.into();
            }
            return Ok(WebDriverInfo { driver: *driver_type, location: WebDriverLocation::Local((path_buf, env_args(driver_name))) });
        }

        // Next, check PATH. If we can find any supported driver, use that by
        // default.
        for path in env::split_paths(&env::var_os("PATH").unwrap_or_default()) {
            let found = drivers.iter().find(|(name, ..)| {
                path.join(name)
                    .with_extension(env::consts::EXE_EXTENSION)
                    .exists()
            });
            let (driver, driver_type, ..) = match found {
                Some(p) => p,
                None => continue,
            };
            return Ok(WebDriverInfo { driver: *driver_type, location: WebDriverLocation::Local((driver.into(), env_args(driver))) });
        }

        bail!(
            "\
failed to find a suitable WebDriver binary or remote running WebDriver to drive
testing; to configure the location of the webdriver binary you can use
environment variables like `GECKODRIVER=/path/to/geckodriver`
or `GECKODRIVER=auto` (for gecko and chrome it will be downloaded if not present locally)
or make sure that the binary is in `PATH`; to configure the address of remote webdriver you can
use environment variables like `GECKODRIVER_REMOTE=http://remote.host/`

This crate currently supports `geckodriver`, `chromedriver`, `safaridriver`, and
`msedgedriver`, although more driver support may be added! You can download these at:

    * geckodriver - https://github.com/mozilla/geckodriver/releases
    * chromedriver - https://chromedriver.chromium.org/downloads
    * msedgedriver - https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
    * safaridriver - should be preinstalled on OSX
    "
        )
    }

    fn get_or_install_geckodriver() -> Result<PathBuf> {
        let cache = cache::get_wasm_pack_cache()
            .map_err(|e| anyhow!(e))?;
        webdriver::get_or_install_geckodriver(&cache, InstallMode::Normal)
            .map_err(|e| anyhow!(e))
    }

    fn get_or_install_chromedriver() -> Result<PathBuf> {
        let cache = cache::get_wasm_pack_cache()
            .map_err(|e| anyhow!(e))?;
        webdriver::get_or_install_chromedriver(&cache, InstallMode::Normal)
            .map_err(|e| anyhow!(e))
    }
}