use web_sys::HtmlInputElement;
use yew::prelude::*;

use chat::Chat;

mod chat;

struct FullStackApp {
    chat: Chat,
    messages: Vec<String>,
    input: NodeRef,
}

pub enum Msg {
    Received(String),
    Send,
}

impl Component for FullStackApp {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let chat = Chat::new(move |s| link.send_message(Msg::Received(s)));
        Self { chat, messages: vec![], input: NodeRef::default() }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Received(message) => {
                self.messages.push(message);
                true
            }
            Msg::Send => {
                let input = self.input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = input.value();
                    self.messages.push("You: ".to_owned() + &message);
                    self.chat.send(message);
                    input.set_value("");
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let send = ctx.link().callback(|_| Msg::Send);
        html! {
            <div>
                <h1>{"Rust chat"}</h1>
                <div>
                    {
                        self.messages.iter()
                            .map(|message| {
                                html! {
                                    <p>{message}</p>
                                }
                            })
                            .collect::<Html>()
                    }
                </div>
                <input type="text" ref={self.input.clone()}/>
                <button type="button" onclick={send}>{"Send"}</button>
            </div>
        }
    }
}

pub fn main() {
    console_log::init()
        .expect("error initializing log");
    yew::start_app::<FullStackApp>();
}
