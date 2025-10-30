# Backend Image Optimizer Plan

This document outlines an optional, reference-aware image optimization pipeline for the media module using the `image` crate (image-rs). It focuses on: skipping optimization when images are already optimal, applying strategies based on `MediaReference` type, and following industry best practices for performance, quality, and safety.

## Goals
- Optional optimization at upload time; never block uploads on optimizer failures.
- Skip optimization when size/dimensions/format are already optimal.
- Strategy per `reference_type` (user, category, post) with sensible defaults when unspecified.
- Generate responsive variants and a consistent naming scheme.
- Keep storage and public URL conventions stable; variants live alongside the original.
- Safe, deterministic processing; bounded resource usage and strong guardrails.

## Scope
- Ingest single image files via existing `POST /media_v1/create`.
- Optimize synchronously on upload first; background/async worker is a later phase.
- Upload optimized original and/or variants to R2/S3 using the existing AWS SDK client.
- Persist base `media` record as today; add optional `media_variants` table in a later phase.

## Reference Types & Strategies
Reference types are defined in `src/db/sea_models/media/model.rs` as `MediaReference` with values `category`, `user`, `post`.

- User (avatars)
  - Targets: 48, 96, 192, 384 px squares.
  - Crop: center crop to square; never upscale.
  - Format: WebP lossy Q≈82 if photo; PNG or WebP lossless if transparent flat art.
  - Strip metadata; clamp to sRGB.

- Category (logos/banners)
  - Logos/icons (with alpha): PNG or WebP lossless; attempt palette/bit-depth reduction.
  - Banners/photos: WebP lossy Q≈80; fallback to JPEG progressive Q≈80 if WebP not desired.
  - Targets: icons 128/256; banners 640/1280/1920 (maintain aspect ratio).
  - Strip metadata; sRGB; preserve transparency when present.

- Post (content images) [default]
  - Targets: widths 480, 768, 1024, 1600, 2048 (maintain AR; cap max width to 2560).
  - Format: WebP lossy Q≈80 for photos; JPEG progressive Q≈80 as fallback.
  - Generate a tiny LQIP placeholder (e.g., width 24) for blurred previews.
  - Apply EXIF orientation normalization if/when EXIF parsing is added.

## Skip Optimization Criteria
Skip early if any of the following hold:
- Dimensions already ≤ smallest target for the strategy AND bytes-per-pixel (BPP) is below a threshold (e.g., ≤ 1.5 for lossy, ≤ 3.0 for PNG). Avoid re-encoding churn.
- Re-encoded candidate of same dimensions/format/quality is larger or not meaningfully smaller (e.g., < 3% delta) than the original.
- Image is already in a target format with acceptable quality and no larger than an alternative (e.g., current WebP smaller than candidate JPEG/WebP).
- Never upscale: if all strategy targets exceed original width/height, generate only variants ≤ original; do not replace original.
- Hard safety cap: reject decode if total pixels > `OPTIMIZER_MAX_PIXELS` (e.g., 40 MP) to avoid decompression bombs.

Notes:
- We will probe dimensions with `imagesize` (already present) prior to decode.
- “Already optimized” user hint can be considered later as an override to skip.

## Architecture
- New module: `src/services/image_optimizer.rs`
  - Optimize orchestrator:
    - `optimize_and_upload(state, file_bytes, meta, inferred_content_type) -> OptimizedOutcome`
      - Chooses strategy from `meta.reference_type`.
      - Applies skip logic using size and a quick in-memory trial encode.
      - If optimizing, decodes, resizes, encodes, and uploads (bounded concurrency) using existing S3 client.
      - Returns outcome specifying whether the original was replaced and the list of uploaded variants.
  - Strategy trait:
    ```rust
    pub trait OptimizeStrategy {
        fn plan(&self, meta: &MediaUploadMetadata, probed: Probed) -> Plan; // sizes, formats
        fn should_skip(&self, probed: &Probed) -> bool; // dimension/format/size heuristics
        fn produce(&self, img: &DynamicImage, plan: &Plan) -> Vec<VariantSpec>; // resize specs
    }
    ```
  - Concrete strategies:
    - `PostImageStrategy` (default)
    - `UserAvatarStrategy`
    - `CategoryArtworkStrategy`

- Data types
  - `Probed { width, height, mime, ext, size_bytes, bpp }`
  - `VariantSpec { width, height, format, quality, label }`
  - `OptimizedVariant { width, height, format, quality, object_key, size_bytes }`
  - `OptimizedOutcome { original_replaced: bool, original_key: String, variants: Vec<OptimizedVariant>, mime: String, ext: String }`

## Pipeline Details (image-rs)
- Decode: `image::load_from_memory` with format guessing; reject unsupported.
- Resize: `image::imageops::resize` using `FilterType::Lanczos3` for downscales.
- Encode:
  - JPEG: `image::codecs::jpeg::JpegEncoder::new_with_quality` (progressive enabled).
  - WebP: `image::codecs::webp::WebPEncoder` (lossy quality parameter).
  - PNG: `image::codecs::png::PngEncoder` with adequate compression; no ancillary chunks.
- Orientation: Initially ignore EXIF; later add EXIF normalization.
- Metadata: strip; standardize to sRGB where feasible.

Optional future improvement:
- Use `fast_image_resize` for higher-quality downscales with gamma-aware filtering.

## Storage & Naming
- Base object key (current): `media/YYYY/MM/uuid.ext`.
- Variants:
  - `media/YYYY/MM/uuid@{width}w.{fmt}` (e.g., `abc@768w.webp`).
  - LQIP: `media/YYYY/MM/uuid@lqip.{fmt}`.
- Keep original by default (`OPTIMIZER_KEEP_ORIGINAL=true`); if `false`, replace original using best format/quality with same base name+ext.

## Database Changes (later phase)
- New table: `media_variants`
  - Columns: `id`, `media_id (fk)`, `width`, `height`, `format`, `quality`, `object_key (unique)`, `size`, `created_at`.
- Optional columns on `media`: `is_optimized bool`, `optimized_at timestamp`, `original_key text`.
- List API can include variants in payloads when implemented.

## Configuration
Environment variables (with sensible defaults):
- `OPTIMIZE_ON_UPLOAD` (default: true)
- `OPTIMIZER_MAX_PIXELS` (default: 40000000)
- `OPTIMIZER_KEEP_ORIGINAL` (default: true)
- `OPTIMIZER_WEBP_QUALITY_DEFAULT` (default: 80)
- Per-reference presets can be in code constants or env overrides later.

## Integration Points
- Upload handler: `src/modules/media_v1/controller.rs` (create)
  1. Read `file_bytes`, metadata, infer extension/mime.
  2. If `OPTIMIZE_ON_UPLOAD=true` and content is an image, call orchestrator.
  3. On `OptimizedOutcome`:
     - If `original_replaced`, upload optimized original and persist new mime/ext/size/dimensions.
     - Upload variants; collect results; optionally persist to `media_variants`.
  4. On any optimizer error: log, fall back to current behavior, and proceed.

No route or schema changes are needed in Phase 1.

## Operational Considerations
- Safety: pixel cap, MIME/format allowlist, size limits, and decode timeouts.
- Performance: bounded parallel uploads (e.g., 3–4 at a time); reuse buffers.
- Observability: tracing spans for decode/resize/encode/upload, with size deltas.
- Idempotency: deterministic naming; safe to retry uploads.

## Phased Delivery
1. ✅ **Phase 1** (completed in current implementation)
   - `image` crate added and orchestrator scaffolding lives in `src/services/image_optimizer.rs`.
   - Skip logic, probing, and optional WebP re-encode implemented with fail-open behaviour.
   - Media upload flow now calls the optimizer behind the `OPTIMIZE_ON_UPLOAD` gate; no schema changes required yet.

2. ✅ **Phase 2** (completed)
   - Reference-aware strategies now shape variant sets: avatars (square JPEG/PNG), category icons/banners, and post content (multi-width JPEG plus LQIP).
   - Optimized variants are generated and uploaded alongside the base asset while the canonical media record remains unchanged.
   - Variant persistence and API surfaces stay deferred to Phase 3.
   - Note: the `image` crate only supports lossless WebP encoding, so lossy outputs currently use JPEG for photo-oriented variants; switching to `webp` crate remains an option for later phases.

3. ⬜ **Phase 3**
   - Introduce `media_variants` table and persistence.
   - Optionally extend list/response payloads with variant summaries.

4. ⬜ **Phase 4** (optional)
   - Async optimization worker (Redis queue), content hashing dedupe, EXIF normalization, LQIP.

## Dependencies
- Add to `Cargo.toml`:
  ```toml
  image = { version = "^0.25", default-features = false, features = ["png", "jpeg", "webp", "tiff"] }
  ```
- Optional later: `fast_image_resize` and `exif` crates.

## Best Practices Summary
- Never upscale; preserve transparency when present.
- Prefer WebP lossy for photos; PNG/WebP lossless for flat/alpha art.
- Use responsive widths; strip metadata; standardize to sRGB.
- Guard rails against decompression bombs and excessive CPU time.
- Keep uploads resilient: fail-open with clear logs; do not block user flow.
