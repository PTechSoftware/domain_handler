use std::fs;
use std::io::{Write, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

// === Paths ===

fn home_dir() -> PathBuf {
    dirs::home_dir().expect("No home directory found")
}

fn local_bin() -> PathBuf {
    home_dir().join(".local/bin")
}

fn config_dir() -> PathBuf {
    home_dir().join(".config/domainhdlr")
}

fn systemd_user_dir() -> PathBuf {
    home_dir().join(".config/systemd/user")
}

fn service_path() -> PathBuf {
    systemd_user_dir().join("domainhdlr.service")
}

fn bin_dest() -> PathBuf {
    local_bin().join("domainhdlr")
}

fn config_dest() -> PathBuf {
    config_dir().join("domainhdlr.json")
}

// === Add ~/.local/bin to PATH ===

fn ensure_local_bin_in_path() -> anyhow::Result<()> {
    let home = home_dir();
    let bashrc_path = home.join(".bashrc");
    let export_line = r#"export PATH="$HOME/.local/bin:$PATH""#;

    // Leer o crear .bashrc
    let mut content = if bashrc_path.exists() {
        fs::read_to_string(&bashrc_path)?
    } else {
        String::new()
    };

    if !content.contains(export_line) {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&bashrc_path)?;
        writeln!(file, "\n{}", export_line)?;
        println!("➕ Added ~/.local/bin to PATH in .bashrc");
    }

    Ok(())
}

// === Install ===

pub fn install_service() -> anyhow::Result<()> {
    // Crear carpetas necesarias
    fs::create_dir_all(local_bin())?;
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(systemd_user_dir())?;

    // Copiar ejecutable
    fs::copy("domainhdlr", &bin_dest())?;
    fs::set_permissions(&bin_dest(), fs::Permissions::from_mode(0o755))?;

    // Copiar config si existe
    if Path::new("domainhdlr.json").exists() {
        fs::copy("domainhdlr.json", &config_dest())?;
        fs::set_permissions(&config_dest(), fs::Permissions::from_mode(0o644))?;
    }

    // Agregar ~/.local/bin al PATH
    ensure_local_bin_in_path()?;

    // Escribir service
    let service_content = format!(
        r#"[Unit]
Description=Domain Handler Service for DuckDNS
After=network.target

[Service]
ExecStart={} start --detach=false
Restart=always
User={}
WorkingDirectory={}

[Install]
WantedBy=default.target
"#,
        bin_dest().to_string_lossy(),
        whoami::username(),
        local_bin().to_string_lossy()
    );

    fs::write(service_path(), service_content)?;
    fs::set_permissions(service_path(), fs::Permissions::from_mode(0o644))?;

    // Reload systemd user units
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    println!("✅ Service installed for current user.");
    println!("➡️  Run `source ~/.bashrc` or restart your terminal.");
    println!("➡️  Enable service: `systemctl --user enable --now domainhdlr.service`");

    Ok(())
}

// === Uninstall ===

pub fn uninstall_service() -> anyhow::Result<()> {
    let _ = fs::remove_file(service_path());
    let _ = fs::remove_file(bin_dest());
    let _ = fs::remove_file(config_dest());

    if fs::read_dir(config_dir()).map_or(false, |mut d| d.next().is_none()) {
        let _ = fs::remove_dir(config_dir());
    }

    if fs::read_dir(local_bin()).map_or(false, |mut d| d.next().is_none()) {
        let _ = fs::remove_dir(local_bin());
    }

    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    println!("🗑️ Service uninstalled.");
    Ok(())
}

// === Enable/Disable on boot ===

pub fn set_enable_on_boot(enable: bool) -> anyhow::Result<()> {
    let action = if enable { "enable" } else { "disable" };
    let status = Command::new("systemctl")
        .args(["--user", action, "domainhdlr.service"])
        .status()?;

    if status.success() {
        println!("🔁 Service {}d on user boot.", action);
    } else {
        eprintln!("❌ Failed to {} service.", action);
    }

    Ok(())
}
