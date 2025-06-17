//! Email notification functionality for test results
//! 
//! This module provides email sending capabilities for test reports,
//! supporting both HTML and plain text formats with optional attachments.

#[cfg(feature = "email")]
use anyhow::{anyhow, Result};
#[cfg(feature = "email")]
use lettre::{
    message::{header::ContentType, Attachment, Mailbox, Message, MultiPart, SinglePart},
    transport::smtp::{authentication::Credentials, PoolConfig},
    SmtpTransport, Transport,
};
#[cfg(feature = "email")]
use log::{info, warn, error};
#[cfg(feature = "email")]
use std::path::Path;

#[cfg(feature = "email")]
use crate::cli::EmailConfig;
#[cfg(feature = "email")]
use crate::report::TestSuiteResult;

/// Email sender for test reports
#[cfg(feature = "email")]
pub struct MailSender {
    transport: SmtpTransport,
    from: Mailbox,
    config: EmailConfig,
}

#[cfg(feature = "email")]
impl MailSender {
    /// Create a new mail sender with the given configuration
    pub fn new(config: EmailConfig) -> Result<Self> {
        let credentials = Credentials::new(config.username.clone(), config.password.clone());
        
        let mut transport_builder = SmtpTransport::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(credentials);

        // Configure TLS
        if config.enable_tls {
            transport_builder = transport_builder.tls(lettre::transport::smtp::client::Tls::Required(
                lettre::transport::smtp::client::TlsParameters::new(config.smtp_host.clone())?
            ));
        }

        let transport = transport_builder
            .pool_config(PoolConfig::new().max_size(5))
            .build();

        let from: Mailbox = if config.from.contains('@') {
            config.from.parse()?
        } else {
            format!("{} <{}>", config.from, config.username).parse()?
        };

        Ok(Self {
            transport,
            from,
            config,
        })
    }

    /// Send test report email with both HTML and plain text versions
    pub fn send_test_report(
        &self,
        suite_result: &TestSuiteResult,
        plain_text_body: &str,
        html_body: &str,
        xunit_file_path: Option<&Path>,
    ) -> Result<()> {
        info!("Preparing to send test report email to {} recipients", self.config.to.len());

        // Create email parts
        let plain_part = SinglePart::builder()
            .header(ContentType::TEXT_PLAIN)
            .body(plain_text_body.to_string());

        let html_part = SinglePart::builder()
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string());

        // Create multipart alternative (HTML + plain text)
        let mut multipart = MultiPart::alternative()
            .singlepart(plain_part)
            .singlepart(html_part);

        // Add attachment if requested and file exists
        if self.config.attach_xunit {
            if let Some(xunit_path) = xunit_file_path {
                if xunit_path.exists() {
                    match self.create_attachment(xunit_path) {
                        Ok(attachment) => {
                            multipart = MultiPart::mixed()
                                .multipart(multipart)
                                .singlepart(attachment);
                            info!("Added JUnit XML attachment: {}", xunit_path.display());
                        }
                        Err(e) => {
                            warn!("Failed to create attachment: {}", e);
                        }
                    }
                } else {
                    warn!("XUnit file not found: {}", xunit_path.display());
                }
            }
        }

        // Send to each recipient
        let mut success_count = 0;
        let mut error_count = 0;

        for recipient in &self.config.to {
            match self.send_to_recipient(recipient, &multipart, suite_result) {
                Ok(_) => {
                    success_count += 1;
                    info!("✓ Email sent successfully to: {}", recipient);
                }
                Err(e) => {
                    error_count += 1;
                    error!("✗ Failed to send email to {}: {}", recipient, e);
                }
            }
        }

        if error_count > 0 {
            warn!("Email sending completed with {} successes and {} failures", success_count, error_count);
        } else {
            info!("📧 All emails sent successfully to {} recipients", success_count);
        }

        Ok(())
    }

    /// Send email to a single recipient
    fn send_to_recipient(
        &self,
        recipient: &str,
        multipart: &MultiPart,
        suite_result: &TestSuiteResult,
    ) -> Result<()> {
        let to: Mailbox = recipient.parse()
            .map_err(|e| anyhow!("Invalid recipient email address '{}': {}", recipient, e))?;

        let subject = format!(
            "{} - {} Tests ({} Passed, {} Failed)",
            self.config.subject,
            suite_result.total_tests(),
            suite_result.passed_tests(),
            suite_result.failed_tests()
        );

        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject)
            .multipart(multipart.clone())?;

        self.transport.send(&email)?;
        Ok(())
    }

    /// Create email attachment from file
    fn create_attachment(&self, file_path: &Path) -> Result<SinglePart> {
        let file_content = std::fs::read(file_path)?;
        let filename = file_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid file path"))?
            .to_string_lossy()
            .to_string();

        let content_type = if filename.ends_with(".xml") {
            ContentType::parse("application/xml")?
        } else {
            ContentType::parse("application/octet-stream")?
        };

        Ok(Attachment::new(filename).body(file_content, content_type))
    }
}

/// Stub implementation when email feature is disabled
#[cfg(not(feature = "email"))]
pub struct MailSender;

#[cfg(not(feature = "email"))]
impl MailSender {
    pub fn new(_config: crate::cli::EmailConfig) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!("Email feature is not enabled. Please compile with --features email"))
    }

    pub fn send_test_report(
        &self,
        _suite_result: &crate::report::TestSuiteResult,
        _plain_text_body: &str,
        _html_body: &str,
        _xunit_file_path: Option<&std::path::Path>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Email feature is not enabled"))
    }
}
