use aws_sdk_sesv2::{
    config::Region,
    types::{Body, Content, Destination, EmailContent, Message},
    Client, Error,
};
use color_eyre::eyre::Result;
use secrecy::ExposeSecret;

use crate::domain::{Email, EmailClient};

#[derive(Debug)]
pub struct AWSEmailClient {
    email_client: Client,
}

impl AWSEmailClient {
    pub fn new(email_client: Client) -> Self {
        Self { email_client }
    }
}

#[async_trait::async_trait]
impl EmailClient for AWSEmailClient {
    #[tracing::instrument(name = "Sending email", skip_all)]
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> Result<()> {
        let recipient_email = recipient.as_ref().expose_secret().to_owned();
        tracing::info!("Sending email to: {}", recipient_email);

        // let mut dest: Destination = Destination::builder().build();
        // dest.to_addresses = Some(vec![recipient.as_ref().expose_secret().to_owned()]);
        let dest = Destination::builder().to_addresses(recipient_email).build();

        let subject_content = Content::builder()
            .data(subject.to_string())
            .charset("UTF-8")
            .build()
            .expect("building Content");

        let body_content = Content::builder()
            .data(content.to_string())
            .charset("UTF-8")
            .build()
            .expect("building Content");
        let body = Body::builder().text(body_content).build();
        let msg = Message::builder()
            .subject(subject_content)
            .body(body)
            .build();

        let email_content = EmailContent::builder().simple(msg).build();
        tracing::info!("Email content prepared: {:?}", email_content);

        self.email_client
            .send_email()
            .from_email_address_identity_arn(
                "arn:aws:ses:us-west-1:043152383660:identity/rustyauth.com",
            )
            .from_email_address("auth@rustyauth.com")
            .content(email_content)
            .destination(dest)
            .send()
            .await?;
        Ok(())
    }
}
