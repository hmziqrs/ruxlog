# Editor.js Integration Plan (Dioxus Web)

This plan integrates Editor.js as the rich, block-based editor for blog posts, replacing the current `textarea` in the BlogForm. It is incremental: a fast CDN-based setup for immediate use, followed by an optional npm/bun-managed setup for production stability and offline builds.

## Goals
- Replace the BlogForm `textarea` with Editor.js.
- Keep autosave and publish flows intact (store → services → API).
- Store content as a `String` in `Post{Create,Edit}Payload` while deciding JSON vs HTML format.
- Make the integration reversible and easy to iterate on (no deep infra changes required).

## Key Decisions
- Storage: store Editor.js output as JSON (stringified) in `Post*Payload.content`.
- Tools (core only for v1): header, list, image, code, quote.
- Autosave: 1500ms debounce (reuse existing logic in BlogForm).
- Initial data: seed new posts with an H2 header and a paragraph (blog default).
- Media uploads: phase 1 uses URL-only image insertion; phase 2 integrates uploads via the `store/media` actions (Rust), bridged from JS.

## Quick Start (CDN) — Recommended for first merge
This is fastest and requires zero bundling. We'll load the plugins you listed via CDN (using `@latest` for now); we can pin versions later for stability.

1) Create a small Dioxus host component that loads Editor.js scripts and renders the holder element.

Example host component skeleton (CDN scripts via a vector + loop):

```rust
use dioxus::prelude::*;

#[component]
pub fn EditorJsHost(initial_json: Option<String>) -> Element {
    // All plugin scripts loaded from CDN (core only)
    let editor_cdn_scripts = vec![
        // Core + core tools (using @latest for now; can pin later)
        "https://cdn.jsdelivr.net/npm/@editorjs/editorjs@latest",
        "https://cdn.jsdelivr.net/npm/@editorjs/header@latest",
        "https://cdn.jsdelivr.net/npm/@editorjs/list@latest",
        "https://cdn.jsdelivr.net/npm/@editorjs/image@latest",
        "https://cdn.jsdelivr.net/npm/@editorjs/code@latest",
        "https://cdn.jsdelivr.net/npm/@editorjs/quote@latest",
    ];

    rsx! {
        // Load all Editor.js scripts from the vector
        {
            editor_cdn_scripts.iter().enumerate().map(|(i, src)| rsx! {
                document::Script { key: "cdn-{i}", src: "{src}" }
            })
        }

        // Project init script: mounts EditorJS and hooks save/autosave
        document::Script { src: asset!("/assets/editor.js") }

        // Holder for the editor
        div { id: "editorjs", class: "min-h-[300px] border rounded-md" }

        // Optional debugging / manual save triggers
        // button { id: "save-editor", class: "btn btn-sm mt-3", "Save" }
    }
}
```

2) Add a simple init script at `assets/editor.js` that:
- Waits for DOM ready and global scripts to be present.
- Mounts `EditorJS` at `#editorjs` with desired tools.
- Exposes a small API on `window.editor` for Rust to call via JS.
- Emits `CustomEvent`s for autosave and content changes.

Starter (JS) outline (keeps working with the CDN globals):

```js
// assets/editor.js
(function () {
  const ready = (fn) => (
    document.readyState === 'loading' ? document.addEventListener('DOMContentLoaded', fn) : fn()
  );

  ready(() => {
    if (!window.EditorJS) {
      console.error('[EditorJS] core not loaded');
      return;
    }

    const editor = new EditorJS({
      holder: 'editorjs',
      tools: {
        header: window.Header,
        list: window.List,
        // Image: by-URL only in Phase 1; file uploads in Phase 2 via store
        image: {
          class: window.ImageTool || window.SimpleImage,
          config: {
            uploader: {
              // Basic by-URL support: immediately accept the URL
              uploadByUrl: async (url) => ({ success: 1, file: { url } }),
            },
          },
        },
        code: window.CodeTool,
        quote: window.Quote,
      },
      placeholder: 'Start writing your post…',
      // Optionally hydrate initial content if provided via a global
      data: window.__EDITOR_INITIAL_DATA__ || {
        blocks: [
          { type: 'header', data: { text: 'Welcome to Editor.js!', level: 2 } },
          { type: 'paragraph', data: { text: 'Start writing your blog post here…' } }
        ]
      },
      onChange: async () => {
        try {
          const data = await editor.save();
          window.dispatchEvent(
            new CustomEvent('editor:change', { detail: data })
          );
        } catch (e) {
          console.error('[EditorJS] onChange save failed', e);
        }
      },
    });

    // Expose helpers for Rust to call via JS eval
    window.editor = {
      save: () => editor.save(),
      clear: () => editor.clear(),
      render: (data) => editor.render(data),
      destroy: () => editor.destroy(),
    };

    // Optional: wire a manual save button for debugging
    const btn = document.getElementById('save-editor');
    if (btn) {
      btn.addEventListener('click', async () => {
        const data = await editor.save();
        console.log('[EditorJS] save', data);
        window.dispatchEvent(new CustomEvent('editor:save', { detail: data }));
      });
    }
  });
})();
```

3) Replace the textarea in BlogForm with `EditorJsHost` and bridge state:
- Integration point: `src/containers/blog_form/blog_form.rs:334` (the current `// Content field` block).
- Pass initial content into the editor (if JSON present) via a global, e.g., set `window.__EDITOR_INITIAL_DATA__` in a small inline script or via `use_effect` + `use_eval`.
- Subscribe to `window` events (`editor:change`) to update form state and autosave.

Minimal bridging hooks (Rust):
- Use `use_effect` to register a JS event listener via `use_eval` (Dioxus Web) or `web_sys`.
- On `editor:change`, receive JSON, `serde_json::to_string` and update `form.data.content`.
- On submit, use `form.data.content` as-is (stringified JSON) for `Post{Create,Edit}Payload.content`.

4) Autosave
- Reuse the existing autosave debounce in BlogForm. Instead of text input, trigger autosave when receiving `editor:change`. The payload stays `String` (JSON string) until a rendering decision is made.

5) Viewer (read side)
- For list/preview screens, continue to show excerpt or a plain text summary.
- For full post view (when implemented), either:
  - Render JSON with Editor.js on a read‑only instance, or
  - Convert JSON → HTML (client or server) and render as HTML.

## Media Integration Roadmap
- Phase 1 (this PR): Enable image tool for by-URL insertion only; avoid file uploads to keep JS↔Rust bridge minimal.
- Phase 2: Provide a custom `uploader` for the image tool that accepts a `File`, calls a small JS glue `window.dioxus_media_upload(file)` implemented with `wasm_bindgen` to pass the `web_sys::File` into Rust, and invokes `use_media().upload(payload)`. On success, return `{ success: 1, file: { url } }` to Editor.js. This reuses existing `src/store/media` upload logic and API endpoints, no UI dialogs.

## Package‑Managed Setup (npm/bun) — Optional hardening
Out of scope for now (per request). We will rely exclusively on CDN for a smooth first pass. Keeping steps here for future reference.

1) Install packages with bun (mirror of npm):

```sh
bun add -D \
  @editorjs/editorjs@2.30.7 \
  @editorjs/header@2.8.1 \
  @editorjs/list@1.9.0 \
  @editorjs/quote@2.7.2 \
  @editorjs/delimiter@1.4.0 \
  @editorjs/image@2.8.1 \
  @editorjs/embed@2.6.0 \
  @editorjs/link-tool@2.6.2 \
  @editorjs/attaches@1.3.0 \
  @editorjs/code@2.8.0 \
  @editorjs/raw@2.4.2 \
  @editorjs/inline-code@1.4.0 \
  @editorjs/table@2.3.0 \
  @editorjs/checklist@1.6.0 \
  @editorjs/warning@1.3.0 \
  @editorjs/marker@1.3.0 \
  @editorjs/underline@1.2.0 \
  @editorjs/nested-list@1.4.2 \
  editorjs-drag-drop@1.1.15 \
  editorjs-undo@1.0.7 \
  @editorjs/text-variant-tune@1.0.4 \
  editorjs-button@1.0.6 \
  editorjs-alert@1.1.5
```

2) Create `assets/editor/index.ts` importing Editor.js and tools, and implement the same init code as above, but with ESM imports.

3) Bundle with bun:

```sh
bun build assets/editor/index.ts --outdir assets --outfile editor.bundle.js --minify --format=esm
```

4) Load the bundle instead of CDN scripts:

- Replace the CDN `<script>` tags with `<script type="module" src={asset!("/assets/editor.bundle.js")}></script>` in `EditorJsHost`.
- Ensure `Dioxus.toml` watches `assets/` (already configured).

5) Add a package.json script and mention in docs:

```json
{
  "scripts": {
    "editor:build": "bun build assets/editor/index.ts --outdir assets --outfile editor.bundle.js --minify --format=esm"
  }
}
```

## Integration Points
- Replace content field in: `src/containers/blog_form/blog_form.rs:334`
- Optional head injection (global): `index.html:6` or via `document::Script` inside the `EditorJsHost`
- Project script (if needed): `scripts/header.ts:1` can be used to inject tags at build, but `document::Script` is simpler.
- Assets served under: `assets/` (watched by `dx serve` via `Dioxus.toml`).

## Data Flow & Store
- Form state remains source of truth in Rust; `form.data.content` holds stringified JSON.
- On change: JS → `window` event → Rust handler → update form → debounce → `posts.autosave`.
- On submit: `PostCreatePayload.content` = `form.data.content`.
- On edit screen hydrate: detect if `content` is valid JSON; if yes, pass to editor as initial data; if not, keep textarea fallback or run a migration (TBD).

## Testing & QA
- Dev: `dx serve` and validate editor loads, typing works, and autosave fires.
- Edge cases:
  - Large documents save within debounce window.
  - Switching between draft/published preserves content.
  - Reloading edit screen hydrates previous JSON content correctly.
- Avoid introducing regressions to tag/category selectors and submission flow.

## Rollout Plan
1) Land CDN quick start with `EditorJsHost` + `assets/editor.js` and BlogForm replacement behind a feature flag (if desired).
2) Verify autosave and submit flows to backend.
3) Decide JSON vs HTML strategy for content storage and viewing.
4) Optional: migrate to npm/bun bundle for determinism and offline builds.
5) Add more tools (image upload, table, checklist) and wire uploads to `services/http_client.rs`.

## Notes
- Security: Pin versions when using CDN; prefer bundling for production.
- Performance: Lazy load advanced tools only when needed (future improvement).
- Accessibility: Provide keyboard shortcuts and clear focus states; ensure editor area announces as a text box.

## Open Questions
- Content format: store Editor.js output as JSON (stringified) or convert to HTML at save time? (Plan assumes JSON initially.)
- Image uploads: do we have an upload endpoint and auth requirements for `@editorjs/image`/`attaches`? Provide URL(s) and expected payload/response shape.
- Tool set: do you want all listed tools enabled from day one, or start minimal (header, list, image, code, quote) and add the rest incrementally?
- Desktop preview: should we guard the CDN loader to run only on web builds, and keep a textarea fallback for desktop?
- Autosave debounce: keep current 1500ms interval or adjust?
- Initial content: can we assume new posts start with an empty JSON or a two-block seed (header + paragraph) like the sample?
- Backwards compatibility: if existing posts contain HTML, should we keep a textarea fallback or run a one-time JSON migration strategy?

---

References
- BlogForm container: `src/containers/blog_form/blog_form.rs:334`
- Dioxus resource watching: `Dioxus.toml:7`
- HTML shell: `index.html:1`
- Optional header script hook: `scripts/header.ts:1`
