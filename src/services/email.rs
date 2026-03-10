use crate::config::CONFIG;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Serialize)]
struct ResendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
    reply_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResendEmailResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ResendErrorResponse {
    message: String,
}

pub struct EmailService;

impl EmailService {
    /// Send an email via Resend API
    pub async fn send_email(
        to: &str,
        subject: &str,
        html_body: &str,
        reply_to: Option<&str>,
    ) -> AppResult<String> {
        if CONFIG.resend_api_key.is_empty() {
            error!("Resend API key not configured, skipping email send");
            return Err(AppError::InternalError(
                "Email service not configured".to_string(),
            ));
        }

        let client = reqwest::Client::new();

        let request = ResendEmailRequest {
            from: CONFIG.resend_from_email.clone(),
            to: vec![to.to_string()],
            subject: subject.to_string(),
            html: html_body.to_string(),
            reply_to: reply_to.map(|s| s.to_string()),
        };

        let response = client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", CONFIG.resend_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("Failed to send email: {}", e)))?;

        if response.status().is_success() {
            let result: ResendEmailResponse = response
                .json()
                .await
                .map_err(|e| AppError::InternalError(format!("Failed to parse response: {}", e)))?;

            info!("Email sent successfully, id: {}", result.id);
            Ok(result.id)
        } else {
            let error: ResendErrorResponse = response.json().await.unwrap_or(ResendErrorResponse {
                message: "Unknown error".to_string(),
            });

            error!("Failed to send email: {}", error.message);
            Err(AppError::InternalError(format!(
                "Email send failed: {}",
                error.message
            )))
        }
    }

    /// Send contact form notification to admin
    pub async fn send_contact_notification(
        sender_name: &str,
        sender_email: &str,
        subject: &str,
        message: &str,
    ) -> AppResult<String> {
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background: #0A0A0A; color: white; padding: 20px; border-radius: 8px 8px 0 0; }}
                    .content {{ background: #f9fafb; padding: 20px; border: 1px solid #e5e7eb; border-top: none; border-radius: 0 0 8px 8px; }}
                    .field {{ margin-bottom: 16px; }}
                    .label {{ font-size: 12px; color: #6b7280; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 4px; }}
                    .value {{ font-size: 16px; }}
                    .message-box {{ background: white; padding: 16px; border-radius: 8px; border: 1px solid #e5e7eb; white-space: pre-wrap; }}
                    .footer {{ margin-top: 20px; font-size: 12px; color: #6b7280; text-align: center; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h2 style="margin: 0;">New Contact Form Submission</h2>
                        <p style="margin: 8px 0 0 0; opacity: 0.8;">kadynpearce.dev</p>
                    </div>
                    <div class="content">
                        <div class="field">
                            <div class="label">From</div>
                            <div class="value">{sender_name} &lt;{sender_email}&gt;</div>
                        </div>
                        <div class="field">
                            <div class="label">Subject</div>
                            <div class="value">{subject}</div>
                        </div>
                        <div class="field">
                            <div class="label">Message</div>
                            <div class="message-box">{message}</div>
                        </div>
                    </div>
                    <div class="footer">
                        Reply directly to this email to respond to {sender_name}
                    </div>
                </div>
            </body>
            </html>
            "#,
            sender_name = html_escape(sender_name),
            sender_email = html_escape(sender_email),
            subject = html_escape(subject),
            message = html_escape(message),
        );

        Self::send_email(
            &CONFIG.contact_email,
            &format!("[Contact] {}", subject),
            &html_body,
            Some(sender_email),
        )
        .await
    }

    /// Send auto-reply to the person who submitted the contact form
    pub async fn send_contact_auto_reply(
        recipient_name: &str,
        recipient_email: &str,
    ) -> AppResult<String> {
        let html_body = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; }}
                    .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
                    .header {{ background: #0A0A0A; color: white; padding: 20px; border-radius: 8px 8px 0 0; }}
                    .content {{ background: #f9fafb; padding: 20px; border: 1px solid #e5e7eb; border-top: none; border-radius: 0 0 8px 8px; }}
                    .footer {{ margin-top: 20px; font-size: 12px; color: #6b7280; text-align: center; }}
                    a {{ color: #2563eb; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h2 style="margin: 0;">Thanks for reaching out!</h2>
                    </div>
                    <div class="content">
                        <p>Hi {recipient_name},</p>
                        <p>Thanks for getting in touch! I've received your message and will get back to you as soon as possible, usually within 24-48 hours.</p>
                        <p>In the meantime, feel free to check out my latest work on <a href="https://kadynpearce.dev/projects">my portfolio</a> or connect with me on <a href="https://github.com/kadynjaipearce">GitHub</a>.</p>
                        <p>Best,<br>Kadyn</p>
                    </div>
                    <div class="footer">
                        This is an automated response from kadynpearce.dev
                    </div>
                </div>
            </body>
            </html>
            "#,
            recipient_name = html_escape(recipient_name),
        );

        Self::send_email(
            recipient_email,
            "Thanks for reaching out!",
            &html_body,
            None,
        )
        .await
    }
}

/// Simple HTML escaping for user input
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
