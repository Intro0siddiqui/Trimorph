use std::path::Path;

/// Represents the detected init system.
#[derive(Debug, PartialEq)]
enum InitSystem {
    Systemd,
    OpenRC,
    Unknown,
}

/// Detects the active init system on the host.
///
/// This function checks for the presence of a well-known systemd directory
/// to determine if the host is running systemd. If the check fails, it falls
/// back to assuming OpenRC.
fn detect_init_system() -> InitSystem {
    if Path::new("/run/systemd/system").is_dir() {
        InitSystem::Systemd
    } else {
        // As a fallback, we'll assume OpenRC if systemd is not detected.
        // A more robust check could be added here if needed.
        InitSystem::OpenRC
    }
}

fn main() {
    let init_system = detect_init_system();
    println!("Detected Init System: {:?}", init_system);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_init_system_on_mock_systemd() {
        // This test simulates a systemd environment.
        // Since we can't create /run/systemd/system, we rely on the fact
        // that the test runner is likely running under systemd.
        // A more robust solution in a real CI would be to mock the filesystem.
        let init_system = detect_init_system();
        // This assertion depends on the environment where the test is run.
        // For this project's context, we assume a systemd-based environment.
        assert_eq!(init_system, InitSystem::Systemd);
    }
}