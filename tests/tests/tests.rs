use app::driver::ApplicationDriver;
use app::pages::ChatPage;

mod process;
mod webdriver;
mod app;

#[test]
fn two_users_can_exchange_messages() {
    let app = ApplicationDriver::new();

    let chat1 = ChatPage::new(&app);
    let chat2 = ChatPage::new(&app);

    chat2.enter_message("Hi! How are you?");
    chat2.click_send();

    chat2.shows_last_message(
        "You: Hi! How are you?");
    chat1.shows_last_message(
        "<User#2>: Hi! How are you?");

    chat1.enter_message("Hi there!");
    chat1.click_send();

    chat1.shows_messages(&["<User#2>: Hi! How are you?", "You: Hi there!"]);
    chat2.shows_messages(&["You: Hi! How are you?", "<User#1>: Hi there!"]);
}
