pub mod model;
// mod tests;


#[macro_use]
extern crate lazy_static;

use std::fmt::Debug;






use model::{ProcessResult, Response, Request};



// lazy_static! {
//     static ref WORKERS: Mutex<HashMap<String, CommandHandler<Vec<u8>>> = {
//         let mut m = HashMap::new();
//         Mutex::new(m)
//     };
// }

// lazy_static! {
//     static ref CALLBACK: Mutex<HashMap<String, CommandHandle<Vec<u8>>> = {
//         let mut m = HashMap::new();
//         Mutex::new(m)
//     };
// }

// lazy_static! {
//     static ref FROM: Mutex<HashMap<String, String>> = {
//         let mut m = HashMap::new();
//         Mutex::new(m)
//     };
// }

pub trait Processor<S, T: Debug> {
    fn map(source: S) -> ProcessResult<T>;
    fn process(&self, message: T) -> ProcessResult<T>;
    fn from(&self) -> String;
    fn receive(&self) -> ProcessResult<T>;
}

pub trait ServerProcessor<S, Req: Debug + Request, Res: Debug + Response> {
    fn receive(&self) -> ProcessResult<Req>;
    fn validate(&self,  request: Req) -> bool;
    fn process(&self,   message: Req) -> ProcessResult<Res>;
    fn response(&self, response: Res) -> ProcessResult<Res>;
}



#[derive(Debug, Clone)]
pub struct CommandHandler {
    sender: crossbeam::channel::Sender<Vec<u8>>,
}

impl CommandHandler {
    pub fn send(&self, message: Vec<u8>) {
        let _ = self.sender.try_send(message);
    }
}

// pub fn start_worker(worker_name: String) -> AppResult<()> {
//     let mut workers = WORKERS.lock().unwrap();
//     if workers.contains_key(&worker_name) {
//         let msg = format!(
//             "worker with name: {} already has been started.",
//             worker_name
//         );
//         return Err(AppError::throw(&msg));
//     }
//     let (tx, rx) = crossbeam::channel::unbounded();
//     workers.insert(worker_name.clone(), CommandHandler { sender: tx });
//     let _handler = thread::spawn(move || server_loop(&worker_name, rx));

//     Ok(())
// }

// pub fn server_loop(
//     registered_name: &str,
//     receiver: crossbeam::channel::Receiver<Vec<u8>>,
// ) -> AppResult<()> {
//     println!("Server is started...");

//     loop {
//         match receiver.recv_timeout(Duration::from_secs(1)) {
//             Ok(buffer) => {
//                 let message = match from_binary::<Message>(buffer) {
//                     Ok(cmd) => cmd,
//                     Err(_) => continue,
//                 };

//                 println!("message is: {}", message);

//                 match message.command {
//                     Some(cmd) {
//                         match cmd {
                            
//                         Command::Stop => break,
//                         Command::Ping => {
//                             println!("do pong");
//                             pong(registered_name.to_string().clone(), message.from).unwrap()
//                         }
//                         _ => break,
//                         }
//                     },
//                     None => continue
//                 }
//             }
//             Err(_) => {
//                 println!("Hello from the server.....");
//                 continue;
//             }
//         }
//     }

//     Ok(())
// }

// pub fn run_server<S, Req: Debug + Request, Res: Debug + Response>(server: impl ServerProcessor<S, Req, Res>) -> AppResult<()>  {
//     println!("Server is started...");

//     loop {
//         match server.receive() {
            
//             Ok(in_message) => {
//                 if server.validate(in_message) {
                    
//                     let response = server.process(in_message);
                    
//                     match response {
//                         Err(err) => {
//                             if ProcessErrorType::Continue == err.error_type {
//                                 continue;
//                             } else {
//                                 break;
//                             }
//                         },
//                         Ok(res) => pong(in_message.from()),
//                     };

//                 } else {
//                     pong(receiver)
//                 }
//                 info!("message has been processed: {:?}", in_message)
//             }
//             Err(err) => {
//                 if ProcessErrorType::Continue == err.error_type {
//                     continue;
//                 } else {
//                     break;
//                 }
//             }
//         }
//     }
//     Ok(())
// }


// pub fn start_workders<S, T: Debug>(processor: Box<impl Processor<S, T>>) -> AppResult<()>  {
//     println!("Server is started...");

//     loop {
//         match processor.receive() {
//             Ok(in_message) => {
//                 info!("message has been processed: {:?}", in_message)
//             }
//             Err(err) => {
//                 if ProcessErrorType::Continue == err.error_type {
//                     continue;
//                 } else {
//                     break;
//                 }
//             }
//         }
//     }
//     Ok(())
// }

// impl Worker {
//     pub fn new(name: String) -> Self {
//         let mut workers = WORKERS.lock().unwrap();
//         if workers.contains_key(&name) {
//             let msg = format!("worker with name: {} already has been started.", name);
//             error!("failed: {}", msg);
//             return Worker {
//                 name: String::default(),
//                 receiver: None,
//             };
//         }
//         let (tx, rx) = crossbeam::channel::unbounded();
//         workers.insert(name.clone(), CommandHandler { sender: tx });
//         Worker {
//             name,
//             receiver: Some(rx),
//         }
//     }
// }





// impl Processor<Vec<u8>, Message> for Worker {
//     fn process(&self, message: Message) -> ProcessResult<Message> {
//         let orig_message = message.clone();
//         let message_type = message.message_type;
//         match message_type {
//             model::MessageType::Request => MessageProcessor::process(self, Box::new(message)),
//             _ => todo!()
//         };
//         if message.command.is_some() {
//             let command = message.command.unwrap();
//             match command {
//                 Command::Stop => {
//                     return Err(ProcessError {
//                         reason: None,
//                         error_type: ProcessErrorType::Break,
//                     })
//                 }
//                 Command::Ping => pong(self.name.clone(), message.from).unwrap(),
//                 cmd => error!("command.not.implemented: {:?}.", cmd),
//             };
//         } else {

//         }
       
//         Ok(orig_message)
//     }

//     fn from(&self) -> String {
//         self.name.clone()
//     }

//     fn receive(&self) -> ProcessResult<Message> {
//         let rcv = match &self.receiver {
//             Some(rcv1) => rcv1,
//             None => {
//                 return Err(ProcessError {
//                     reason: None,
//                     error_type: ProcessErrorType::Fatal,
//                 })
//             }
//         };

//         match rcv.recv_timeout(Duration::from_secs(1)) {
//             Ok(binary) => {
//                 let message = Self::map(binary)?;
//                 Ok(self.process(message)?)
//             }

//             Err(_) => {
//                 info!("{} is waiting for a message....", self.from());
//                 Err(ProcessError {
//                     reason: None,
//                     error_type: ProcessErrorType::Continue,
//                 })
//             }
//         }
//     }


// impl SendMessage<Message> for Worker {
//     fn send_message_to(message: Message, destination: String) -> ProcessResult<()> {
//         let cmd_handler = WORKERS.lock().unwrap().get(&destination).unwrap().clone();        
//         let json = serde_json::to_string(&message).unwrap();
//         info!("Sending: {} to {}", json, &destination);
//         let _ = cmd_handler.sender.try_send(json.as_bytes().to_vec());
//         Ok(())
//     }
// }

// fn pong(receiver: String) -> ProcessResult<Box<dyn TResponse>> {
//     let handler = match WORKERS.lock().unwrap().get(&receiver) {
//         Some(handler) => handler.clone(),
//         None => {
//             let msg = format!("Pong command failed. {} not found.", receiver);
//             return Err(ProcessError {
//                 error_type: ProcessErrorType::Continue,
//                 reason: Some("client.not.found".to_string()),
//             });
//         }
//     };
//     let pong = Message {
//         from: None,
//         ack: Some(true),
//         command: None,
//         payload: Some("pong".to_string().as_bytes().to_vec()),
//         correlation_id: String::default(),
//         message_type: MessageType::Response,
//         status: Some(Status::InProgress),
//     };
//     let json = serde_json::to_string(&pong).unwrap();
//     println!("sending pong command: {}", json);
//     let _ = handler.sender.try_send(json.clone().as_bytes().to_vec());

//     Ok(Box::new(pong))
// }

// #[test]
// fn start_server_test() {
//     let _ok = start_worker("alex_is_working".to_string());
//     let cmd_handler = WORKERS
//         .lock()
//         .unwrap()
//         .get("alex_is_working")
//         .unwrap()
//         .clone();
//     sleep(Duration::from_secs(1));
//     cmd_handler.sender.try_send("xxxxxx".as_bytes().to_vec());
//     sleep(Duration::from_secs(2));
//     assert!(true);
//     assert_eq!(1, WORKERS.lock().unwrap().len());
// }

// #[test]
// fn ping_pong_test() {
//     let _ok = start_worker("the_server".to_string());
//     let _ok = start_worker("the_client".to_string());
//     let cmd_handler = WORKERS.lock().unwrap().get("the_server").unwrap().clone();
//     let ping = Message {
//         command: Command::Ping,
//         from: "the_client".to_string(),
//         payload: None,
//         ack: None,
//         correlation_id: "1".to_string(),
//     };
//     let json = serde_json::to_string(&ping).unwrap();
//     println!("Sending: {}", json);
//     let _ = cmd_handler.sender.try_send(json.as_bytes().to_vec());
//     sleep(Duration::from_secs(2));
// }

// #[test]
// fn ping_pong_workers_test() {
//     configure_log4rs();
//     let worker1 = Worker::new("worker1".to_string());
//     let worker2 = Worker::new("worker2".to_string());
//     let _ = thread::spawn(move || start_workders(Box::new(worker1)));
//     let _ = thread::spawn(move || start_workders(Box::new(worker2)));
//     let ping = Message {
//         command: Command::Ping,
//         from: "worker1".to_string(),
//         payload: None,
//         ack: None,
//         correlation_id: "1".to_string(),
//     };
//     let _ = Worker::send_message_to(ping, "worker2".to_string());
//     sleep(Duration::from_secs(2));
// }
