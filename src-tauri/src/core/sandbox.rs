use bollard::Docker;
use bollard::service::ContainerCreateBody;
use bollard::models::HostConfig;
use bollard::query_parameters::{CreateContainerOptions, StartContainerOptions, RemoveContainerOptions, WaitContainerOptions};
use bollard::container::LogOutput;
use futures_util::stream::StreamExt;
use std::default::Default;

pub struct SandboxManager {
    docker: Docker,
}

impl SandboxManager {
    pub fn new() -> Result<Self, String> {
        // Conectar al socket por defecto (Named Pipe en Windows, Unix Socket en Linux/Mac)
        let docker = Docker::connect_with_local_defaults().map_err(|e| format!("Docker connection error: {}", e))?;
        Ok(SandboxManager { docker })
    }

    pub async fn run_python_code(&self, code: &str) -> Result<String, String> {
        self.run_code("python:3.9-alpine", &["python", "-c", code]).await
    }

    pub async fn run_node_code(&self, code: &str) -> Result<String, String> {
        self.run_code("node:18-alpine", &["node", "-e", code]).await
    }

    async fn run_code(&self, image: &str, cmd: &[&str]) -> Result<String, String> {
        if !crate::core::security::SecurityPolicy::is_image_allowed(image) {
            return Err(format!("Security Violation: Image {} is not allowed", image));
        }

        // 1. Asegurar que la imagen existe (esto puede tardar la primera vez)
        // En producción deberíamos hacer pull explícito o asumir que están cacheadas.
        // Por ahora confiamos en que Docker intentará usarla o fallará si no está y la política es tal.
        // Para robustez, idealmente: pull_image.

        // 2. Crear Contenedor
        // HINT/PISTA: We use `ContainerCreateBody` here instead of `ContainerConfig` because `bollard` 0.14+
        // refined its types. This struct maps correctly to the Docker API JSON body.
        // Usamos `ContainerCreateBody` aquí en lugar de `ContainerConfig` porque `bollard` 0.14+
        // refinó sus tipos. Esta estructura se mapea correctamente al cuerpo JSON de la API de Docker.
        let config = ContainerCreateBody {
            image: Some(image.to_string()),
            cmd: Some(cmd.iter().map(|s| s.to_string()).collect()),
            host_config: Some(HostConfig {
                network_mode: Some("none".to_string()), // Sin red / Network isolation
                memory: Some(128 * 1024 * 1024), // 128MB RAM Limit
                // auto_remove: Some(true), // Auto-remove is risky if we want logs / Es arriesgado si queremos logs
                ..Default::default()
            }),
            ..Default::default()
        };

        let container_name = format!("sandbox_{}", uuid::Uuid::new_v4()); // Necesitamos uuid crate o generar random string
        
        // Usamos un nombre aleatorio simple por ahora si no tenemos uuid crate añadido
        // Mejor añadir uuid a cargo.toml o usar timestamp
        let _id = self.docker.create_container(
            Some(CreateContainerOptions{ name: Some(container_name.clone()), ..Default::default() }),
            config,
        ).await.map_err(|e| format!("Failed to create container: {}", e))?.id;

        // 3. Iniciar
        self.docker.start_container(&container_name, None::<StartContainerOptions>)
            .await.map_err(|e| format!("Failed to start container: {}", e))?;

        // 4. Wait for it to finish (with timeout logic ideally, but blocking for now)
        // Bollard wait_container returns a stream
        let mut wait_stream = self.docker.wait_container(
            &container_name,
            None::<WaitContainerOptions>
        );
        
        // Simple timeout logic could be implemented with tokio::select!
        // Por brevedad, esperamos resultado
        let _ = wait_stream.next().await; 

        // 5. Logs
        let mut logs_stream = self.docker.logs(
            &container_name,
            Some(bollard::query_parameters::LogsOptions {
                stdout: true,
                stderr: true,
                ..Default::default()
            })
        );

        let mut output = String::new();
        while let Some(log_result) = logs_stream.next().await {
            match log_result {
                Ok(LogOutput::StdOut{ message }) => output.push_str(&String::from_utf8_lossy(&message)),
                Ok(LogOutput::StdErr{ message }) => output.push_str(&String::from_utf8_lossy(&message)),
                Ok(_) => {},
                Err(e) => output.push_str(&format!("Error reading logs: {}", e)),
            }
        }

        // 6. Cleanup
        let _ = self.docker.remove_container(
            &container_name,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            })
        ).await;

        Ok(output)
    }
}
