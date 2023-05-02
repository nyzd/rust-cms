use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
    message::header::ContentType,
    message::SinglePart,
};
use std::error::Error;

#[derive(Clone)]
pub struct EmailManager {
    default_from: String,
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

impl EmailManager {
    /// Grab creds and create smtpTransport and return EmailManage
    pub fn new(
        host: &str,
        port: u16,
        creds: Credentials,
        default_from: String,
    ) -> Result<Self, Box<dyn Error>> {
        // Create a new smtpTransport object
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)?
            .port(port)
            .credentials(creds)
            .build();

        Ok(Self {
            default_from,
            mailer,
        })
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: String,
    ) -> Result<(), Box<dyn Error>> {
        let part = SinglePart::html(body);

        let message = Message::builder()
            .from(self.default_from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .singlepart(part)?;

        self.mailer.send(message).await?;

        Ok(())
    }
}
