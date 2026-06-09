use lettre::message::{header, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use std::time::Duration;

/// Configuración del remitente (podés ponerla en tu archivo de config global)
pub struct MailConfig {
    pub smtp_server: String, // ejemplo: "smtp.gmail.com"
    pub smtp_port: u16,      // 587 para STARTTLS
    pub sender: String,      // correo origen
    pub password: String,    // clave de aplicación (no tu password real)
    pub recipient: String,   // correo destino
}

/// Envía un correo asincrónicamente usando Gmail y TLS.
/// Devuelve `Ok(())` si el envío fue exitoso.
pub async fn send_email_alert(
    cfg: &MailConfig,
    subject: &str,
    body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 📨 Construir el mensaje
    let email = Message::builder()
        .from(Mailbox::new(None, cfg.sender.parse()?))
        .to(Mailbox::new(None, cfg.recipient.parse()?))
        .subject(subject)
        .header(header::ContentType::TEXT_PLAIN)
        .body(body.to_string())?;

    // 🔐 Autenticación
    let creds = Credentials::new(cfg.sender.clone(), cfg.password.clone());
    let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_server)?
        .port(cfg.smtp_port)
        .credentials(creds)
        .timeout(Some(Duration::from_secs(10)))
        .build();

    // Enviar
    match mailer.send(email).await {
        Ok(_) => {
            println!("📧 Alerta enviada correctamente a {}", cfg.recipient);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Error al enviar correo: {}", e);
            Err(Box::new(e))
        }
    }
}
