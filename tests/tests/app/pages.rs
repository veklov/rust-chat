use std::future::Future;

use anyhow::{bail, Context, Result};
use thirtyfour::prelude::*;

use crate::app::driver::ApplicationDriver;
use crate::webdriver::client::WebDriver;

pub struct ChatPage<'a> {
    driver: &'a WebDriver,
    window: WindowHandle,
}

impl<'a> ChatPage<'a> {
    pub fn new(app: &'a ApplicationDriver) -> ChatPage<'a> {
        let driver = app.webdriver();
        driver.run(async {
            let window;
            if *app.app_url() == driver.current_url().await? {
                window = driver.new_tab().await?;
                driver.switch_to_window(window.clone()).await?;
            } else {
                window = driver.window().await?;
            }

            driver.goto(app.app_url()).await?;
            driver.demo_pause().await?;

            Ok(Self { driver, window })
        })
    }

    pub fn enter_message(&self, message: &str) {
        self.run(async {
            self.ensure_window().await?;

            let elem_text = self.driver.query_single(By::Css("input[type='text']")).await
                .context("Could not find the input for chat messages")?;
            elem_text.send_keys(message).await
                .context("Could not enter a message to the chat's input")?;

            self.driver.demo_pause().await
        })
    }

    pub fn click_send(&self) {
        self.run(async {
            self.ensure_window().await?;

            let elem_button = self.driver.query_single(By::Css("button[type='button']")).await
                .context("Could not find the send button")?;
            elem_button.click().await
                .context("Could not click the send button")?;

            self.driver.demo_pause().await
        })
    }

    pub fn shows_messages(&self, messages: &[&'static str]) {
        self.run(async {
            self.ensure_window().await?;

            let last_message = messages.last()
                .expect("The list of expected messages must not be empty");
            // we call this to fait for the UI to reflect the change
            let _ = self.shows_last_message0(last_message).await;

            let message_elements = self.driver.query(By::Css(" div p")).all().await
                .context("Could not get chat messages")?;
            let mut actual_messages = vec![];
            // we cannot use .iter().map() as async closures are not supported
            for message_element in message_elements {
                let actual_message = message_element.text().await
                    .context("Could not get chat message from its element")?;
                actual_messages.push(actual_message);
            }

            if actual_messages != messages {
                bail!("Expected messages:\n {}\nare not equal to actual ones:\n {}",
                    messages.join("\n "), actual_messages.join("\n "))
            }

            self.driver.demo_pause().await
        })
    }

    pub fn shows_last_message(&self, last_message: &'static str) {
        self.run(async {
            self.ensure_window().await?;

            self.shows_last_message0(last_message).await
        })
    }

    pub fn run<F, R>(&self, future: F) -> R
        where F: Future<Output=Result<R>>
    {
        self.driver.run(future)
    }

    async fn ensure_window(&self) -> Result<()> {
        if self.window != self.driver.window().await? {
            self.driver.switch_to_window(self.window.clone()).await?;
            self.driver.demo_pause().await?;
        }

        Ok(())
    }

    async fn shows_last_message0(&self, last_message: &'static str) -> Result<()> {
        let result = self.driver.query(By::Css("div p"))
            .with_text(last_message)
            .single().await;

        if let Err(_) = result {
            let last_message_element = self.driver.query(By::Css("div p:last-child"))
                .single().await
                .context("Could not get the last chat message")?;
            let actual_last_message = last_message_element.text().await?;
            if last_message != actual_last_message {
                bail!("The last message \"{}\" is not equal to the exepected one \"{}\"", actual_last_message, last_message);
            }
        }

        Ok(())
    }
}
