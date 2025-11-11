/// Helper function to generate avatar fallback text from a name
pub fn generate_avatar_fallback(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect::<String>()
}
