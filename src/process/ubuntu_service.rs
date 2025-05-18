use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::process::rutas::{
    bin_dir, bin_path, config_dir, config_file, service_path, systemd_user_dir,
};

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

    // Agregar ~/.local/bin al PATH
    let bashrc_path = dirs::home_dir().unwrap().join(".bashrc");
    let export_line = r#"export PATH="$HOME/.local/bin:$PATH""#;
    let mut added_to_bashrc = false;

    if bashrc_path.exists() {
        let content = fs::read_to_string(&bashrc_path)?;
        if !content.contains(export_line) {
            let mut file = fs::OpenOptions::new().append(true).open(&bashrc_path)?;
            writeln!(file, "\n{}", export_line)?;
            added_to_bashrc = true;
        }
    } else {
        let mut file = fs::File::create(&bashrc_path)?;
        writeln!(file, "{}", export_line)?;
        added_to_bashrc = true;
    }

    // Crear archivo systemd
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
        println!("👉 Or run:\n   source <(domainhdlr install)\n   to apply instantly.");
        println!("{}", export_line);
    }

    Ok(())
}

pub fn uninstall_service() -> anyhow::Result<()> {
    let _ = fs::remove_file(service_path());
    let _ = fs::remove_file(bin_path());
    let _ = fs::remove_file(config_file());

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
    println!("ℹ️ If you manually added the PATH to .bashrc, you can remove it.");
    Ok(())
}

pub fn set_enable_on_boot(enable: bool) -> anyhow::Result<()> {
    let action = if enable { "enable" } else { "disable" };
    let output = Command::new("systemctl")
        .args(["--user", action, "domainhdlr.service"])
        .output()?;

    if output.status.success() {
        println!("✅ Service {}d on user boot.", action);
    } else {
        eprintln!("❌ Failed to {} service.", action);
        if !output.stderr.is_empty() {
            eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.stdout.is_empty() {
            eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        }
    }

    Ok(())
}
