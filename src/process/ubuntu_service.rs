use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

const SERVICE_PATH: &str = "/etc/systemd/system/domainhdlr.service";

pub fn install_service() -> anyhow::Result<()> {
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

    Command::new("systemctl").args(["daemon-reexec"]).status()?;
    Command::new("systemctl").args(["daemon-reload"]).status()?;

    println!("Service installed successfully.");
    Ok(())
}

pub fn uninstall_service() -> anyhow::Result<()> {
    if fs::remove_file(SERVICE_PATH).is_ok() {
        Command::new("systemctl").args(["daemon-reload"]).status()?;
        println!("Service uninstalled.");
    } else {
        println!("Service was not installed.");
    }
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
