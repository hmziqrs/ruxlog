//! Server-side HTML allowlist sanitization of stored post (EditorJS) content.
//!
//! Post `content` is stored as raw EditorJS JSON. Several block `data` fields
//! are rendered by the frontends via `dangerous_inner_html` — i.e. parsed as
//! HTML, not escaped text:
//!   - `paragraph.data.text` — consumer + admin `render_paragraph_block`
//!   - `list.data.items[]`   — consumer + admin `render_list_block`
//!   - `raw.data.html`       — consumer + admin `render_raw_block`
//!
//! Because posts are admin-writable but unauthenticated-readable, an attacker
//! who can author (or compromise an account that authors) a post could
//! otherwise persist `<script>`, `onerror=`, or `javascript:` payloads that
//! execute for every reader. We neutralise those three fields at the
//! serialization chokepoint (`PostWithRelations::content`, via
//! `serialize_with`) using the `ammonia` allowlist sanitizer, which drops
//! `<script>`/event-handler attributes/`javascript:` URLs while preserving
//! benign inline markup (`<b>`, `<i>`, `<a>`, …).
//!
//! Fields rendered as *escaped text* (header, quote, code, table, captions) are
//! intentionally NOT touched here: `ammonia` would HTML-encode their literal
//! `<`/`&`, which the escaped frontend renderers would then escape a second
//! time, corrupting legitimate content. Those fields are safe precisely because
//! they are never `dangerous_inner_html` sinks. The one historical exception —
//! the TOC component rendering header text via `dangerous_inner_html` — is
//! fixed at the frontend (it renders a plain-text label). The layered CSP
//! (`script-src 'self'`, plan Phase 6c) blocks any residual inline `<script>`
//! as defence-in-depth. See plan Phase 6e.

use serde::Serialize;
use serde_json::Value;

/// Clean an HTML string through ammonia's known-safe allowlist.
///
/// `ammonia::clean` is the documented shorthand for
/// `Builder::default().clean(input).to_string()`: it strips unknown elements,
/// all `on*` event-handler attributes, and `javascript:`/`vbscript:` URLs from
/// `href`/`src`, while keeping safe inline formatting.
fn clean(html: &str) -> String {
    ammonia::clean(html)
}

/// Sanitize a top-level string field on a block's `data` object, in place.
fn clean_field(data: &mut Value, key: &str) {
    // Read the current string into an owned value so the immutable borrow ends
    // before we mutate `data`.
    let cleaned = match data.get(key).and_then(|v| v.as_str()) {
        Some(s) => clean(s),
        None => return,
    };
    if let Some(slot) = data.get_mut(key) {
        *slot = Value::String(cleaned);
    }
}

/// Sanitize `list` block items, which are an array of HTML strings.
fn clean_items(data: &mut Value) {
    let Some(arr) = data.get_mut("items").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for item in arr.iter_mut() {
        if let Some(s) = item.as_str() {
            *item = Value::String(clean(s));
        }
    }
}

/// Sanitize EditorJS post content in place: strip XSS payloads from the three
/// block fields the frontends render as `dangerous_inner_html`.
///
/// Unknown block types and non-string fields are left untouched. The `code`
/// block (`data.code`) is deliberately never handled here — it is source text
/// rendered as an escaped text node, and sanitizing it would corrupt code
/// samples containing `<`.
pub fn sanitize_editorjs_content(content: &mut Value) {
    let Some(blocks) = content.get_mut("blocks").and_then(|v| v.as_array_mut()) else {
        return;
    };
    for block in blocks.iter_mut() {
        let btype = block
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_owned();
        let Some(data) = block.get_mut("data") else {
            continue;
        };
        match btype.as_str() {
            "paragraph" => clean_field(data, "text"),
            "list" => clean_items(data),
            "raw" => clean_field(data, "html"),
            _ => {}
        }
    }
}

/// `serde` `serialize_with` adapter: clone, sanitize, then serialize.
///
/// The stored value is never mutated — sanitization happens on read, so every
/// client-facing serialization of `PostWithRelations::content` is XSS-clean
/// regardless of which handler or list endpoint produced it.
pub fn serialize_sanitized_content<S>(value: &Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut cloned = value.clone();
    sanitize_editorjs_content(&mut cloned);
    cloned.serialize(serializer)
}

/// `serde` `serialize_with` adapter for a content field stored as a JSON
/// *string* (e.g. `post_revision::Model::content`), mirroring
/// [`serialize_sanitized_content`] for the `Value` form.
///
/// The stored string is parsed to JSON, sanitized through the ammonia
/// allowlist, then re-serialized to a JSON string — so XSS payloads in the
/// `dangerous_inner_html` block fields are stripped on read while the stored
/// value is left untouched (sanitization-on-read). A malformed (non-JSON)
/// string is passed through verbatim rather than corrupted: we cannot parse it,
/// and rewriting it would risk data loss. The post-revision endpoints (autosave
/// result, revisions list) flow revision content to the client through this.
pub fn serialize_sanitized_content_string<S>(
    value: &String,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match serde_json::from_str::<Value>(value) {
        Ok(mut parsed) => {
            sanitize_editorjs_content(&mut parsed);
            // `parsed` came from a JSON string and is serializable, so this
            // cannot fail; fall back to the raw string defensively anyway.
            let cleaned = serde_json::to_string(&parsed).unwrap_or_else(|_| value.clone());
            cleaned.serialize(serializer)
        }
        // Not valid JSON — leave it untouched (do not corrupt non-JSON data).
        Err(_) => value.serialize(serializer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tag_from_paragraph() {
        let mut c = serde_json::json!({
            "blocks": [
                { "type": "paragraph", "data": { "text": "hi <script>alert(1)</script><b>bold</b>" } }
            ]
        });
        sanitize_editorjs_content(&mut c);
        let text = c["blocks"][0]["data"]["text"].as_str().unwrap();
        assert!(
            !text.contains("<script"),
            "script tag must be stripped, got: {text}"
        );
        assert!(
            text.contains("<b>bold</b>"),
            "benign inline markup must survive, got: {text}"
        );
    }

    #[test]
    fn strips_event_handler_from_list_items() {
        let mut c = serde_json::json!({
            "blocks": [
                { "type": "list", "data": { "items": [
                    "<img src=x onerror=alert(1)>",
                    "<i>safe</i>"
                ] } }
            ]
        });
        sanitize_editorjs_content(&mut c);
        let items = c["blocks"][0]["data"]["items"].as_array().unwrap();
        let first = items[0].as_str().unwrap();
        assert!(!first.contains("onerror"), "onerror must be dropped, got: {first}");
        let second = items[1].as_str().unwrap();
        assert!(second.contains("<i>safe</i>"), "benign markup survives: {second}");
    }

    #[test]
    fn strips_javascript_url_from_raw() {
        let mut c = serde_json::json!({
            "blocks": [
                { "type": "raw", "data": { "html": "<a href=\"javascript:alert(1)\">click</a>" } }
            ]
        });
        sanitize_editorjs_content(&mut c);
        let html = c["blocks"][0]["data"]["html"].as_str().unwrap();
        assert!(
            !html.contains("javascript:"),
            "javascript: URL must be dropped, got: {html}"
        );
        assert!(html.contains("click"), "link text survives: {html}");
    }

    #[test]
    fn code_block_is_untouched() {
        // `code` is rendered as an escaped text node — it must round-trip
        // verbatim, including `<` and even literal `<script>` inside a sample.
        let mut c = serde_json::json!({
            "blocks": [
                { "type": "code", "data": { "code": "let x = a < b;\n<script>alert(1)</script>" } }
            ]
        });
        let before = c["blocks"][0]["data"]["code"].as_str().unwrap().to_owned();
        sanitize_editorjs_content(&mut c);
        let after = c["blocks"][0]["data"]["code"].as_str().unwrap();
        assert_eq!(before, after, "code block must be untouched on read");
    }

    #[test]
    fn header_text_is_untouched() {
        // Header text is consumed by the escaped header renderer; sanitizing it
        // server-side would double-encode literal `<`. The TOC inner_html sink
        // is neutralized at the frontend instead.
        let mut c = serde_json::json!({
            "blocks": [
                { "type": "header", "data": { "text": "if a < b then", "level": 2 } }
            ]
        });
        sanitize_editorjs_content(&mut c);
        assert_eq!(
            c["blocks"][0]["data"]["text"].as_str().unwrap(),
            "if a < b then"
        );
    }

    #[test]
    fn unknown_block_types_pass_through() {
        let mut c = serde_json::json!({
            "blocks": [
                { "type": "table", "data": { "content": [["a < b", "c > d"]] } },
                { "type": "checklist", "data": { "items": [{ "text": "x", "checked": false }] } }
            ]
        });
        let before = c.clone();
        sanitize_editorjs_content(&mut c);
        assert_eq!(c, before, "non-sink blocks must be byte-identical");
    }

    #[test]
    fn missing_or_malformed_blocks_is_noop() {
        let cases = [
            serde_json::json!({}),
            serde_json::json!({ "blocks": "not-an-array" }),
            serde_json::json!({ "blocks": [/* empty */] }),
            serde_json::json!({ "blocks": [{ "type": "paragraph" /* no data */ }] }),
        ];
        for mut c in cases {
            // Must not panic.
            sanitize_editorjs_content(&mut c);
        }
    }

    #[test]
    fn serialize_adapter_cleans_without_mutating_source() {
        let value = serde_json::json!({
            "blocks": [
                { "type": "paragraph", "data": { "text": "<script>evil()</script>ok" } }
            ]
        });

        #[derive(serde::Serialize)]
        struct Wrap<'a>(
            #[serde(serialize_with = "serialize_sanitized_content")] &'a Value,
        );

        let serialized = serde_json::to_string(&Wrap(&value)).unwrap();
        assert!(!serialized.contains("<script"), "serialized output is clean");
        assert!(serialized.contains("ok"));
        // The original value is unmutated (sanitization-on-read, not in place).
        assert!(
            value["blocks"][0]["data"]["text"]
                .as_str()
                .unwrap()
                .contains("<script>"),
            "source value must be unmutated"
        );
    }

    #[test]
    fn string_adapter_cleans_json_string_content() {
        // Revision content is stored as a JSON *string*; the string adapter
        // must parse, sanitize, and re-emit clean JSON.
        let raw = serde_json::to_string(&serde_json::json!({
            "blocks": [
                { "type": "paragraph", "data": { "text": "<script>alert(1)</script>hi" } },
                { "type": "list", "data": { "items": ["<i>ok</i>", "<b onclick=x>y</b>"] } }
            ]
        }))
        .unwrap();

        #[derive(serde::Serialize)]
        struct Wrap<'a>(
            #[serde(serialize_with = "serialize_sanitized_content_string")] &'a String,
        );

        let serialized = serde_json::to_string(&Wrap(&raw)).unwrap();
        assert!(!serialized.contains("<script"), "script must be stripped");
        assert!(!serialized.contains("onclick"), "event handler must be stripped");
        assert!(serialized.contains("hi"));
        assert!(serialized.contains("<i>ok</i>"), "benign markup survives");
        // Source string left unmutated.
        assert!(raw.contains("<script>"));
    }

    #[test]
    fn string_adapter_passes_malformed_through() {
        // Non-JSON string must round-trip verbatim (not corrupted, not dropped).
        let raw = String::from("plain text, not json <script>x</script>");
        #[derive(serde::Serialize)]
        struct Wrap<'a>(
            #[serde(serialize_with = "serialize_sanitized_content_string")] &'a String,
        );
        let serialized = serde_json::to_string(&Wrap(&raw)).unwrap();
        assert!(
            serialized.contains("<script>"),
            "malformed non-JSON content is passed through, not sanitized"
        );
    }
}
