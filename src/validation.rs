use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::constants::{
    MAX_CAROUSEL_ITEMS, MAX_LINKS, MAX_POSTS_PER_REQUEST, MAX_TEXT_ATTACHMENT_LENGTH,
    MAX_TEXT_ENTITIES, MAX_TEXT_LENGTH, MIN_CAROUSEL_ITEMS, MIN_SEARCH_TIMESTAMP,
};
use crate::error::new_validation_error;
use crate::types::{GifAttachment, PaginationOptions, SearchOptions, TextAttachment, TextEntity};

/// Regex matching HTTP(S) URLs.
static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s)<>\]\}]+").unwrap());

/// Validate that an optional limit does not exceed `MAX_POSTS_PER_REQUEST`.
///
/// Works with any options type — just pass `opts.limit` directly.
pub fn validate_limit(limit: Option<usize>) -> crate::Result<()> {
    if let Some(limit) = limit {
        if limit > MAX_POSTS_PER_REQUEST {
            return Err(new_validation_error(
                400,
                &format!("limit {limit} exceeds maximum of {MAX_POSTS_PER_REQUEST}"),
                "limit too large",
                "limit",
            ));
        }
    }
    Ok(())
}

/// Validate that `text` does not exceed `MAX_TEXT_LENGTH` unicode characters.
pub fn validate_text_length(text: &str, field_name: &str) -> crate::Result<()> {
    let count = text.chars().count();
    if count > MAX_TEXT_LENGTH {
        return Err(new_validation_error(
            400,
            &format!(
                "{field_name} exceeds maximum length of {MAX_TEXT_LENGTH} characters (got {count})"
            ),
            "text too long",
            field_name,
        ));
    }
    Ok(())
}

/// Validate that the combined unique link count from `text` and
/// `link_attachment_url` does not exceed `MAX_LINKS`.
pub fn validate_link_count(text: &str, link_attachment_url: &str) -> crate::Result<()> {
    let mut unique: HashSet<&str> = HashSet::new();

    for m in URL_REGEX.find_iter(text) {
        unique.insert(m.as_str());
    }

    if !link_attachment_url.is_empty() {
        unique.insert(link_attachment_url);
    }

    if unique.len() > MAX_LINKS {
        return Err(new_validation_error(
            400,
            &format!(
                "post contains {} unique links, maximum allowed is {MAX_LINKS}",
                unique.len()
            ),
            "too many links",
            "text",
        ));
    }
    Ok(())
}

/// Validate a text attachment: plaintext length and styling ranges.
pub fn validate_text_attachment(attachment: &TextAttachment) -> crate::Result<()> {
    let char_count = attachment.plaintext.chars().count();
    if char_count > MAX_TEXT_ATTACHMENT_LENGTH {
        return Err(new_validation_error(
            400,
            &format!(
                "text attachment plaintext exceeds maximum length of {MAX_TEXT_ATTACHMENT_LENGTH} characters (got {char_count})"
            ),
            "text attachment too long",
            "text_attachment.plaintext",
        ));
    }

    if let Some(ref styles) = attachment.text_with_styling_info {
        for (i, info) in styles.iter().enumerate() {
            let end = info.offset.saturating_add(info.length);
            if end > char_count {
                return Err(new_validation_error(
                    400,
                    &format!(
                        "text_with_styling_info[{i}] range ({offset}..{end}) exceeds plaintext length ({char_count})",
                        offset = info.offset,
                    ),
                    "styling range out of bounds",
                    "text_attachment.text_with_styling_info",
                ));
            }
        }
    }

    Ok(())
}

/// Validate text entities: at most `MAX_TEXT_ENTITIES`, each must be SPOILER
/// type, and offsets must be non-negative (enforced by `usize`).
pub fn validate_text_entities(
    entities: &[TextEntity],
    text_char_count: usize,
) -> crate::Result<()> {
    if entities.len() > MAX_TEXT_ENTITIES {
        return Err(new_validation_error(
            400,
            &format!(
                "too many text entities: got {}, maximum is {MAX_TEXT_ENTITIES}",
                entities.len()
            ),
            "too many text entities",
            "text_entities",
        ));
    }

    for (i, entity) in entities.iter().enumerate() {
        if entity.entity_type != "SPOILER" {
            return Err(new_validation_error(
                400,
                &format!(
                    "text_entities[{i}] has unsupported entity_type '{}', only 'SPOILER' is allowed",
                    entity.entity_type,
                ),
                "invalid entity type",
                "text_entities",
            ));
        }

        if entity.length == 0 {
            return Err(new_validation_error(
                400,
                &format!("text_entities[{i}] has zero length"),
                "invalid entity length",
                "text_entities",
            ));
        }

        let end = entity.offset.saturating_add(entity.length);
        if end > text_char_count {
            return Err(new_validation_error(
                400,
                &format!(
                    "text_entities[{i}] range ({}..{end}) exceeds text length ({text_char_count})",
                    entity.offset,
                ),
                "entity range out of bounds",
                "text_entities",
            ));
        }
    }

    Ok(())
}

/// Validate a media URL: must be non-empty and start with `http://` or `https://`.
pub fn validate_media_url(url: &str, media_type: &str) -> crate::Result<()> {
    if url.is_empty() {
        return Err(new_validation_error(
            400,
            &format!("{media_type} URL is required"),
            "empty media url",
            &format!("{media_type}_url"),
        ));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(new_validation_error(
            400,
            &format!("{media_type} URL must start with http:// or https://"),
            "invalid media url scheme",
            &format!("{media_type}_url"),
        ));
    }

    Ok(())
}

/// Validate a topic tag: must not contain periods (`.`) or ampersands (`&`).
pub fn validate_topic_tag(tag: &str) -> crate::Result<()> {
    if tag.contains('.') {
        return Err(new_validation_error(
            400,
            "topic tag must not contain periods",
            "invalid character in topic tag",
            "topic_tag",
        ));
    }
    if tag.contains('&') {
        return Err(new_validation_error(
            400,
            "topic tag must not contain ampersands",
            "invalid character in topic tag",
            "topic_tag",
        ));
    }
    Ok(())
}

/// Validate country codes: each must be exactly 2 alphabetic ASCII characters.
pub fn validate_country_codes(codes: &[String]) -> crate::Result<()> {
    for (i, code) in codes.iter().enumerate() {
        if code.len() != 2 {
            return Err(new_validation_error(
                400,
                &format!("allowlisted_country_codes[{i}] '{code}' must be exactly 2 characters"),
                "invalid country code length",
                "allowlisted_country_codes",
            ));
        }
        if !code.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(new_validation_error(
                400,
                &format!(
                    "allowlisted_country_codes[{i}] '{code}' must contain only alphabetic characters"
                ),
                "invalid country code characters",
                "allowlisted_country_codes",
            ));
        }
    }
    Ok(())
}

/// Validate carousel child count is between `MIN_CAROUSEL_ITEMS` and
/// `MAX_CAROUSEL_ITEMS` inclusive.
pub fn validate_carousel_children(count: usize) -> crate::Result<()> {
    if count < MIN_CAROUSEL_ITEMS {
        return Err(new_validation_error(
            400,
            &format!("carousel requires at least {MIN_CAROUSEL_ITEMS} items, got {count}"),
            "too few carousel items",
            "children",
        ));
    }
    if count > MAX_CAROUSEL_ITEMS {
        return Err(new_validation_error(
            400,
            &format!("carousel allows at most {MAX_CAROUSEL_ITEMS} items, got {count}"),
            "too many carousel items",
            "children",
        ));
    }
    Ok(())
}

/// Validate pagination options: limit must not exceed `MAX_POSTS_PER_REQUEST`.
pub fn validate_pagination_options(opts: &PaginationOptions) -> crate::Result<()> {
    if let Some(limit) = opts.limit {
        if limit > MAX_POSTS_PER_REQUEST {
            return Err(new_validation_error(
                400,
                &format!("limit {limit} exceeds maximum of {MAX_POSTS_PER_REQUEST}"),
                "limit too large",
                "limit",
            ));
        }
    }
    Ok(())
}

/// Validate search options: limit and since timestamp constraints.
pub fn validate_search_options(opts: &SearchOptions) -> crate::Result<()> {
    if let Some(limit) = opts.limit {
        if limit > MAX_POSTS_PER_REQUEST {
            return Err(new_validation_error(
                400,
                &format!("limit {limit} exceeds maximum of {MAX_POSTS_PER_REQUEST}"),
                "limit too large",
                "limit",
            ));
        }
    }

    if let Some(since) = opts.since {
        if since < MIN_SEARCH_TIMESTAMP {
            return Err(new_validation_error(
                400,
                &format!(
                    "since timestamp {since} is before the minimum allowed ({MIN_SEARCH_TIMESTAMP})"
                ),
                "since timestamp too early",
                "since",
            ));
        }
    }

    Ok(())
}

/// Validate a GIF attachment: `gif_id` must be non-empty and `provider` must
/// be a known variant (enforced at the type level by `GifProvider`).
pub fn validate_gif_attachment(attachment: &GifAttachment) -> crate::Result<()> {
    if attachment.gif_id.is_empty() {
        return Err(new_validation_error(
            400,
            "gif_id is required",
            "empty gif_id",
            "gif_attachment.gif_id",
        ));
    }
    // `provider` is a `GifProvider` enum — only valid variants are representable,
    // so no runtime check is needed.
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GifProvider, TextStylingInfo};

    // --- validate_text_length ---

    #[test]
    fn text_length_ok() {
        assert!(validate_text_length("hello", "text").is_ok());
    }

    #[test]
    fn text_length_exact_limit() {
        let s: String = "a".repeat(MAX_TEXT_LENGTH);
        assert!(validate_text_length(&s, "text").is_ok());
    }

    #[test]
    fn text_length_exceeds() {
        let s: String = "a".repeat(MAX_TEXT_LENGTH + 1);
        let err = validate_text_length(&s, "text").unwrap_err();
        assert!(err.is_validation());
    }

    #[test]
    fn text_length_unicode() {
        // Each emoji is one char but multiple bytes.
        let s: String = "\u{1F600}".repeat(MAX_TEXT_LENGTH);
        assert!(validate_text_length(&s, "text").is_ok());

        let s2: String = "\u{1F600}".repeat(MAX_TEXT_LENGTH + 1);
        assert!(validate_text_length(&s2, "text").is_err());
    }

    // --- validate_link_count ---

    #[test]
    fn link_count_ok() {
        assert!(validate_link_count("check https://a.com", "").is_ok());
    }

    #[test]
    fn link_count_with_attachment() {
        let text = "https://a.com https://b.com https://c.com https://d.com";
        assert!(validate_link_count(text, "https://e.com").is_ok());
    }

    #[test]
    fn link_count_exceeds() {
        let text = "https://a.com https://b.com https://c.com https://d.com https://e.com";
        let err = validate_link_count(text, "https://f.com").unwrap_err();
        assert!(err.is_validation());
    }

    #[test]
    fn link_count_deduplicates() {
        let text = "https://a.com https://a.com https://a.com";
        assert!(validate_link_count(text, "https://a.com").is_ok());
    }

    // --- validate_text_attachment ---

    #[test]
    fn text_attachment_ok() {
        let att = TextAttachment {
            plaintext: "hello world".into(),
            link_attachment_url: None,
            text_with_styling_info: None,
        };
        assert!(validate_text_attachment(&att).is_ok());
    }

    #[test]
    fn text_attachment_too_long() {
        let att = TextAttachment {
            plaintext: "x".repeat(MAX_TEXT_ATTACHMENT_LENGTH + 1),
            link_attachment_url: None,
            text_with_styling_info: None,
        };
        assert!(validate_text_attachment(&att).is_err());
    }

    #[test]
    fn text_attachment_styling_in_range() {
        let att = TextAttachment {
            plaintext: "hello".into(),
            link_attachment_url: None,
            text_with_styling_info: Some(vec![TextStylingInfo {
                offset: 0,
                length: 5,
                styling_info: vec!["BOLD".into()],
            }]),
        };
        assert!(validate_text_attachment(&att).is_ok());
    }

    #[test]
    fn text_attachment_styling_out_of_range() {
        let att = TextAttachment {
            plaintext: "hello".into(),
            link_attachment_url: None,
            text_with_styling_info: Some(vec![TextStylingInfo {
                offset: 3,
                length: 5,
                styling_info: vec!["BOLD".into()],
            }]),
        };
        assert!(validate_text_attachment(&att).is_err());
    }

    // --- validate_text_entities ---

    #[test]
    fn text_entities_ok() {
        let entities = vec![TextEntity {
            entity_type: "SPOILER".into(),
            offset: 0,
            length: 5,
        }];
        assert!(validate_text_entities(&entities, 100).is_ok());
    }

    #[test]
    fn text_entities_empty_ok() {
        assert!(validate_text_entities(&[], 0).is_ok());
    }

    #[test]
    fn text_entities_too_many() {
        let entities: Vec<TextEntity> = (0..MAX_TEXT_ENTITIES + 1)
            .map(|i| TextEntity {
                entity_type: "SPOILER".into(),
                offset: i,
                length: 1,
            })
            .collect();
        assert!(validate_text_entities(&entities, 100).is_err());
    }

    #[test]
    fn text_entities_wrong_type() {
        let entities = vec![TextEntity {
            entity_type: "LINK".into(),
            offset: 0,
            length: 5,
        }];
        assert!(validate_text_entities(&entities, 100).is_err());
    }

    #[test]
    fn text_entities_zero_length() {
        let entities = vec![TextEntity {
            entity_type: "SPOILER".into(),
            offset: 0,
            length: 0,
        }];
        assert!(validate_text_entities(&entities, 100).is_err());
    }

    #[test]
    fn text_entities_out_of_bounds() {
        let entities = vec![TextEntity {
            entity_type: "SPOILER".into(),
            offset: 8,
            length: 5,
        }];
        assert!(validate_text_entities(&entities, 10).is_err());
    }

    // --- validate_media_url ---

    #[test]
    fn media_url_ok_https() {
        assert!(validate_media_url("https://example.com/img.jpg", "image").is_ok());
    }

    #[test]
    fn media_url_ok_http() {
        assert!(validate_media_url("http://example.com/img.jpg", "image").is_ok());
    }

    #[test]
    fn media_url_empty() {
        assert!(validate_media_url("", "image").is_err());
    }

    #[test]
    fn media_url_bad_scheme() {
        assert!(validate_media_url("ftp://example.com/img.jpg", "image").is_err());
    }

    // --- validate_topic_tag ---

    #[test]
    fn topic_tag_ok() {
        assert!(validate_topic_tag("rustlang").is_ok());
    }

    #[test]
    fn topic_tag_period() {
        assert!(validate_topic_tag("rust.lang").is_err());
    }

    #[test]
    fn topic_tag_ampersand() {
        assert!(validate_topic_tag("rust&go").is_err());
    }

    // --- validate_country_codes ---

    #[test]
    fn country_codes_ok() {
        let codes = vec!["US".into(), "GB".into(), "DE".into()];
        assert!(validate_country_codes(&codes).is_ok());
    }

    #[test]
    fn country_codes_wrong_length() {
        let codes = vec!["USA".into()];
        assert!(validate_country_codes(&codes).is_err());
    }

    #[test]
    fn country_codes_non_alpha() {
        let codes = vec!["U1".into()];
        assert!(validate_country_codes(&codes).is_err());
    }

    #[test]
    fn country_codes_empty_list() {
        assert!(validate_country_codes(&[]).is_ok());
    }

    // --- validate_carousel_children ---

    #[test]
    fn carousel_ok() {
        assert!(validate_carousel_children(5).is_ok());
    }

    #[test]
    fn carousel_min_boundary() {
        assert!(validate_carousel_children(MIN_CAROUSEL_ITEMS).is_ok());
    }

    #[test]
    fn carousel_max_boundary() {
        assert!(validate_carousel_children(MAX_CAROUSEL_ITEMS).is_ok());
    }

    #[test]
    fn carousel_too_few() {
        assert!(validate_carousel_children(1).is_err());
    }

    #[test]
    fn carousel_too_many() {
        assert!(validate_carousel_children(MAX_CAROUSEL_ITEMS + 1).is_err());
    }

    // --- validate_pagination_options ---

    #[test]
    fn pagination_ok() {
        let opts = PaginationOptions {
            limit: Some(50),
            ..Default::default()
        };
        assert!(validate_pagination_options(&opts).is_ok());
    }

    #[test]
    fn pagination_no_limit() {
        let opts = PaginationOptions::default();
        assert!(validate_pagination_options(&opts).is_ok());
    }

    #[test]
    fn pagination_exceeds() {
        let opts = PaginationOptions {
            limit: Some(MAX_POSTS_PER_REQUEST + 1),
            ..Default::default()
        };
        assert!(validate_pagination_options(&opts).is_err());
    }

    // --- validate_search_options ---

    #[test]
    fn search_ok() {
        let opts = SearchOptions {
            limit: Some(25),
            since: Some(MIN_SEARCH_TIMESTAMP + 100),
            ..Default::default()
        };
        assert!(validate_search_options(&opts).is_ok());
    }

    #[test]
    fn search_limit_exceeds() {
        let opts = SearchOptions {
            limit: Some(MAX_POSTS_PER_REQUEST + 1),
            ..Default::default()
        };
        assert!(validate_search_options(&opts).is_err());
    }

    #[test]
    fn search_since_too_early() {
        let opts = SearchOptions {
            since: Some(MIN_SEARCH_TIMESTAMP - 1),
            ..Default::default()
        };
        assert!(validate_search_options(&opts).is_err());
    }

    #[test]
    fn search_since_exact_boundary() {
        let opts = SearchOptions {
            since: Some(MIN_SEARCH_TIMESTAMP),
            ..Default::default()
        };
        assert!(validate_search_options(&opts).is_ok());
    }

    #[test]
    fn search_defaults_ok() {
        let opts = SearchOptions::default();
        assert!(validate_search_options(&opts).is_ok());
    }

    // --- validate_gif_attachment ---

    #[test]
    fn gif_attachment_ok() {
        let att = GifAttachment {
            gif_id: "abc123".into(),
            provider: GifProvider::Giphy,
        };
        assert!(validate_gif_attachment(&att).is_ok());
    }

    #[test]
    fn gif_attachment_tenor_ok() {
        let att = GifAttachment {
            gif_id: "xyz".into(),
            provider: GifProvider::Tenor,
        };
        assert!(validate_gif_attachment(&att).is_ok());
    }

    #[test]
    fn gif_attachment_empty_id() {
        let att = GifAttachment {
            gif_id: "".into(),
            provider: GifProvider::Giphy,
        };
        assert!(validate_gif_attachment(&att).is_err());
    }
}
