# EditorJS Media ID Implementation Plan

## Overview
Implement media ID storage for EditorJS inline images in blog posts, allowing database joins while keeping the built-in image tool's resizing/alignment features.

## Current State
- **EditorJS Image Tool**: Currently stores `{ url: "..." }` directly in post content
- **Media Store**: Categories successfully use media IDs (logo_media_id, cover_media_id) with `use_media().upload()`
- **PostView Renderer**: Custom renderer handles EditorJS blocks (header, paragraph, code, quote, alert, checklist) but not images yet
- **Upload Flow**: `window.editorjs_upload_file()` → `js_bridge.rs` → returns `{ success: 1, file: { url: media.file_url } }`

## Solution: Custom Image Tool with Media ID

### Approach
Create a custom EditorJS tool that:
1. Wraps/extends the built-in @editorjs/image tool
2. Preserves all resizing and alignment features
3. Stores both `media_id` and `url` in saved data: `{ media_id: 123, url: "https://..." }`
4. Uses `url` for EditorJS display
5. Uses `media_id` for database storage and joins

### Data Format
```json
{
  "type": "image",
  "data": {
    "media_id": 123,
    "url": "https://cdn.example.com/image.jpg",
    "caption": "Image caption",
    "stretched": false,
    "withBackground": false,
    "withBorder": false
  }
}
```

## Implementation Steps

### 1. Update Upload Response (src/utils/js_bridge.rs)

**Current:**
```rust
let response = EditorJsUploadResponse {
    success: 1,
    file: EditorJsFile {
        url: media.file_url,
    },
};
```

**Updated:**
```rust
#[derive(Serialize)]
struct EditorJsFile {
    url: String,
    media_id: i32,
}

let response = EditorJsUploadResponse {
    success: 1,
    file: EditorJsFile {
        url: media.file_url,
        media_id: media.id,
    },
};
```

### 2. Create Custom Image Tool (assets/editor/index.ts)

Create a new tool class that extends the Image tool:

```typescript
import Image from "@editorjs/image";

class ImageWithMediaId extends Image {
  async save(blockContent: HTMLElement) {
    // Get base data from parent (url, caption, etc.)
    const baseData = await super.save(blockContent);

    // Add media_id from file URL context
    // The media_id is already available from upload response
    return {
      ...baseData,
      media_id: baseData.file?.media_id || null
    };
  }

  static get pasteConfig() {
    return {
      patterns: {
        image: /image\/[\w\.\-\/]+/
      }
    };
  }
}
```

Or use a wrapper approach:

```typescript
class ImageWithMediaId {
  constructor(config: any) {
    this.innerImage = new Image(config);
  }

  render() {
    return this.innerImage.render();
  }

  async save(blockContent: HTMLElement) {
    const baseData = await this.innerImage.save(blockContent);
    // EditorJS automatically has access to upload response
    // We can access it through the block data
    return {
      ...baseData,
      media_id: baseData.file?.media_id || null
    };
  }
}
```

### 3. Configure EditorJS (assets/editor/index.ts)

**Current:**
```typescript
image: {
  class: Image,
  config: {
    uploader: {
      uploadByFile: async (file: File) => {
        const response = await window.editorjs_upload_file(file);
        return response;
      }
    }
  }
}
```

**Updated:**
```typescript
image: {
  class: ImageWithMediaId, // Use custom class
  config: {
    uploader: {
      uploadByFile: async (file: File) => {
        // Response now includes media_id
        const response = await window.editorjs_upload_file(file);
        return response;
      }
    }
  }
}
```

### 4. Update PostView Renderer (src/screens/posts/view/components.rs)

**Add new variant to EditorJsBlock enum:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum EditorJsBlock {
    // ... existing variants ...
    #[serde(rename = "image")]
    ImageWithMediaId {
        id: String,
        data: ImageBlock,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImageBlock {
    media_id: Option<i32>,
    url: String,
    caption: Option<String>,
    stretched: bool,
    with_background: bool,
    with_border: bool,
}
```

**Add renderer function:**
```rust
fn render_image_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::ImageWithMediaId { data, .. } = block {
        let url = data.url.clone();
        let caption = data.caption.clone();
        let stretched = data.stretched;

        let container_class = if stretched {
            "my-8 -mx-6 sm:-mx-8 md:-mx-12 lg:-mx-16"
        } else {
            "my-6 max-w-full"
        };

        rsx! {
            figure { class: "{container_class}",
                img {
                    src: "{url}",
                    class: "w-full h-auto rounded-lg",
                    alt: caption.as_deref().unwrap_or("")
                }
                if let Some(caption) = caption {
                    figcaption {
                        class: "text-sm text-center text-muted-foreground mt-2 italic",
                        "{caption}"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}
```

**Update main renderer:**
```rust
fn render_editorjs_content(content: &str) -> Element {
    match serde_json::from_str::<EditorJsData>(content) {
        Ok(data) => {
            rsx! {
                div { class: "prose prose-neutral dark:prose-invert max-w-none",
                    for block in &data.blocks {
                        match block {
                            // ... existing cases ...
                            EditorJsBlock::ImageWithMediaId { .. } => render_image_block(block),
                            EditorJsBlock::Unknown { .. } => render_unknown_block(block),
                        }
                    }
                }
            }
        }
        // ... error handling ...
    }
}
```

### 5. Optional: Database Join for URL Resolution

If you want to resolve URLs via database join instead of storing the URL:

**In PostView renderer:**
```rust
// Fetch all media used in the post
let media_map = use_memo(|| {
    let mut map = HashMap::new();
    // Parse content, extract all media_id values
    // Fetch media objects
    // Build map: media_id -> Media
    map
});

// In render_image_block:
let media = media_map.get(&data.media_id);
let url = media.map(|m| &m.file_url).unwrap_or(&data.url);
```

## Benefits

✅ **Keeps all image tool features**: Resize, alignment, caption, etc.
✅ **Stores media IDs**: Enables database joins for URL management
✅ **Backward compatible**: Can handle existing URL-only data
✅ **Consistent pattern**: Matches category/cover image approach
✅ **Flexible rendering**: Can use URL directly or join with media table

## Migration Path

**For existing posts with URL-only images:**
- Update renderer to handle both `url` only and `media_id + url` formats
- Gradually migrate to media_id format as posts are edited

**Storage:**
- Keep `url` field for backward compatibility and immediate rendering
- Use `media_id` for proper database normalization

## Testing Checklist

- [ ] Upload an image in EditorJS
- [ ] Verify JSON includes `media_id`
- [ ] Save post to database
- [ ] Render post view with image
- [ ] Test all image features: caption, alignment, resize
- [ ] Verify media ID is properly stored
- [ ] Test backward compatibility with old posts

## Related Files

- `assets/editor/index.ts` - EditorJS bundle and custom tool
- `src/utils/js_bridge.rs` - Upload bridge to Rust
- `src/screens/posts/view/components.rs` - Post renderer
- `src/store/media/state.rs` - Media store types
- `src/store/media/actions.rs` - Upload actions
