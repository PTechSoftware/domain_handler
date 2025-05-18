use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

const SERVICE_PATH: &str = "/etc/systemd/system/domainhdlr.service";
const BIN_PATH: &str = "/usr/local/bin/domainhdlr";
const CONFIG_SRC: &str = "domainhdlr.json";
const CONFIG_DST_DIR: &str = "/etc/domainhdlr";
const CONFIG_DST: &str = "/etc/domainhdlr/domainhdlr.json";

pub fn install_service() -> anyhow::Result<()> {
    // Copiar ejecutable
    fs::copy("domainhdlr", BIN_PATH)?;
    fs::set_permissions(BIN_PATH, fs::Permissions::from_mode(0o755))?;

    // Copiar configuración JSON si existe
    if Path::new(CONFIG_SRC).exists() {
        fs::create_dir_all(CONFIG_DST_DIR)?;
        fs::copy(CONFIG_SRC, CONFIG_DST)?;
        fs::set_permissions(CONFIG_DST, fs::Permissions::from_mode(0o644))?;
    }

    // Escribir unit file
    let service_content = r#"[Unit]
Description=Domain Handler Service for DuckDNS
After=network.target

[Service]
ExecStart=/usr/local/bin/domainhdlr start --detach=false
Restart=always
User=root
WorkingDirectory=/usr/local/bin

[Install]
WantedBy=multi-user.target
"#;

    fs::write(SERVICE_PATH, service_content)?;
    fs::set_permissions(SERVICE_PATH, fs::Permissions::from_mode(0o644))?;

    // Recargar systemd
    Command::new("systemctl").args(["daemon-reexec"]).status()?;
    Command::new("systemctl").args(["daemon-reload"]).status()?;

    println!("Service installed successfully.");
    Ok(())
}

pub fn uninstall_service() -> anyhow::Result<()> {
    // Eliminar service
    if fs::remove_file(SERVICE_PATH).is_ok() {
        println!("Service file removed.");
    }

    // Eliminar ejecutable
    if fs::remove_file(BIN_PATH).is_ok() {
        println!("Executable removed.");
    }

    // Eliminar config JSON si existe
    if Path::new(CONFIG_DST).exists() {
        let _ = fs::remove_file(CONFIG_DST);
        println!("Configuration file removed.");
    }

    // Eliminar carpeta si está vacía
    if fs::read_dir(CONFIG_DST_DIR).map_or(false, |mut d| d.next().is_none()) {
        let _ = fs::remove_dir(CONFIG_DST_DIR);
    }

    // Recargar systemd
    Command::new("systemctl").args(["daemon-reload"]).status()?;
    println!("Service uninstalled.");
    Ok(())
}









pub fn set_enable_on_boot(enable: bool) -> anyhow::Result<()> {
    let action = if enable { "enable" } else { "disable" };
    let status = Command::new("systemctl")
        .args([action, "domainhdlr.service"])
        .status()?;

    if status.success() {
        println!(
            "Service {}d to start on boot successfully.",
            if enable { "enable" } else { "disable" }
        );
    } else {
        eprintln!("Failed to {} service at boot.", action);
    }

    Ok(())
}
