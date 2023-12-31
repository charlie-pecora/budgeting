use sqlx::SqlitePool;

use std::fmt;

use iced::subscription::{self, Subscription};
use iced::widget::{Button, Column, Container, Text};
use iced::{executor, futures, Application, Command, Element, Settings, Theme};

use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

use crate::transactions::{list_transactions, Transaction};

pub fn run(db: SqlitePool) -> iced::Result {
    let settings = Settings {
        id: None,
        window: iced::window::Settings::default(),
        flags: Flags { db },
        default_font: iced::Font::default(),
        default_text_size: 16.0,
        antialiasing: false,
        exit_on_close_request: true,
    };
    App::run(settings)
}

pub struct Flags {
    db: SqlitePool,
}

pub struct App {
    db: SqlitePool,
    transactions: Vec<Transaction>,
    state: AppState,
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    LoadTransactions,
    Worker(Event),
}

enum AppState {
    Disconnected,
    Connected(Connection),
}

impl Default for AppState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = Flags;
    type Message = AppMessage;
    type Theme = Theme;

    fn new(flags: Self::Flags) -> (App, Command<Self::Message>) {
        (
            App {
                transactions: Vec::new(),
                db: flags.db,
                state: AppState::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Budgeting")
    }

    fn update(&mut self, message: AppMessage) -> Command<AppMessage> {
        match message {
            AppMessage::Worker(m) => {
                println!("{:?}", m);
                Command::none()
            }
            AppMessage::LoadTransactions => {
                println!("Loading transactions");
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        connect(self.db.clone()).map(AppMessage::Worker)
    }

    fn view(&self) -> Element<Self::Message> {
        let title = Text::new("Transactions");
        let load_transactions_button =
            Button::new("Load Transactions").on_press(AppMessage::LoadTransactions);
        let col = Column::new().push(title).push(load_transactions_button);
        Container::new(col)
            .center_x()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .into()
    }
}

pub fn connect(db: SqlitePool) -> Subscription<Event> {
    struct Connect;

    subscription::channel(
        std::any::TypeId::of::<Connect>(),
        100,
        |mut output| async move {
            let mut state = State::Disconnected;

            loop {
                match &mut state {
                    State::Disconnected => {
                        let (sender, receiver) = mpsc::channel(100);

                        let _ = output.send(Event::Connected(Connection(sender))).await;

                        state = State::Connected(receiver);
                    }
                    State::Connected(receiver) => {
                        let input = receiver.select_next_some().await;

                        match input {
                            _ => {}
                        }
                    }
                }
            }
        },
    )
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum State {
    Disconnected,
    Connected(mpsc::Receiver<Message>),
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Connection),
    Disconnected,
    MessageReceived(Message),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

impl Connection {
    pub fn send(&mut self, message: Message) {
        self.0
            .try_send(message)
            .expect("Send message to echo server");
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Connected,
    Disconnected,
    User(String),
}

impl Message {
    pub fn new(message: &str) -> Option<Self> {
        if message.is_empty() {
            None
        } else {
            Some(Self::User(message.to_string()))
        }
    }

    pub fn connected() -> Self {
        Message::Connected
    }

    pub fn disconnected() -> Self {
        Message::Disconnected
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::Connected => write!(f, "Connected successfully!"),
            Message::Disconnected => {
                write!(f, "Connection lost... Retrying...")
            }
            Message::User(message) => write!(f, "{message}"),
        }
    }
}
