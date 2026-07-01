use std::env;
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

        // Uses the bundled bin/php-x86_64-apple-darwin (or whatever target)
        // If not bundled (like in dev), we can fallback to standard php via env var
        // but for Tauri sidecars it will try to find the binary named `php` sidecar.
        let mut command = Command::new_sidecar("php")
            .map_err(|e| format!("Failed to create sidecar command: {}", e))?
            .args(["artisan", "serve", "--host=127.0.0.1", &format!("--port={}", self.port)]);

        command = command.envs(vec![
            ("NATIVEPHP_RUNNING".to_string(), "true".to_string()),
            ("NATIVEPHP_API_URL".to_string(), format!("http://127.0.0.1:{}/_native/api/", api_port)),
            ("APP_PATH".to_string(), app_path)
        ].into_iter().collect());

        match command.spawn() {
            Ok((mut rx, child)) => {
                self.process = Some(child);
                
                // Spawn a thread to log stdout
                tokio::spawn(async move {
                    while let Some(event) = rx.recv().await {
                        if let CommandEvent::Stdout(line) = event {
                            println!("PHP: {}", line);
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
