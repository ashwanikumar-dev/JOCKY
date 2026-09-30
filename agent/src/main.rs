pub mod agent;
pub mod collector;
pub mod config;
pub mod dispatcher;
pub mod evidence;
pub mod runtime;
#[path = "../../collectors/windows/mod.rs"]
pub mod windows_collectors;

use agent::JockyAgent;
use config::AgentConfig;
use std::time::Duration;
// use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("MAIN STARTED");

    println!("Creating config...");
    let config = AgentConfig::default();

    println!("Creating agent...");
    let mut agent = JockyAgent::new(config);

    println!("Agent created.");

    println!("Starting registration...");

    if let Err(e) = agent.register().await {
        println!("Registration error: {}", e);
    }

    println!("Registration finished.");
    println!("Entering poll loop...");

    loop {
        println!("Calling poll_tasks...");

        match agent.poll_tasks().await {
            Ok(Some(task)) => {
                println!("Received task: {}", task.task_id);

                if let Err(e) = agent.execute_task(&task).await {
                    println!("Task execution error: {}", e);
                }
            }
            Ok(None) => {
                println!("No task available");
            }
            Err(e) => {
                println!("Polling error: {}", e);
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}