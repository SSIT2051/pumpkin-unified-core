use pumpkin_plugin_api::text::TextComponent;

/// Formats a string with legacy `&` colour codes (EssentialsX style) into a
/// [`TextComponent`]. Pumpkin's text API parses legacy codes natively.
pub fn format_text(text: &str) -> TextComponent {
    TextComponent::from_legacy_string_with_code(text, '&')
}