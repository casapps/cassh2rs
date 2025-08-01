use anyhow::{Result, Context};
use notify_rust::Notification;
use lettre::{
    Message, SmtpTransport, Transport,
    transport::smtp::authentication::Credentials,
};
use reqwest;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub desktop: bool,
    pub email: EmailConfig,
    pub webhooks: WebhookConfig,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub from_email: String,
    pub from_name: String,
    pub to_emails: Vec<String>,
    pub subject_prefix: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebhookConfig {
    pub slack: Option<String>,
    pub discord: Option<String>,
    pub teams: Option<String>,
    pub custom: Vec<String>,
}

pub struct NotificationManager {
    config: NotificationConfig,
    app_name: String,
}

impl NotificationManager {
    pub fn new(config: NotificationConfig, app_name: String) -> Self {
        Self { config, app_name }
    }
    
    pub async fn send(&self, title: &str, message: &str, level: NotificationLevel) -> Result<()> {
        let mut errors = Vec::new();
        
        // Send desktop notification
        if self.config.desktop {
            if let Err(e) = self.send_desktop(title, message, level) {
                errors.push(format!("Desktop: {}", e));
            }
        }
        
        // Send email
        if self.config.email.enabled {
            if let Err(e) = self.send_email(title, message, level).await {
                errors.push(format!("Email: {}", e));
            }
        }
        
        // Send to webhooks
        if let Err(e) = self.send_webhooks(title, message, level).await {
            errors.push(format!("Webhooks: {}", e));
        }
        
        if !errors.is_empty() {
            anyhow::bail!("Some notifications failed: {}", errors.join(", "));
        }
        
        Ok(())
    }
    
    fn send_desktop(&self, title: &str, message: &str, level: NotificationLevel) -> Result<()> {
        let mut notification = Notification::new();
        notification
            .appname(&self.app_name)
            .summary(title)
            .body(message);
        
        // Set icon based on level
        match level {
            NotificationLevel::Info => notification.icon("dialog-information"),
            NotificationLevel::Success => notification.icon("dialog-positive"),
            NotificationLevel::Warning => notification.icon("dialog-warning"),
            NotificationLevel::Error => notification.icon("dialog-error"),
        };
        
        notification.show()
            .context("Failed to show desktop notification")?;
        
        Ok(())
    }
    
    async fn send_email(&self, title: &str, message: &str, level: NotificationLevel) -> Result<()> {
        let email_config = &self.config.email;
        
        // Prepare email subject
        let subject = format!(
            "{} {} - {}",
            email_config.subject_prefix.replace("{app_name}", &self.app_name),
            level.as_str(),
            title
        );
        
        // Build email content
        let body = format!(
            "Notification from {}\n\nLevel: {}\nTitle: {}\n\nMessage:\n{}",
            self.app_name,
            level.as_str(),
            title,
            message
        );
        
        // Send to each recipient
        for to_email in &email_config.to_emails {
            let email = Message::builder()
                .from(format!("{} <{}>", email_config.from_name, email_config.from_email).parse()?)
                .to(to_email.parse()?)
                .subject(&subject)
                .body(body.clone())
                .context("Failed to build email")?;
            
            // Create SMTP transport
            let mut builder = SmtpTransport::relay(&email_config.smtp_server)?;
            
            if let (Some(username), Some(password)) = (&email_config.username, &email_config.password) {
                let creds = Credentials::new(username.clone(), password.clone());
                builder = builder.credentials(creds);
            }
            
            let mailer = builder
                .port(email_config.smtp_port)
                .build();
            
            mailer.send(&email)
                .context("Failed to send email")?;
        }
        
        Ok(())
    }
    
    async fn send_webhooks(&self, title: &str, message: &str, level: NotificationLevel) -> Result<()> {
        let webhook_config = &self.config.webhooks;
        let client = reqwest::Client::new();
        
        // Send to Slack
        if let Some(slack_url) = &webhook_config.slack {
            let payload = json!({
                "text": format!("{} - {}", title, message),
                "attachments": [{
                    "color": level.slack_color(),
                    "fields": [
                        {
                            "title": "Application",
                            "value": &self.app_name,
                            "short": true
                        },
                        {
                            "title": "Level",
                            "value": level.as_str(),
                            "short": true
                        }
                    ]
                }]
            });
            
            client.post(slack_url)
                .json(&payload)
                .send()
                .await
                .context("Failed to send Slack notification")?;
        }
        
        // Send to Discord
        if let Some(discord_url) = &webhook_config.discord {
            let payload = json!({
                "username": &self.app_name,
                "embeds": [{
                    "title": title,
                    "description": message,
                    "color": level.discord_color(),
                    "fields": [
                        {
                            "name": "Level",
                            "value": level.as_str(),
                            "inline": true
                        }
                    ]
                }]
            });
            
            client.post(discord_url)
                .json(&payload)
                .send()
                .await
                .context("Failed to send Discord notification")?;
        }
        
        // Send to Teams
        if let Some(teams_url) = &webhook_config.teams {
            let payload = json!({
                "@type": "MessageCard",
                "@context": "https://schema.org/extensions",
                "summary": title,
                "themeColor": level.teams_color(),
                "sections": [{
                    "activityTitle": title,
                    "activitySubtitle": &self.app_name,
                    "text": message,
                    "facts": [{
                        "name": "Level",
                        "value": level.as_str()
                    }]
                }]
            });
            
            client.post(teams_url)
                .json(&payload)
                .send()
                .await
                .context("Failed to send Teams notification")?;
        }
        
        // Send to custom webhooks
        for custom_url in &webhook_config.custom {
            let payload = json!({
                "app": &self.app_name,
                "title": title,
                "message": message,
                "level": level.as_str(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            client.post(custom_url)
                .json(&payload)
                .send()
                .await
                .context("Failed to send custom webhook notification")?;
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
    
    pub fn slack_color(&self) -> &'static str {
        match self {
            Self::Info => "#36a64f",
            Self::Success => "good",
            Self::Warning => "warning",
            Self::Error => "danger",
        }
    }
    
    pub fn discord_color(&self) -> u32 {
        match self {
            Self::Info => 0x3498db,    // Blue
            Self::Success => 0x2ecc71, // Green
            Self::Warning => 0xf39c12, // Orange
            Self::Error => 0xe74c3c,   // Red
        }
    }
    
    pub fn teams_color(&self) -> &'static str {
        match self {
            Self::Info => "0078D4",    // Blue
            Self::Success => "00CC00", // Green
            Self::Warning => "FF8800", // Orange
            Self::Error => "CC0000",   // Red
        }
    }
}

// Helper function to expand environment variables in strings
pub fn expand_env_vars(s: &str) -> String {
    let mut result = s.to_string();
    
    // Simple environment variable expansion
    for (key, value) in std::env::vars() {
        result = result.replace(&format!("${}", key), &value);
        result = result.replace(&format!("${{{}}}", key), &value);
    }
    
    // Handle special variables
    if result.contains("$HOSTNAME") {
        if let Ok(hostname) = hostname::get() {
            let hostname_str = hostname.to_string_lossy();
            result = result.replace("$HOSTNAME", &hostname_str);
        }
    }
    
    if result.contains("$USER") {
        if let Ok(user) = std::env::var("USER") {
            result = result.replace("$USER", &user);
        }
    }
    
    result
}

// Function to be used in generated code
pub fn send_notification_sync(
    config: &NotificationConfig,
    app_name: &str,
    title: &str,
    message: &str,
    level: NotificationLevel,
) -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    let manager = NotificationManager::new(config.clone(), app_name.to_string());
    
    runtime.block_on(async {
        manager.send(title, message, level).await
    })
}