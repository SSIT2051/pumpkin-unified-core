//! Chat module: private messaging, mail, and broadcasts.

use crate::config::ChatConfig;
use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub mod commands;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Mail {
    pub sender: String,
    pub recipient: String,
    pub message: String,
    pub timestamp: i64,
    pub read: bool,
}

pub struct Chat {
    storage: Arc<Storage>,
    config: ChatConfig,
    reply_targets: Mutex<HashMap<String, String>>,
}

impl Chat {
    pub fn new(storage: Arc<Storage>, config: ChatConfig) -> Self {
        Self {
            storage,
            config,
            reply_targets: Mutex::new(HashMap::new()),
        }
    }

    pub fn set_reply(&self, from: &str, to: &str) {
        self.reply_targets
            .lock()
            .unwrap()
            .insert(from.to_string(), to.to_string());
    }

    pub fn get_reply(&self, from: &str) -> Option<String> {
        self.reply_targets.lock().unwrap().get(from).cloned()
    }

    fn mail_key(recipient: &str) -> String {
        format!("mail:{recipient}")
    }

    pub fn send_mail(&self, sender: &str, recipient: &str, message: &str) {
        let mail = Mail {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            message: message.to_string(),
            timestamp: crate::utils::time::now_timestamp(),
            read: false,
        };
        let key = Self::mail_key(recipient);
        let mut mails: Vec<Mail> = self.storage.get("chat", &key).unwrap_or_default();
        mails.push(mail);
        self.storage.set("chat", &key, &mails);
    }

    pub fn read_mail(&self, recipient: &str) -> Vec<Mail> {
        let key = Self::mail_key(recipient);
        let mut mails: Vec<Mail> = self.storage.get("chat", &key).unwrap_or_default();
        for m in &mut mails {
            m.read = true;
        }
        self.storage.set("chat", &key, &mails);
        mails
    }

    pub fn format(&self) -> &str {
        &self.config.format
    }

    pub fn broadcast_format(&self) -> &str {
        &self.config.broadcast_format
    }

    pub fn private_message_format(&self) -> &str {
        &self.config.private_message_format
    }

    pub fn anti_spam_enabled(&self) -> bool {
        self.config.anti_spam
    }

    pub async fn is_spam(&self, uuid: &str, message: &str) -> bool {
        // Simple spam detection - could be enhanced
        if !self.config.anti_spam {
            return false;
        }
        // Basic check - could track message history
        false
    }
}