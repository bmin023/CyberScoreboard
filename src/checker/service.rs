use std::{
    collections::BTreeMap, fs, time::Duration
};

use tokio::{process::Command, time::timeout};

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::resource_location;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Service {
    pub name: String,
    pub command: String,
    pub multiplier: u8,
}

impl Service {
    pub fn new(name: String, command: String, multiplier: u8) -> Self {
        Service {
            name,
            command,
            multiplier,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.name != "" && self.command != ""
    }
    #[tracing::instrument]
    pub async fn check_with_env(&self, env: &Vec<(String, String)>) -> Result<TestOutput, ()> {
        // get PATH from env
        let resource_dir = resource_location();
        let path = std::env::var("PATH").unwrap_or("/usr/bin:/bin:/usr/sbin:/sbin".to_string());
        let output = Command::new("bash")
            .current_dir(resource_dir)
            .arg("-c")
            .arg(&self.command)
            .env_clear()
            .env("PATH", path)
            .envs(env.clone())
            .output();
        let Ok(res) = timeout(Duration::from_secs(5), output).await else {
            debug!("{} timed out", self.name);
            return Ok(TestOutput {
                up: false,
                message: "".to_string(),
                error: "timeout".to_string(),
            });
        };
        let Ok(res) = res else {
            return Err(());
        };
        debug!(
            "{} is {}. stdout:{} stderr:{}",
            self.name,
            if res.status.success() { "UP" } else { "DOWN" },
            String::from_utf8_lossy(&res.stdout),
            String::from_utf8_lossy(&res.stderr)
        );
        Ok(TestOutput {
            up: res.status.success(),
            message: String::from_utf8_lossy(&res.stdout).to_string(),
            error: String::from_utf8_lossy(&res.stderr).to_string(),
        })
    }
}

pub struct TestOutput {
    pub up: bool,
    pub message: String,
    pub error: String,
}

#[derive(Deserialize)]
struct ServiceYaml {
    command: String,
    multiplier: u8,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum ServiceYamlForms {
    Command(String),
    Full(ServiceYaml),
}

pub fn load_services() -> Vec<Service> {
    let service_file = std::env::var("SB_SERVICES").unwrap_or_else(|_| "services.yaml".to_owned());
    let file = fs::read_to_string(format!("{}/{}", resource_location(), service_file))
        .expect(format!("{} should be in the resources directory", service_file).as_str());
    let yaml_services = serde_yaml::from_str::<BTreeMap<String, ServiceYamlForms>>(&file)
        .expect(format!("{} should be formatted correctly", service_file).as_str());
    let mut services = Vec::new();
    for service in yaml_services {
        match service {
            (name, ServiceYamlForms::Command(command)) => {
                services.push(Service::new(name, command, 1));
            }
            (name, ServiceYamlForms::Full(service)) => {
                services.push(Service::new(name, service.command, service.multiplier));
            }
        };
    }
    services
}
