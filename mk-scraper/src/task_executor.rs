use std::{sync::Arc, time::Duration};

use log::info;

pub static mut COUNTER: i32 = 1;

thread_local! {

    static CURRENT_HANDLER: Arc<CommandHandler> = Arc::new(Default::default());
}
#[derive(Debug, Clone)]
pub struct CommandHandler {
    sender: crossbeam::channel::Sender<Vec<u8>>,
    receiver: crossbeam::channel::Receiver<Vec<u8>>,
}

impl Default for CommandHandler {
    fn default() -> Self {
        let (tx, rx) = crossbeam::channel::bounded(0);
        unsafe {
            COUNTER += 1;
        }
        Self {
            sender: tx,
            receiver: rx,
        }
    }
}
pub async fn server_loop(max: i32) {
    println!("Server is started...");
    let reciver = CURRENT_HANDLER.with(|f| f.receiver.clone());
    let mut message_counter = 0;
    let mut cycles = 0;
    loop {
        match reciver.recv_timeout(Duration::from_secs(1)) {
            Ok(buffer) => {
                message_counter += 1;
                info!(
                    " => Message number #{}: {}",
                    message_counter,
                    String::from_utf8_lossy(&buffer)
                );
                if message_counter < max {
                    continue;
                } else {
                    info!("!!! Threshold maximum messages reached: {}", max);
                    break;
                }
            }
            Err(_) => {
                cycles += 1;
                if cycles < 5 {
                    continue;
                } else {
                    break;
                }
            }
        }
    }
}

pub fn client(id: i32) {
    println!("client #{} is started", id);
    let sender = CURRENT_HANDLER.with(|f| f.sender.clone());
    let msg = format!("Hello. I am #{}", id);
    sender
        .send_timeout(msg.as_bytes().to_vec(), Duration::from_secs(1))
        .unwrap();
}

pub fn ddd() {
    println!("testing");
    let _sender = CURRENT_HANDLER.with(|f| f.sender.clone());
    let _msg = format!("Hello. I am #{}", 21);
    //    sender.send(msg.as_bytes().to_vec()).unwrap();
}

#[cfg(test)]
mod task_executor_tests {
    use std::{
        thread::{self, sleep},
        time::Duration,
    };

    use futures::future::lazy;

    use crate::task_executor::COUNTER;

    use super::{client, CURRENT_HANDLER};

    #[tokio::test]
    async fn singleton_test1() {
        unsafe {
            assert_eq!(1, COUNTER);
        }

        let _f = tokio::spawn(async move {
            let _ = CURRENT_HANDLER;
        })
        .await;
        unsafe {
            assert_eq!(1, COUNTER);
        }
    }

    #[tokio::test]
    async fn singleton_test2() {
        unsafe {
            assert_eq!(1, COUNTER);
        }

        
        let a1 = thread::spawn(|| {
            println!("hello");
            client(1);
            println!("finished");
        });
        let a2 = thread::spawn(|| {
            println!("hello");
            client(2);
            println!("finished");
        });
        let a3 = thread::spawn(|| {
            println!("hello");
            client(3);
            println!("finished");
        });
        let a4 = thread::spawn(|| {
            println!("hello");
            client(4);
            println!("finished");
        });
        _ = a1.join();
        _ = a2.join();
        _ = a3.join();
        _ = a4.join();

        println!("all threads are started");
        sleep(Duration::from_secs(1));

        unsafe {
            assert_eq!(6, COUNTER);
        }
    }

    #[tokio::test]
    async fn test_tokio() {
        println!("started");
        let mut v = vec![];
        for i in 0..4 {
            v.push(tokio::spawn(lazy(move |_| {
                println!("Hello from task {}", i);                
            })));
        }
        futures::future::join_all(v).await;
    }
}
