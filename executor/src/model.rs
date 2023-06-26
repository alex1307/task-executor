use common_libs::error::FmtResult;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use crate::CommandHandler;

pub type ProcessResult<T> = std::result::Result<T, ProcessError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessError {
    pub reason: Option<String>,
    pub error_type: ProcessErrorType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessErrorType {
    Continue,
    Break,
    Fatal,
}

pub trait Request {
    fn correlation_id(&self) -> String;
    fn from(&self) -> String;
    fn command(&self) -> Command;
    fn payload(&self) -> Option<Vec<u8>>;
}

pub trait Response {
    fn correlation_id(&self) -> String;
    fn status(&self) -> Status;
    fn payload(&self) -> Option<Vec<u8>>;
}

pub struct Worker {
    pub name: String,
    pub receiver: Option<crossbeam::channel::Receiver<Vec<u8>>>,
}

trait BinaryProcessor<Rhs = Self> {
    type Output;

    fn process(self, rhs: Rhs) -> ProcessResult<Self::Output>;
}

pub struct BaseMessage {
    pub from: String,
    pub payload: Option<Vec<u8>>,
    pub correlation_id: Option<String>,
    pub sequence: Option<u32>,
    pub headers: Option<Vec<BaseMessageHeader>>,
}

pub enum BaseMessageHeader {
    NoAck,
    Ack,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub from: Option<String>,
    pub correlation_id: String,
    pub message_type: MessageType,
    pub ack: Option<bool>,
    pub command: Option<Command>,
    pub status: Option<Status>,
    pub payload: Option<Vec<u8>>,
}

lazy_static! {
    static ref ACTORS: Mutex<HashMap<String, CommandHandler>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    Stop,
    Ping,
    Status,
    Execute,
    Pause,
    Undefined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Error,
    InProgress,
}

impl From<String> for Command {
    fn from(source: String) -> Self {
        match source.to_lowercase().trim() {
            "stop" => Command::Stop,
            "ping" => Command::Ping,
            "status" => Command::Status,
            "execute" => Command::Execute,
            "pause" => Command::Pause,
            _ => Command::Undefined,
        }
    }
}

impl From<String> for MessageType {
    fn from(source: String) -> Self {
        match source.to_lowercase().trim() {
            "respsone" => MessageType::Response,
            _ => MessageType::Request,
        }
    }
}

impl From<String> for Status {
    fn from(source: String) -> Self {
        match source.to_lowercase().trim() {
            "ok" => Status::Ok,
            "inprogress" => Status::InProgress,
            "in_progress" => Status::InProgress,
            _ => Status::Error,
        }
    }
}

pub mod actor {
    use crate::CommandHandler;
    
    use common_libs::{
        error::{AppError, AppResult},
        utils::from_binary,
    };
    
    use futures::join;
    use log::{error, info};
    use mk_scraper::{scrape_all, tokio_scrape_all};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Map;
    use std::{
        ops::ControlFlow,
        thread::{self, sleep},
        time::Duration,
        vec,
    };
    use tokio::runtime::{self, Builder};

    use super::{MessageType, ProcessResult, ACTORS};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum SystemCommand {
        Ping,
        Pong,
        Ack,
        NoAck,
        HealthCheck,
        Ok,
        Err,
        Seq(u32),
    }

    impl Default for MessageType {
        fn default() -> Self {
            Self::Request
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SystemMessage {
        pub from: String,
        pub payload: Option<Vec<u8>>,
        pub correlation_id: Option<String>,
        pub command: Option<SystemCommand>,
        pub message_type: MessageType,
    }

    pub trait Message {
        fn headers(&self) -> Vec<Map<String, String>>;
        fn command(&self) -> Option<String>;
        fn body(&self) -> Vec<u8>;
    }

    pub trait Reply {
        fn reply(&self) -> bool;
        fn message_type(&self) -> MessageType;
    }

    pub trait MessageHandler<Rhs = Self> {
        type Output;
        fn handle(self, message: Rhs) -> ProcessResult<Self::Output>;
    }

    fn handle_message<Req: ?Sized + DeserializeOwned, Res>(
        binary: Vec<u8>,
        handler: impl MessageHandler<Req, Output = Res>,
    ) -> ProcessResult<Res> {
        let req = from_binary::<Req>(binary).unwrap();
        handler.handle(req)
    }

    pub fn start_actor(actor_name: String) -> AppResult<()> {
        let mut workers = ACTORS.lock().unwrap();
        let wlh = WordListReq { words: vec![] };
        if workers.contains_key(&actor_name) {
            let msg = format!("worker with name: {} already has been started.", actor_name);
            return Err(AppError::throw(&msg));
        }
        let (tx, rx) = crossbeam::channel::unbounded();
        workers.insert(actor_name.clone(), CommandHandler { sender: tx });
        let cname = actor_name.clone();
        let _handler = thread::spawn(move || run(cname.clone(), rx, wlh));
        Ok(())
    }

    pub fn exchange_command(from: String, to: String, command: SystemCommand) {
        let actors = ACTORS.lock().unwrap();
        let destination = actors.get(&to);
        if destination.is_some() {
            let handler = destination.unwrap();
            let system_message = SystemMessage {
                from: from.clone(),
                payload: None,
                correlation_id: None,
                command: Some(command.clone()),
                message_type: MessageType::Command,
            };

            let result = serde_json::to_string(&system_message);
            let binary = Vec::from(result.unwrap().as_bytes());
            let _ = handler.send(binary);
        }
    }

    pub fn exchange_message(
        from: String,
        to: String,
        message: String,
        correlation_id: Option<String>,
        message_type: MessageType,
    ) {
        let actors = ACTORS.lock().unwrap();
        let destination = actors.get(&to);
        if destination.is_some() {
            let handler = destination.unwrap();
            let system_message = SystemMessage {
                from: from.clone(),
                payload: Some(Vec::from(message.as_bytes())),
                correlation_id,
                command: None,
                message_type,
            };

            let result = serde_json::to_string(&system_message);
            let binary = Vec::from(result.unwrap().as_bytes());
            info!("sending message: {:?}", system_message);
            let _ = handler.send(binary);
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WordListReq {
        pub words: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WordListRes {
        pub processed: Vec<String>,
        pub not_found: Vec<String>,
    }

    impl Reply for WordListReq {
        fn reply(&self) -> bool {
            true
        }

        fn message_type(&self) -> MessageType {
            MessageType::Request
        }
    }

    impl Reply for WordListRes {
        fn reply(&self) -> bool {
            false
        }

        fn message_type(&self) -> MessageType {
            MessageType::Response
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct WordListHander {}

    impl WordListReq {
        async fn do_it(list: Vec<String>) {
            scrape_all(list).await
        }
    }

    impl MessageHandler for WordListReq {
        type Output = WordListRes;

        fn handle(self, request: WordListReq) -> ProcessResult<Self::Output> {
            info!("message is being handled...");
            runtime::Runtime::new().unwrap().block_on(async move {
                let _value = tokio::spawn(tokio_scrape_all(request.words)).await.unwrap();
            });
            info!("scrape_all should be spawned...");
            Ok(WordListRes {
                processed: vec![],
                not_found: vec![],
            })
        }
    }

    pub async fn run<
        Req: ?Sized + DeserializeOwned,
        Res: Sized + Serialize + DeserializeOwned + Reply,
    >(
        registered_name: String,
        receiver: crossbeam::channel::Receiver<Vec<u8>>,
        message_handler: impl MessageHandler<Req>
            + MessageHandler<Req, Output = Res>
            + Clone
            + Send
            + Sync,
    ) -> AppResult<()> {
        info!("The actor {} is being started...", registered_name);
        loop {
            match receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(bin) => {
                    let res = from_binary::<SystemMessage>(bin);
                    if res.is_ok() {
                        let system_message = res.unwrap();
                        let from = system_message.from;

                        let _correlation_id = system_message.correlation_id;

                        if system_message.command.is_some() {
                            let command = system_message.command.unwrap();

                            if let ControlFlow::Break(_) =
                                process_command(command, &registered_name, &from)
                            {
                                continue;
                            }
                        }

                        info!("processing msg payload...");

                        if system_message.payload.is_some() {
                            let binary = system_message.payload.unwrap();
                            let fut = async move { 
                                info!("start sleeping for 2 sec...");
                                let f = sleep(Duration::from_secs(2));
                                info!("done");
                                f
                            };
                            let mut multi_threaded_runtime = Builder::new_multi_thread()
                            .worker_threads(4)
                            .thread_name("my-custom-name")
                            .thread_stack_size(3 * 1024 * 1024)
                            .build()
                            .unwrap();
                            let fhandler = multi_threaded_runtime.spawn(fut);
                            let _joined_future = join!(fhandler);
                        }

                        info!(
                            "Server {} reports that message has been handled successfully",
                            registered_name
                        );
                    } else {
                        error!("invalid.message");
                        break;
                    }
                }
                Err(_) => {
                    info!("the servier {} is running.....", &registered_name);
                    continue;
                }
            }
        }
        Ok(())
    }

    fn process_command(
        command: SystemCommand,
        registered_name: &str,
        from: &String,
    ) -> ControlFlow<()> {
        match command {
            SystemCommand::Ping => {
                info!(
                    "{} -> Command {:?} has been sent from {}",
                    &registered_name, command, &from
                );
                exchange_command(
                    registered_name.to_string(),
                    from.clone(),
                    SystemCommand::Pong,
                );
                return ControlFlow::Break(());
            }
            SystemCommand::Pong => {
                info!(
                    "{} -> Command {:?} has been sent from {}",
                    &registered_name, command, &from
                );
                return ControlFlow::Break(());
            }
            SystemCommand::Ack => {
                info!(
                    "{} -> Command {:?} has been sent from {}",
                    &registered_name, command, &from
                );
                exchange_command(
                    registered_name.to_string(),
                    from.clone(),
                    SystemCommand::Seq(123),
                );
            }
            SystemCommand::NoAck => {
                info!(
                    "{} -> Command {:?} has been sent from {}",
                    &registered_name, command, &from
                )
            }
            SystemCommand::HealthCheck => {
                info!(
                    "{} -> Command {:?} has been sent from {}",
                    &registered_name, command, &from
                );
                exchange_command(registered_name.to_string(), from.clone(), SystemCommand::Ok);
                return ControlFlow::Break(());
            }
            SystemCommand::Ok => {
                info!("Command {:?} has been sent from {}", command, &from);
                return ControlFlow::Break(());
            }
            SystemCommand::Err => {
                info!("Command {:?} has been sent from {}", command, &from);
                return ControlFlow::Break(());
            }
            SystemCommand::Seq(seq) => {
                info!(
                    "Command {:?} has been sent from {} with seq: {}",
                    command, &from, seq
                )
            }
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod actor_unit_test {

    use std::{thread::sleep, time::Duration};

    use common_libs::configure_log4rs;

    use crate::model::actor::{exchange_command, start_actor};

    use super::{
        actor::{exchange_message, WordListReq},
        MessageType,
    };

    #[tokio::test]
    async fn ping_pong_test() {
        configure_log4rs();
        let arnold = "arnold".to_string();
        let silvester = "silvester".to_string();
        let _ok = start_actor(silvester.clone());
        let _ok = start_actor(arnold.clone());
        exchange_command(
            silvester.clone(),
            arnold.clone(),
            crate::model::actor::SystemCommand::Ping,
        );
        exchange_command(
            silvester.clone(),
            arnold.clone(),
            crate::model::actor::SystemCommand::Ack,
        );
        exchange_command(
            silvester.clone(),
            arnold.clone(),
            crate::model::actor::SystemCommand::HealthCheck,
        );
        sleep(Duration::from_secs(2));

        let words = WordListReq {
            words: vec!["correct".to_string()],
        };
        let json = serde_json::to_string(&words).unwrap();
        exchange_message(
            silvester.clone(),
            arnold.clone(),
            json,
            Some("123".to_string()),
            MessageType::Request,
        );

        sleep(Duration::from_secs(5));
    }
}
