use std::fs;
use std::io::{Write, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use crate::process::rutas::{bin_dir, bin_path, config_dir, config_file, service_path, systemd_user_dir};

pub fn install_service() -> anyhow::Result<()> {
    fs::create_dir_all(bin_dir())?;
    fs::create_dir_all(config_dir())?;
    fs::create_dir_all(systemd_user_dir())?;

    // Copiar binario
    fs::copy("domainhdlr", &bin_path())?;
    fs::set_permissions(&bin_path(), fs::Permissions::from_mode(0o755))?;

    // Copiar config si existe
    if Path::new("domainhdlr.json").exists() {
        fs::copy("domainhdlr.json", &config_file())?;
        fs::set_permissions(&config_file(), fs::Permissions::from_mode(0o644))?;
    }

    let bashrc_path = dirs::home_dir().unwrap().join(".bashrc");
    let export_line = r#"export PATH="$HOME/.local/bin:$PATH""#;

    let mut added_to_bashrc = false;

    if bashrc_path.exists() {
        let content = fs::read_to_string(&bashrc_path)?;
        if !content.contains(export_line) {
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&bashrc_path)?;
            writeln!(file, "\n{}", export_line)?;
            added_to_bashrc = true;
        }
    } else {
        // Crear nuevo .bashrc con export
        let mut file = fs::File::create(&bashrc_path)?;
        writeln!(file, "{}", export_line)?;
        added_to_bashrc = true;
    }

    // Crear service file
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
        bin_path().to_string_lossy(),
        whoami::username(),
        bin_dir().to_string_lossy()
    );

    fs::write(service_path(), service_content)?;
    fs::set_permissions(service_path(), fs::Permissions::from_mode(0o644))?;

    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    println!("✅ Service installed.");

    if added_to_bashrc {
        println!("➕ Added ~/.local/bin to PATH via .bashrc.");
        println!("🔁 Please run `source ~/.bashrc` or reopen your terminal.");
        println!("👉 Or run the install via:\n   source <(domainhdlr install)\n   to apply PATH instantly.");
        println!("{}", export_line); // permite que el export funcione si se usa source <(...)>
    }

    Ok(())
}


// === Uninstall ===
pub fn uninstall_service() -> anyhow::Result<()> {
    let _ = fs::remove_file(service_path());
    let _ = fs::remove_file(bin_path());
    let _ = fs::remove_file(config_file());

    // Borrar directorios si están vacíos
    if fs::read_dir(config_dir()).map_or(false, |mut d| d.next().is_none()) {
        let _ = fs::remove_dir(config_dir());
    }

    if fs::read_dir(bin_dir()).map_or(false, |mut d| d.next().is_none()) {
        let _ = fs::remove_dir(bin_dir());
    }

    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    println!("🗑️ Service uninstalled.");
    println!("If you added to PATH manually in .bashrc, you may remove the export line.");
    Ok(())
}
// === Enable/Disable on boot ===

pub fn set_enable_on_boot(enable: bool) -> anyhow::Result<()> {
    let action = if enable { "enable" } else { "disable" };
    let status = Command::new("systemctl")
        .args(["--user", action, "domainhdlr.service"])
        .status()?;

    if status.success() {
        println!("✅ Service {}d on user boot.", action);
    } else {
        eprintln!("❌ Failed to {} service.", action);
    }

    Ok(())
}
