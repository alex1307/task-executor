#[cfg(test)]
mod actor_unit_test {

    use std::thread;
    use std::{thread::sleep, time::Duration};

    use common_libs::configure_log4rs;
    use futures::executor::block_on;
    use futures::future::{self, FutureExt, join_all};
    use futures::join;
    use futures::stream::{self, StreamExt};
    
    use tokio::runtime::Handle;
    use tokio::spawn;
    use tokio::task::block_in_place;

    use crate::model::MessageType;
    use crate::model::actor::{exchange_command, start_actor, WordListReq, exchange_message};

    

    #[tokio::test]
    async fn ping_pong_test() {
        configure_log4rs();
        let arnold = "arnold".to_string();
        let silvester = "silvester".to_string();
        
        let pid1 = tokio::spawn(async {
            start_actor("arnold".to_string());
        }).boxed();
        
        let pid2 = tokio::spawn(async {
            start_actor("silvester".to_string());
        }).boxed();
        
        let tasks = vec![pid1, pid2];
        let task_futures = stream::iter(tasks).map(spawn);
        let handles = task_futures.collect::<Vec<_>>().await;
        future::join_all(handles).await;
        
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