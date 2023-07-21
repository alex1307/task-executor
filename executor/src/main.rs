use common_libs::{self, configure_log4rs};
use executor::model::actor::{start_actor, exchange_command, SystemCommand};
use futures::{FutureExt, stream, StreamExt};

use tokio::spawn;

#[tokio::main]
async fn main() {
    configure_log4rs();
    let arnold = "arnold".to_string();
    let silvester = "silvester".to_string(); 

    let pid1 = tokio::spawn(async {
        _ = start_actor("arnold".to_string());
    }).boxed();
    
    let pid2 = tokio::spawn(async {
        _ = start_actor("silvester".to_string());
    }).boxed();
    
    let tasks = vec![pid1, pid2];
    let task_futures = stream::iter(tasks).map(spawn);
    let handles = task_futures.collect::<Vec<_>>().await;
    futures::future::join_all(handles).await;
    exchange_command(
        silvester.clone(),
        arnold.clone(),
        SystemCommand::Ping,
    );
    exchange_command(
        silvester.clone(),
        arnold.clone(),
        SystemCommand::Ack,
    );
    exchange_command(
        silvester.clone(),
        arnold.clone(),
        SystemCommand::HealthCheck,
    );
}
