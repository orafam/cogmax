use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Macos,
    Windows,
}

pub fn platform() -> Result<Platform, String> {
    match env::consts::OS {
        "linux" => Ok(Platform::Linux),
        "macos" => Ok(Platform::Macos),
        "windows" => Ok(Platform::Windows),
        os => Err(format!("unsupported operating system: {os}")),
    }
}

#[allow(dead_code)]
pub fn checksum(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[allow(dead_code)]
pub fn verify_checksum(bytes: &[u8], expected: &str) -> Result<(), String> {
    let expected = expected.split_whitespace().next().unwrap_or_default();
    if checksum(bytes).eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err("release checksum mismatch".into())
    }
}

fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} failed with {status}"))
    }
}

pub fn install() -> Result<(), String> {
    let platform = platform()?;
    let binary = env::current_exe().map_err(|e| e.to_string())?;
    let target = install_path(platform);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(&binary, &target).map_err(|e| format!("cannot install binary: {e}"))?;
    install_skill(&target).map_err(|e| format!("cannot install skill: {e}"))?;
    service_file(platform, &target).map_err(|e| format!("cannot configure service: {e}"))?;
    println!("Cogmax installed at {}", target.display());
    Ok(())
}

fn install_skill(binary: &Path) -> io::Result<()> {
    let root = binary
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("invalid installation path"))?;
    let skill_root = root.join("share/cogmax/skill");
    fs::create_dir_all(skill_root.join("scripts"))?;
    fs::write(
        skill_root.join("SKILL.md"),
        include_bytes!("../../../skill/SKILL.md"),
    )?;
    let script = skill_root.join("scripts/memory");
    fs::write(&script, include_bytes!("../../../skill/scripts/memory"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&script)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(script, permissions)?;
    }
    Ok(())
}

pub fn uninstall() -> Result<(), String> {
    let p = platform()?;
    match p {
        Platform::Linux => {
            let _ = run(
                "systemctl",
                &["--user", "disable", "--now", "cogmax.service"],
            );
            let _ = fs::remove_file(service_path(p));
        }
        Platform::Macos => {
            let _ = run(
                "launchctl",
                &["unload", service_path(p).to_str().unwrap_or_default()],
            );
            let _ = fs::remove_file(service_path(p));
        }
        Platform::Windows => {
            let _ = run("sc.exe", &["delete", "Cogmax"]);
        }
    }
    let _ = fs::remove_file(install_path(p));
    println!("Cogmax uninstalled; data was preserved.");
    Ok(())
}

pub fn lifecycle(action: &str) -> Result<(), String> {
    let p = platform()?;
    match p {
        Platform::Linux => run("systemctl", &["--user", action, "cogmax.service"]),
        Platform::Macos => match action {
            "start" | "restart" => run(
                "launchctl",
                &["load", service_path(p).to_str().unwrap_or_default()],
            ),
            "stop" => run(
                "launchctl",
                &["unload", service_path(p).to_str().unwrap_or_default()],
            ),
            "status" => run("launchctl", &["list", "cogmax"]),
            _ => Err(format!("unsupported lifecycle action: {action}")),
        },
        Platform::Windows => run("sc.exe", &[action, "Cogmax"]),
    }
}

fn install_path(p: Platform) -> PathBuf {
    let configured_root = env::var_os("COGMAX_INSTALL_ROOT").map(PathBuf::from);
    install_path_with_root(p, configured_root.as_deref())
}

fn install_path_with_root(p: Platform, root: Option<&Path>) -> PathBuf {
    if let Some(root) = root {
        return match p {
            Platform::Linux | Platform::Macos => root.join("bin/cogmax"),
            Platform::Windows => root.join("Cogmax/cogmax.exe"),
        };
    }
    match p {
        Platform::Linux => PathBuf::from("/usr/local/bin/cogmax"),
        Platform::Macos => PathBuf::from("/usr/local/bin/cogmax"),
        Platform::Windows => env::var_os("ProgramFiles")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:\\Program Files"))
            .join("Cogmax\\cogmax.exe"),
    }
}
fn service_path(p: Platform) -> PathBuf {
    match p {
        Platform::Linux => linux_service_path(
            env::var_os("HOME").map(PathBuf::from).as_deref(),
            env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).as_deref(),
        ),
        Platform::Macos => env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library/LaunchAgents/com.cogmax.service.plist"),
        Platform::Windows => PathBuf::from("Cogmax"),
    }
}

fn linux_service_path(home: Option<&Path>, config_home: Option<&Path>) -> PathBuf {
    config_home
        .map(Path::to_path_buf)
        .or_else(|| home.map(|path| path.join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join("systemd/user/cogmax.service")
}

fn service_file(p: Platform, binary: &Path) -> io::Result<()> {
    match p {
        Platform::Linux => {
            let path = service_path(p);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, format!("[Unit]\nDescription=Cogmax memory service\n[Service]\nExecStart={} serve\nRestart=on-failure\n[Install]\nWantedBy=default.target\n", binary.display()))?;
        }
        Platform::Macos => {
            let path = service_path(p);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist version=\"1.0\"><dict><key>Label</key><string>com.cogmax.service</string><key>ProgramArguments</key><array><string>{}</string><string>serve</string></array><key>RunAtLoad</key><true/></dict></plist>", binary.display()))?;
        }
        Platform::Windows => {
            run(
                "sc.exe",
                &[
                    "create",
                    "Cogmax",
                    "binPath=",
                    &format!("{} serve", binary.display()),
                ],
            )
            .map_err(io::Error::other)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checksum_is_stable() {
        assert_eq!(
            checksum(b"cogmax"),
            "2b938b59828dfad853ba6488decd9ed69059dda2e3ea2d1c3c606131a33e8490"
        );
    }
    #[test]
    fn invalid_checksum_is_rejected() {
        assert!(verify_checksum(b"cogmax", "bad").is_err());
    }

    #[test]
    fn install_path_can_use_a_configured_root() {
        let root = PathBuf::from("/tmp/cogmax-test");
        assert_eq!(
            install_path_with_root(Platform::Linux, Some(&root)),
            root.join("bin/cogmax")
        );
    }

    #[test]
    fn linux_service_path_uses_home_when_config_home_is_missing() {
        assert_eq!(
            linux_service_path(Some(Path::new("/home/alice")), None),
            PathBuf::from("/home/alice/.config/systemd/user/cogmax.service")
        );
    }
}
