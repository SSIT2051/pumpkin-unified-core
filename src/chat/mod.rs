//! Chat module: private messaging, mail, and broadcasts.

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
    reply_targets: Mutex<HashMap<String, String>>,
}

impl Chat {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self {
            storage,
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
}