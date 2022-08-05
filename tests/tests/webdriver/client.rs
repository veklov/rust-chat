use std::future::Future;
use std::ops::Deref;
use std::time::Duration;

use anyhow::Result;
use thirtyfour::extensions::query::ElementQuery;
use thirtyfour::prelude::*;
use url::Url;

use crate::webdriver::mgmt::BrowserType;

pub struct WebDriver {
    runtime: tokio::runtime::Runtime,
    driver: Option<thirtyfour::WebDriver>,
    demo_mode: bool,
}

impl WebDriver {
    pub fn new(browser: BrowserType, demo_mode: bool, url: &Url) -> Result<WebDriver> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        let caps = Self::determine_capabilities(browser, demo_mode)?;

        let driver = runtime.block_on(async move {
            thirtyfour::WebDriver::new(&url.to_string(), caps).await
        })?;

        Ok(WebDriver { runtime, driver: Some(driver), demo_mode })
    }

    pub fn run<F, R>(&self, future: F) -> R
        where F: Future<Output=Result<R>>
    {
        // Top level test methods panic on error by deisgn
        self.runtime.block_on(future).unwrap()
    }

    pub async fn query_single(&self, by: By) -> WebDriverResult<WebElement> {
        self.query(by).single().await
        // ^^^ Not self.driver.find(by) - as the "query" uses polling and more stable during page updates.
    }

    pub fn query(&self, by: By) -> ElementQuery {
        self.driver().query(by)
            // there is no way to change it globally :((( and default timeout is 20 seconds!!!
            .wait(Duration::from_secs(3), Duration::from_millis(500))
    }

    pub async fn demo_pause(&self) -> Result<()> {
        if self.demo_mode {
            tokio::time::sleep(Duration::from_secs(1)).await
        }

        Ok(())
    }

    fn determine_capabilities(browser: BrowserType, demo_mode: bool) -> Result<Capabilities> {
        Ok(match browser {
            BrowserType::Firefox => {
                let mut capabilities = DesiredCapabilities::firefox();
                if !demo_mode {
                    capabilities.set_headless()?;
                }
                capabilities.into()
            }
            BrowserType::Safari => {
                DesiredCapabilities::safari().into()
            }
            BrowserType::Chrome => {
                let mut capabilities = DesiredCapabilities::chrome();
                if !demo_mode {
                    capabilities.set_headless()?;
                }
                capabilities.into()
            }
            BrowserType::Edge => {
                DesiredCapabilities::edge().into()
            }
        })
    }

    fn driver(&self) -> &thirtyfour::WebDriver {
        // Some(_) is assigned in constructor, and None only in Drop
        self.driver.as_ref().unwrap()
    }
}

impl Deref for WebDriver {
    type Target = thirtyfour::WebDriver;

    fn deref(&self) -> &Self::Target {
        self.driver()
    }
}

impl Drop for WebDriver {
    fn drop(&mut self) {
        // Some(_) is assigned in constructor
        let driver = self.driver.take().unwrap();

        if let Err(error) = self.runtime.block_on(driver.quit()) {
            eprintln!("Could not stop driver: {}, {:?}", error, error)
        }
    }
}
