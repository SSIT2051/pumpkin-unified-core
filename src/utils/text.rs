use pumpkin_plugin_api::text::{TextComponent, Color};
use std::collections::HashMap;

pub fn format_text(text: &str) -> TextComponent {
    let mut component = TextComponent::empty();
    let mut current = String::new();
    let mut chars = text.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '&' || c == '§' {
            if let Some(&next) = chars.peek() {
                if let Some(color) = parse_color_code(next) {
                    if !current.is_empty() {
                        component = component.add_child(TextComponent::text(current.clone()).color(color));
                        current.clear();
                    }
                    chars.next();
                    continue;
                }
            }
        }
        current.push(c);
    }
    
    if !current.is_empty() {
        component = component.add_child(TextComponent::text(current));
    }
    
    component
}

fn parse_color_code(c: char) -> Option<Color> {
    match c.to_ascii_lowercase() {
        '0' => Some(Color::Black),
        '1' => Some(Color::DarkBlue),
        '2' => Some(Color::DarkGreen),
        '3' => Some(Color::DarkAqua),
        '4' => Some(Color::DarkRed),
        '5' => Some(Color::DarkPurple),
        '6' => Some(Color::Gold),
        '7' => Some(Color::Gray),
        '8' => Some(Color::DarkGray),
        '9' => Some(Color::Blue),
        'a' => Some(Color::Green),
        'b' => Some(Color::Aqua),
        'c' => Some(Color::Red),
        'd' => Some(Color::LightPurple),
        'e' => Some(Color::Yellow),
        'f' => Some(Color::White),
        'r' => Some(Color::Reset),
        'l' => None, // Bold
        'o' => None, // Italic
        'n' => None, // Underline
        'm' => None, // Strikethrough
        'k' => None, // Obfuscated
        _ => None,
    }
}

pub fn replace_placeholders(text: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in placeholders {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

pub fn format_with_placeholders(text: &str, placeholders: &HashMap<String, String>) -> TextComponent {
    format_text(&replace_placeholders(text, placeholders))
}