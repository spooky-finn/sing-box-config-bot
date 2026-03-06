use std::{
    io::prelude::*,
    net::TcpStream,
    path::Path,
    sync::{Arc, Mutex},
};

use sing_box_config_bot::{
    config::{self, get_env},
    utils::logger,
};
use ssh2::Session;
use tracing::{error, info};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerNode {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

impl ServerNode {
    pub fn ssh_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn ssh_user_address(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    pub keyfile: String,
    pub deploy_user: String,
    pub deploy_command: String,
    pub deploy_cwd: String,
    pub servers: Vec<ServerNode>,
}

impl DeployConfig {
    pub fn from_env() -> Result<Self, config::EnvError> {
        Ok(Self {
            keyfile: get_env("DEPLOY_KEYFILE")?,
            deploy_user: get_env("DEPLOY_USER")?,
            deploy_command: get_env("DEPLOY_COMMAND")?,
            deploy_cwd: get_env("DEPLOY_CWD")?,
            servers: Self::load_servers()?,
        })
    }

    fn load_servers() -> Result<Vec<ServerNode>, config::EnvError> {
        // Load servers from DEPLOY_SERVERS environment variable
        // Format: "name1:user1:host1:port1,name2:user2:host2:port2,..."
        let servers_str = get_env("DEPLOY_SERVERS")?;

        let servers: Result<Vec<ServerNode>, _> = servers_str
            .split(',')
            .map(|s| {
                let parts: Vec<&str> = s.split(':').collect();
                if parts.len() != 4 {
                    return Err(config::EnvError::Invalid(
                        "DEPLOY_SERVERS".to_string(),
                        format!("Expected name:user:host:port, got: {}", s),
                    ));
                }

                let port = parts[3].parse::<u16>().map_err(|_| {
                    config::EnvError::Invalid(
                        "DEPLOY_SERVERS".to_string(),
                        format!("Invalid port: {}", parts[3]),
                    )
                })?;

                Ok(ServerNode {
                    name: parts[0].to_string(),
                    user: parts[1].to_string(),
                    host: parts[2].to_string(),
                    port,
                })
            })
            .collect();

        servers
    }
}

fn main() {
    let config = DeployConfig::from_env().expect("Failed to load deployment configuration");
    logger::init("info", false);

    info!("Starting deployment to {} server(s)", config.servers.len());

    let config = Arc::new(config);
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];

    // Deploy to all servers in parallel
    for server in config.servers.clone() {
        let config = Arc::clone(&config);
        let results = Arc::clone(&results);

        let handle = std::thread::spawn(move || {
            info!("Deploying to {} ({})", server.name, server.host);

            let output = run_deployment(&config, &server);

            let mut results = results.lock().unwrap();
            results.push((server.name.clone(), output.is_ok()));

            match output {
                Ok(out) => {
                    info!("Deployment to {} completed successfully", server.name);
                    println!("\n=== {} ===\n{}", server.name, out);
                }
                Err(e) => {
                    error!(
                        server = %server.name,
                        error = %e,
                        "Deployment failed"
                    );
                    eprintln!("Deployment to {} failed: {}", server.name, e);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all deployments to complete
    for handle in handles {
        handle.join().expect("Deployment thread panicked");
    }

    // Summary
    let results = results.lock().unwrap();
    let success_count = results.iter().filter(|(_, ok)| *ok).count();
    let fail_count = results.iter().filter(|(_, ok)| !*ok).count();

    println!(
        "\n=== Deployment Summary ===\n{}: {}\n{}: {}",
        "Successful", success_count, "Failed", fail_count
    );

    if fail_count > 0 {
        std::process::exit(1);
    }
}

fn run_deployment(
    config: &DeployConfig,
    server: &ServerNode,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let stream = TcpStream::connect(server.ssh_address())?;
    let mut session = Session::new()?;
    session.set_tcp_stream(stream);
    session.handshake()?;

    session.userauth_pubkey_file(&config.deploy_user, None, Path::new(&config.keyfile), None)?;

    let mut channel = session.channel_session()?;

    // Change to working directory and execute command
    let command = format!("cd {} && {}", config.deploy_cwd, config.deploy_command);
    channel.exec(&command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;
    channel.wait_eof()?;
    channel.close()?;
    channel.wait_close()?;

    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err(format!("Command exited with status {}: {}", exit_status, output).into());
    }

    Ok(output)
}
