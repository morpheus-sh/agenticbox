use bollard::container::{Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::connect_with_local_defaults()?;

    // Remove leftover container
    let _ = docker.remove_container("spike-stdio-rs", Some(RemoveContainerOptions { force: true, ..Default::default() })).await;

    // 1. Create + start container
    let spike_dir = std::env::current_dir()?;
    let script_path = spike_dir.join("echo_server.py");
    let script_path_str = script_path.to_string_lossy().replace("\\", "/");

    let config = Config {
        image: Some("python:3.12-slim"),
        cmd: Some(vec!["sleep", "300"]),
        host_config: Some(bollard::models::HostConfig {
            binds: Some(vec![format!("{}:/app/echo_server.py", script_path_str)]),
            ..Default::default()
        }),
        working_dir: Some("/app"),
        ..Default::default()
    };

    docker.create_container(
        Some(CreateContainerOptions { name: "spike-stdio-rs", ..Default::default() }),
        config,
    ).await?;
    docker.start_container("spike-stdio-rs", None::<StartContainerOptions<String>>).await?;
    println!("[spike] Container started");

    // 2. Create exec
    let exec = docker.create_exec("spike-stdio-rs", CreateExecOptions {
        cmd: Some(vec!["python3", "/app/echo_server.py"]),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        attach_stdin: Some(true),
        tty: Some(false),
        working_dir: Some("/app"),
        ..Default::default()
    }).await?;
    println!("[spike] Exec created: {}", exec.id);

    // 3. Start exec — detached first, then resize/attach
    let start_result = docker.start_exec(&exec.id, None).await?;

    match start_result {
        StartExecResults::Attached { mut output, mut input } => {
            println!("[spike] Attached! Sending + receiving concurrently...\n");

            // Sender task
            let sender = tokio::spawn(async move {
                let messages = vec![
                    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"name":"test"}}"#,
                    r#"{"jsonrpc":"2.0","id":2,"method":"tool_call","params":{"tool":"bash","cmd":"ls"}}"#,
                    r#"{"jsonrpc":"2.0","id":3,"method":"shutdown","params":{}}"#,
                ];

                for msg in &messages {
                    println!("[host → container] {}", msg);
                    input.write_all(msg.as_bytes()).await.unwrap();
                    input.write_all(b"\n").await.unwrap();
                    input.flush().await.unwrap();
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            });

            // Receiver task — read until we get 3 JSON responses or timeout
            let receiver = tokio::spawn(async move {
                let mut responses = Vec::new();
                let timeout = tokio::time::sleep(std::time::Duration::from_secs(10));
                tokio::pin!(timeout);

                loop {
                    tokio::select! {
                        msg = output.next() => {
                            match msg {
                                Some(Ok(bollard::container::LogOutput::StdOut { message })) => {
                                    let text = String::from_utf8_lossy(&message);
                                    for line in text.lines() {
                                        if line.starts_with('{') {
                                            println!("[container → host] {}", line);
                                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                                                responses.push(json);
                                                if responses.len() >= 3 {
                                                    return responses;
                                                }
                                            }
                                        }
                                    }
                                }
                                Some(Ok(bollard::container::LogOutput::StdErr { message })) => {
                                    let text = String::from_utf8_lossy(&message);
                                    for line in text.lines() {
                                        if !line.is_empty() {
                                            println!("[container stderr] {}", line);
                                        }
                                    }
                                }
                                Some(Err(e)) => {
                                    eprintln!("[spike] Stream error: {:?}", e);
                                    break;
                                }
                                None => break,
                                _ => {}
                            }
                        }
                        _ = &mut timeout => {
                            eprintln!("[spike] Timeout waiting for responses");
                            break;
                        }
                    }
                }
                responses
            });

            sender.await?;
            let responses = receiver.await?;

            println!("\n[spike] Received {} JSON-RPC responses", responses.len());
            for (i, r) in responses.iter().enumerate() {
                println!("  [{}] id={:?} echo={:?}",
                    i + 1,
                    r.get("id"),
                    r.get("result").and_then(|r| r.get("echo")).and_then(|e| e.as_str()));
            }

            if responses.len() == 3 {
                println!("\n✓ SUCCESS: bollard bidirectional stdio pipe works!");
            } else {
                println!("\n✗ PARTIAL: expected 3 responses, got {}", responses.len());
            }
        }
        StartExecResults::Detached => {
            println!("✗ FAILED: exec started detached");
        }
    }

    // Cleanup
    docker.remove_container("spike-stdio-rs", Some(RemoveContainerOptions { force: true, ..Default::default() })).await?;
    println!("[spike] Container removed");

    Ok(())
}
