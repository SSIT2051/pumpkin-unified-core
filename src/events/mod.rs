//! Event handlers for PumpkinMC events.
//!
//! This is a placeholder implementation. The pumpkin-plugin-api 0.2.1 doesn't
//! yet expose the event system properly. Event handlers will be implemented
//! when the API stabilizes.
//!
//! For now, the plugin works without events. Features that require events
//! (auto-ban enforcement, chat formatting, /back, AFK detection, etc.)
//! will be added when the API supports event registration.

use crate::config::EventsConfig;
use std::sync::Arc;

/// Placeholder event module that compiles but does nothing.
/// Will be replaced with real event handlers when the API supports it.
pub struct EventModule {
    _economy: Arc<crate::economy::Economy>,
    _warps: Arc<crate::warps::Warps>,
    _homes: Arc<crate::homes::Homes>,
    _tpa: Arc<crate::tpa::Tpa>,
    _kits: Arc<crate::kits::Kits>,
    _moderation: Arc<crate::moderation::Moderation>,
    _permissions: Arc<crate::permissions::Permissions>,
    _spawn: Arc<crate::spawn::SpawnModule>,
    _utility: Arc<crate::utility::Utility>,
    _chat: Arc<crate::chat::Chat>,
    _events_config: crate::config::EventsConfig,
}

impl EventModule {
    pub fn new(
        economy: Arc<crate::economy::Economy>,
        warps: Arc<crate::warps::Warps>,
        homes: Arc<crate::homes::Homes>,
        tpa: Arc<crate::tpa::Tpa>,
        kits: Arc<crate::kits::Kits>,
        moderation: Arc<crate::moderation::Moderation>,
        permissions: Arc<crate::permissions::Permissions>,
        spawn: Arc<crate::spawn::SpawnModule>,
        utility: Arc<crate::utility::Utility>,
        chat: Arc<crate::chat::Chat>,
        events_config: crate::config::EventsConfig,
    ) -> Self {
        Self {
            _economy: economy,
            _warps: warps,
            _homes: homes,
            _tpa: tpa,
            _kits: kits,
            _moderation: moderation,
            _permissions: permissions,
            _spawn: spawn,
            _utility: utility,
            _chat: chat,
            _events_config: events_config,
        }
    }

    pub fn register_handlers(&self, _context: &pumpkin_plugin_api::Context) {
        tracing::info!("Event system placeholder registered (no-op until API supports events)");
    }
}

impl Clone for EventModule {
    fn clone(&self) -> Self {
        Self {
            _economy: self._economy.clone(),
            _warps: self._warps.clone(),
            _homes: self._homes.clone(),
            _tpa: self._tpa.clone(),
            _kits: self._kits.clone(),
            _moderation: self._moderation.clone(),
            _permissions: self._permissions.clone(),
            _spawn: self._spawn.clone(),
            _utility: self._utility.clone(),
            _chat: self._chat.clone(),
            _events_config: self._events_config.clone(),
        }
    }
}