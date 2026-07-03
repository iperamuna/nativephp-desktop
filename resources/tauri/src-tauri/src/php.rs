use std::env;
// Trigger rebuild
use tauri::api::process::{Command, CommandChild, CommandEvent};

pub struct PhpServer {
    process: Option<CommandChild>,
    pub port: u16,
}

impl PhpServer {
    pub fn new() -> Self {
        Self {
            process: None,
            port: 8100,
        }
    }

    pub fn start(&mut self, api_port: u16) -> Result<(), String> {
        let app_path = env::var("APP_PATH").unwrap_or_else(|_| {
            env::current_dir().unwrap().to_string_lossy().to_string()
        });
        
        self.port = port_scanner::request_open_port().unwrap_or(8100);

        println!("Starting PHP Sidecar on port {}", self.port);

        // Uses a custom router.php located in src-tauri to properly set the working directory
        let server_path = format!("{}/router.php", env::current_dir().unwrap().to_string_lossy());
        let public_path = format!("{}/public", app_path);

        let storage_path = format!("{}/storage", app_path);
        let database_path = format!("{}/database/database.sqlite", app_path);

        let mut command = Command::new_sidecar("php")
            .map_err(|e| format!("Failed to create sidecar command: {}", e))?
            .args(["-S", &format!("127.0.0.1:{}", self.port), "-t", &public_path, &server_path]);

        command = command.envs(vec![
            ("NATIVEPHP_RUNNING".to_string(), "true".to_string()),
            ("NATIVEPHP_API_URL".to_string(), format!("http://127.0.0.1:{}/_native/api/", api_port)),
            ("APP_PATH".to_string(), app_path.clone()),
            ("NATIVEPHP_STORAGE_PATH".to_string(), storage_path),
            ("NATIVEPHP_DATABASE_PATH".to_string(), database_path),
            ("NATIVEPHP_SECRET".to_string(), "NativePHPTauriSecret".to_string())
        ].into_iter().collect());

        match command.spawn() {
            Ok((mut rx, child)) => {
                self.process = Some(child);
                
                // Spawn a thread to log stdout
                tauri::async_runtime::spawn(async move {
                    while let Some(event) = rx.recv().await {
                        match &event {
                            CommandEvent::Stdout(line) => println!("PHP: {}", line),
                            CommandEvent::Stderr(line) => println!("PHP STDERR: {}", line),
                            CommandEvent::Error(err) => println!("PHP ERROR: {}", err),
                            CommandEvent::Terminated(payload) => println!("PHP TERMINATED: {:?}", payload),
                            _ => println!("PHP OTHER EVENT: {:?}", event),
                        }
                    }
                });

                Ok(())
            },
            Err(e) => Err(format!("Failed to spawn PHP sidecar: {}", e))
        }
    }

    pub fn stop(&mut self) {
        if let Some(process) = self.process.take() {
            println!("Killing PHP server process...");
            let _ = process.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_server_initialization() {
        let server = PhpServer::new();
        assert_eq!(server.port, 8100);
    }
}
