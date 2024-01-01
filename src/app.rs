use sqlx::SqlitePool;

use iced::subscription::{self, Subscription};
use iced::widget::{Button, Column, Container, Text, text, row, scrollable};
use iced::{executor, futures, Application, Command, Element, Settings, Theme};

use futures::channel::mpsc;
use futures::sink::SinkExt;

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
            AppMessage::Worker(event) => {
                eprintln!("received {event:?}");
                match event {
                    Event::Connected(connection) => {
                        self.state = AppState::Connected(connection);
                        eprintln!("AppState connected!");
                    }
                    Event::LoadedTransactions(transactions) => {
                        self.transactions = transactions;
                    }
                    _ => {eprintln!("Something else happened {event:?}")},
                }
                Command::none()
            },
            AppMessage::LoadTransactions => {
                println!("Loading transactions");
                match &mut self.state {
                    AppState::Connected(connection) => {
                        eprintln!("connected");
                        connection.send(Message::LoadTransactions);
                    }
                    AppState::Disconnected => eprintln!("Can't load transactions, app is not connected."),
                }
                Command::none()
            },
        }
    }

    fn subscription(&self) -> Subscription<AppMessage> {
        connect(self.db.clone()).map(AppMessage::Worker)
    }

    fn view(&self) -> Element<Self::Message> {
        let title = Text::new("Transactions");
        let load_transactions_button =
            Button::new("Load Transactions").on_press(AppMessage::LoadTransactions);
        let transactions: Element<_> = scrollable(
            Column::with_children(
                self.transactions
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, transaction)| {
                    Element::from(row![text(i), text(transaction.description)])
                })
                .collect()
            )
        ).into();
        let col = Column::new().push(title).push(load_transactions_button).push(transactions);
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
            let _ = output.send(Event::Disconnected).await;

            loop {
                match &mut state {
                    State::Disconnected => {
                        let (sender, receiver) = mpsc::channel(100);
                        println!("created");

                        let _ = output.send(Event::Connected(Connection(sender))).await;

                        state = State::Connected(receiver);
                        println!("initialized");
                    }
                    State::Connected(receiver) => {
                        match receiver.try_next() {
                            Ok(Some(input)) => {
                                match input {
                                    Message::LoadTransactions => {
                                        println!("Got loading message");
                                        match list_transactions(&db).await {
                                            Ok(transactions) => {
                                                println!("{:?}", transactions);
                                                let _ = output.send(Event::LoadedTransactions(transactions)).await;
                                            },
                                            Err(e) => {
                                                eprintln!("couldn't load transactions {}", e);
                                            }
                                        }
                                    },
                                }
                            },
                            Ok(None) => continue,
                            Err(e) => {
                                eprintln!("{:?}", e);
                                tokio::time::sleep(
                                    tokio::time::Duration::from_secs(1),
                                )
                                .await;
                                // state = State::Disconnected;
                            },
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
    LoadedTransactions(Vec<Transaction>),
}

#[derive(Debug, Clone)]
pub struct Connection(mpsc::Sender<Message>);

impl Connection {
    pub fn send(&mut self, message: Message) {
        self.0
            .try_send(message)
            .expect("Send message to worker server");
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadTransactions,
}

