use std::path::Path;
use std::process::Command;
use std::fs;

/// Represents the detected init system.
#[derive(Debug, PartialEq)]
enum InitSystem {
    Systemd,
    OpenRC,
    Unknown,
}

/// Represents a jail configuration
#[derive(Debug)]
struct JailConfig {
    name: String,
    root: String,
    pkgmgr: String,
    pkgmgr_args: String,
    mounts: Vec<String>,
    env: Vec<String>,
}

/// Represents the jail runner with different backends
struct JailRunner {
    init_system: InitSystem,
}

impl JailRunner {
    /// Creates a new JailRunner instance
    fn new() -> Self {
        let init_system = Self::detect_init_system();
        Self { init_system }
    }

    /// Detects the active init system on the host.
    fn detect_init_system() -> InitSystem {
        if Path::new("/run/systemd/system").is_dir() {
            InitSystem::Systemd
        } else {
            // As a fallback, we'll assume OpenRC if systemd is not detected.
            // A more robust check could be added here if needed.
            InitSystem::OpenRC
        }
    }

    /// Runs a command in a jail using the appropriate backend
    fn run_command_in_jail(&self, jail_name: &str, command: Vec<String>) -> Result<(), String> {
        match self.init_system {
            InitSystem::Systemd => self.run_with_systemd_nspawn(jail_name, command),
            InitSystem::OpenRC => self.run_with_bubblewrap(jail_name, command),
            InitSystem::Unknown => Err("Unknown init system".to_string()),
        }
    }

    /// Runs command using systemd-nspawn backend
    fn run_with_systemd_nspawn(&self, jail_name: &str, command: Vec<String>) -> Result<(), String> {
        let config = self.load_jail_config(jail_name)
            .ok_or_else(|| format!("Jail config for {} not found", jail_name))?;

        // Create overlay directories
        let overlay_upper = format!("/var/tmp/trimorph_{}_upper_{}", jail_name, std::process::id());
        let overlay_work = format!("/var/tmp/trimorph_{}_work_{}", jail_name, std::process::id());

        // Use systemd-nspawn's native overlay support
        let mut nspawn_cmd = Command::new("systemd-nspawn");
        nspawn_cmd
            .arg("--quiet")
            .arg("--directory")
            .arg(&config.root)
            .arg("--overlay")
            .arg(format!("{}:{}:{}", &config.root, overlay_upper, overlay_work));

        // Add bind mounts
        nspawn_cmd.arg("--bind-ro=/tmp/.X11-unix");
        nspawn_cmd.arg("--bind-ro=/dev/dri");
        nspawn_cmd.arg("--bind-ro=/etc/resolv.conf");

        // Add package manager specific cache binds
        if config.pkgmgr.contains("apt") {
            nspawn_cmd.arg("--bind=/var/cache/apt/archives");
        } else if config.pkgmgr.contains("pacman") {
            nspawn_cmd.arg("--bind=/var/cache/pacman/pkg");
        }

        // Add custom mounts
        for mount in &config.mounts {
            nspawn_cmd.arg("--bind-ro").arg(mount);
        }

        // Add environment variables
        for env_var in &config.env {
            nspawn_cmd.arg("--setenv").arg(env_var);
        }

        // Add the package manager and its arguments
        nspawn_cmd.arg(&config.pkgmgr);
        
        // Parse and add pkgmgr_args
        for arg in self.parse_pkgmgr_args(&config.pkgmgr_args, &config.root) {
            nspawn_cmd.arg(arg);
        }

        // Add the user command
        for cmd_part in &command {
            nspawn_cmd.arg(cmd_part);
        }

        let output = nspawn_cmd.output()
            .map_err(|e| format!("Failed to execute systemd-nspawn: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Command failed: {}", stderr))
        }
    }

    /// Runs command using bubblewrap backend for OpenRC
    fn run_with_bubblewrap(&self, jail_name: &str, command: Vec<String>) -> Result<(), String> {
        let config = self.load_jail_config(jail_name)
            .ok_or_else(|| format!("Jail config for {} not found", jail_name))?;

        // Use bubblewrap to create a sandboxed environment
        let mut bwrap_cmd = Command::new("bwrap");
        
        // Basic sandboxing setup
        bwrap_cmd
            .arg("--unshare-all")
            .arg("--share-net")
            .arg("--ro-bind").arg("/usr").arg("/usr")
            .arg("--ro-bind").arg("/lib").arg("/lib")
            .arg("--ro-bind").arg("/lib64").arg("/lib64")
            .arg("--bind").arg(&config.root).arg("/")   // Mount the jail root as /
            .arg("--bind").arg("/tmp").arg("/tmp")
            .arg("--bind").arg("/dev").arg("/dev")
            .arg("--bind").arg("/proc").arg("/proc")
            .arg("--bind").arg("/sys").arg("/sys");

        // Add network access
        bwrap_cmd
            .arg("--bind").arg("/run").arg("/run")
            .arg("--bind").arg("/etc/resolv.conf").arg("/etc/resolv.conf");

        // Add display access if needed
        if Path::new("/tmp/.X11-unix").is_dir() {
            bwrap_cmd.arg("--bind").arg("/tmp/.X11-unix").arg("/tmp/.X11-unix");
        }

        // Add DRI access for graphics
        if Path::new("/dev/dri").is_dir() {
            bwrap_cmd.arg("--bind").arg("/dev/dri").arg("/dev/dri");
        }

        // Add environment variables
        for env_var in &config.env {
            let parts: Vec<&str> = env_var.split('=').collect();
            if parts.len() == 2 {
                bwrap_cmd.env(parts[0], parts[1]);
            }
        }

        // Add the package manager command
        bwrap_cmd.arg(&config.pkgmgr);
        
        // Parse and add pkgmgr_args
        for arg in self.parse_pkgmgr_args(&config.pkgmgr_args, &config.root) {
            bwrap_cmd.arg(arg);
        }

        // Add the user command
        for cmd_part in &command {
            bwrap_cmd.arg(cmd_part);
        }

        let output = bwrap_cmd.output()
            .map_err(|e| format!("Failed to execute bubblewrap: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Command failed: {}", stderr))
        }
    }

    /// Loads jail configuration from filesystem
    fn load_jail_config(&self, jail_name: &str) -> Option<JailConfig> {
        let config_path = format!("/etc/trimorph/jails.d/{}.conf", jail_name);
        
        if !Path::new(&config_path).exists() {
            return None;
        }

        // This is a simplified config parser - in a real implementation, 
        // you would want a more robust INI file parser
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                let mut name = jail_name.to_string();
                let mut root = String::new();
                let mut pkgmgr = String::new();
                let mut pkgmgr_args = String::new();
                let mut mounts = Vec::new();
                let mut env = Vec::new();

                for line in content.lines() {
                    let parts: Vec<&str> = line.splitn(2, '=').collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let key = parts[0].trim();
                    let value = parts[1].trim();

                    match key {
                        "name" => name = value.to_string(),
                        "root" => root = value.to_string(),
                        "pkgmgr" => pkgmgr = value.to_string(),
                        "pkgmgr_args" => pkgmgr_args = value.to_string(),
                        "mounts" => {
                            for mount in value.split(',') {
                                mounts.push(mount.trim().to_string());
                            }
                        }
                        "env" => {
                            for env_var in value.split(',') {
                                env.push(env_var.trim().to_string());
                            }
                        }
                        _ => {}
                    }
                }

                Some(JailConfig {
                    name,
                    root,
                    pkgmgr,
                    pkgmgr_args,
                    mounts,
                    env,
                })
            }
            Err(_) => None,
        }
    }

    /// Parses package manager arguments, replacing placeholders like {root}
    fn parse_pkgmgr_args(&self, args: &str, root: &str) -> Vec<String> {
        let processed = args.replace("{root}", root);
        processed.split_whitespace().map(|s| s.to_string()).collect()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <jail_name> <command> [args...]", args[0]);
        std::process::exit(1);
    }
    
    let jail_name = &args[1];
    let command: Vec<String> = args[2..].to_vec();
    
    let runner = JailRunner::new();
    
    match runner.run_command_in_jail(jail_name, command) {
        Ok(()) => {
            println!("Command executed successfully in jail '{}'", jail_name);
        }
        Err(e) => {
            eprintln!("Error running command in jail '{}': {}", jail_name, e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_init_system() {
        let init_system = JailRunner::detect_init_system();
        // Test will pass if the function doesn't panic
        assert!(init_system == InitSystem::Systemd || init_system == InitSystem::OpenRC || init_system == InitSystem::Unknown);
    }
}