use std::{borrow::Cow, io::Cursor};

use bytes::Bytes;
use image::{codecs, imageops::FilterType, ColorType, DynamicImage, ImageEncoder, ImageFormat};
use thiserror::Error;

use crate::{modules::media_v1::validator::MediaUploadMetadata, state::OptimizerConfig};

use crate::db::sea_models::media::MediaReference;

#[derive(Debug, Clone)]
pub struct OptimizationRequest<'a> {
    pub bytes: &'a Bytes,
    pub metadata: &'a MediaUploadMetadata,
    pub reference: Option<MediaReference>,
    pub original_mime: Option<&'a str>,
    pub original_extension: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    Disabled,
    UnsupportedFormat,
    ExceedsPixelBudget,
    AlreadyOptimized,
    DecodeFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantLabel {
    Original,
    Width(u32),
    Lqip,
}

#[derive(Debug, Clone)]
pub struct OptimizedImage {
    pub bytes: Bytes,
    pub mime_type: String,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    pub label: VariantLabel,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub replaced_original: bool,
    pub original: OptimizedImage,
    pub variants: Vec<OptimizedImage>,
}

#[derive(Debug, Clone)]
pub enum OptimizationOutcome {
    Skipped(SkipReason),
    Optimized(OptimizationResult),
}

#[derive(Debug)]
struct ProbedImage {
    width: u32,
    height: u32,
    pixel_count: u64,
    format: ImageFormat,
    mime: Cow<'static, str>,
    extension: Cow<'static, str>,
    bytes_per_pixel: f32,
}

#[derive(Debug, Error)]
pub enum OptimizationError {
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("failed to decode image: {0}")]
    DecodeFailed(String),
    #[error("encoding error: {0}")]
    EncodeFailed(String),
}

const LOSSY_BPP_THRESHOLD: f32 = 1.5;
const LOSSLESS_BPP_THRESHOLD: f32 = 3.0;

pub fn optimize(
    config: &OptimizerConfig,
    request: OptimizationRequest<'_>,
) -> Result<OptimizationOutcome, OptimizationError> {
    if !config.enabled {
        return Ok(OptimizationOutcome::Skipped(SkipReason::Disabled));
    }

    let probed = match probe(
        request.bytes,
        request.original_mime,
        request.original_extension,
    ) {
        Ok(info) => info,
        Err(SkipReason::UnsupportedFormat) => {
            return Ok(OptimizationOutcome::Skipped(SkipReason::UnsupportedFormat));
        }
        Err(reason) => return Ok(OptimizationOutcome::Skipped(reason)),
    };

    if probed.pixel_count > config.max_pixels {
        return Ok(OptimizationOutcome::Skipped(SkipReason::ExceedsPixelBudget));
    }

    if should_skip_for_quality(&probed) {
        return Ok(OptimizationOutcome::Skipped(SkipReason::AlreadyOptimized));
    }

    let strategy = match request.reference.unwrap_or(MediaReference::Post) {
        MediaReference::Category => Strategy::Category,
        MediaReference::User => Strategy::User,
        MediaReference::Post => Strategy::Post,
    };

    if strategy.should_skip(&probed) {
        return Ok(OptimizationOutcome::Skipped(SkipReason::AlreadyOptimized));
    }

    let decoded = match image::load_from_memory(request.bytes) {
        Ok(image) => image,
        Err(_) => return Ok(OptimizationOutcome::Skipped(SkipReason::DecodeFailed)),
    };

    let plan = strategy.build_plan(&decoded);

    let mut result = OptimizationResult {
        replaced_original: false,
        original: OptimizedImage {
            bytes: request.bytes.clone(),
            mime_type: probed.mime.to_string(),
            extension: probed.extension.to_string(),
            width: probed.width,
            height: probed.height,
            label: VariantLabel::Original,
        },
        variants: Vec::new(),
    };

    if let Some(replacement) = strategy.reencode_original(
        &decoded,
        &probed,
        config.default_webp_quality,
        request.bytes.len(),
    )? {
        result.replaced_original = true;
        result.original = replacement;
    }

    for spec in plan {
        if spec.width >= probed.width {
            continue;
        }

        if let Some(variant) = encode_variant(&decoded, &spec)? {
            result.variants.push(variant);
        }
    }

    if !result.replaced_original && result.variants.is_empty() {
        return Ok(OptimizationOutcome::Skipped(SkipReason::AlreadyOptimized));
    }

    Ok(OptimizationOutcome::Optimized(result))
}

fn probe(
    bytes: &Bytes,
    mime_hint: Option<&str>,
    ext_hint: Option<&str>,
) -> Result<ProbedImage, SkipReason> {
    let size = imagesize::blob_size(bytes).map_err(|_| SkipReason::DecodeFailed)?;
    let format = image::guess_format(bytes).map_err(|_| SkipReason::UnsupportedFormat)?;

    let extension = match ext_hint.and_then(normalize_extension) {
        Some(ext) => Cow::Owned(ext),
        None => Cow::Owned(format_extensions(&format)),
    };

    let mime = match mime_hint {
        Some(mime) if !mime.trim().is_empty() => Cow::Owned(mime.to_ascii_lowercase()),
        _ => Cow::Owned(format_mime(&format)),
    };

    let width = size.width as u32;
    let height = size.height as u32;
    let pixel_count = u64::from(width) * u64::from(height);
    let bytes_per_pixel = bytes.len() as f32 / pixel_count.max(1) as f32;

    Ok(ProbedImage {
        width,
        height,
        pixel_count,
        format,
        mime,
        extension,
        bytes_per_pixel,
    })
}

fn normalize_extension(ext: &str) -> Option<String> {
    let trimmed = ext.trim().trim_start_matches('.');
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_ascii_lowercase())
    }
}

fn format_extensions(format: &ImageFormat) -> String {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::WebP => "webp",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tif",
        ImageFormat::Avif => "avif",
        ImageFormat::Tga => "tga",
        ImageFormat::Dds => "dds",
        ImageFormat::Pnm => "pnm",
        ImageFormat::Ico => "ico",
        ImageFormat::Hdr => "hdr",
        ImageFormat::OpenExr => "exr",
        _ => "img",
    }
    .to_string()
}

fn format_mime(format: &ImageFormat) -> String {
    match format {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::WebP => "image/webp",
        ImageFormat::Gif => "image/gif",
        ImageFormat::Bmp => "image/bmp",
        ImageFormat::Tiff => "image/tiff",
        ImageFormat::Avif => "image/avif",
        ImageFormat::Tga => "image/x-tga",
        ImageFormat::Dds => "image/vnd.ms-dds",
        ImageFormat::Pnm => "image/x-portable-anymap",
        ImageFormat::Ico => "image/x-icon",
        ImageFormat::Hdr => "image/vnd.radiance",
        ImageFormat::OpenExr => "image/x-exr",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn should_skip_for_quality(probed: &ProbedImage) -> bool {
    match probed.format {
        ImageFormat::Png => probed.bytes_per_pixel <= LOSSLESS_BPP_THRESHOLD,
        ImageFormat::WebP | ImageFormat::Jpeg => probed.bytes_per_pixel <= LOSSY_BPP_THRESHOLD,
        _ => false,
    }
}

enum Strategy {
    Post,
    User,
    Category,
}

struct VariantSpec {
    width: u32,
    format: TargetFormat,
    quality: u8,
    kind: ResizeKind,
}

#[allow(dead_code)]
enum TargetFormat {
    Webp,
    Jpeg,
    Png,
}

enum ResizeKind {
    FitWidth,
    ExactSquare,
}

impl Strategy {
    fn should_skip(&self, probed: &ProbedImage) -> bool {
        match self {
            Strategy::User => probed.width <= 96 && probed.height <= 96,
            Strategy::Category => probed.width <= 256 && probed.height <= 256,
            Strategy::Post => probed.width <= 480,
        }
    }

    fn build_plan(&self, image: &DynamicImage) -> Vec<VariantSpec> {
        match self {
            Strategy::User => {
                let size = image.width().min(image.height());
                [48, 96, 192, 384]
                    .into_iter()
                    .filter(|target| *target < size)
                    .map(|width| VariantSpec {
                        width,
                        format: TargetFormat::Webp,
                        quality: 82,
                        kind: ResizeKind::ExactSquare,
                    })
                    .collect()
            }
            Strategy::Category => {
                vec![
                    VariantSpec {
                        width: 256,
                        format: TargetFormat::Png,
                        quality: 100,
                        kind: ResizeKind::FitWidth,
                    },
                    VariantSpec {
                        width: 640,
                        format: TargetFormat::Webp,
                        quality: 80,
                        kind: ResizeKind::FitWidth,
                    },
                ]
            }
            Strategy::Post => [480, 768, 1024, 1600, 2048]
                .into_iter()
                .map(|width| VariantSpec {
                    width,
                    format: TargetFormat::Webp,
                    quality: 80,
                    kind: ResizeKind::FitWidth,
                })
                .collect(),
        }
    }

    fn reencode_original(
        &self,
        image: &DynamicImage,
        probed: &ProbedImage,
        _quality: u8,
        original_size: usize,
    ) -> Result<Option<OptimizedImage>, OptimizationError> {
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::WebP)
            .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
        let buffer = cursor.into_inner();

        let improvement = 1.0 - (buffer.len() as f32 / original_size as f32);
        if improvement < 0.03 {
            return Ok(None);
        }

        Ok(Some(OptimizedImage {
            bytes: Bytes::from(buffer),
            mime_type: "image/webp".to_string(),
            extension: "webp".to_string(),
            width: probed.width,
            height: probed.height,
            label: VariantLabel::Original,
        }))
    }
}

fn encode_variant(
    source: &DynamicImage,
    spec: &VariantSpec,
) -> Result<Option<OptimizedImage>, OptimizationError> {
    if spec.width == 0 {
        return Ok(None);
    }

    let prepared = match spec.kind {
        ResizeKind::ExactSquare => {
            let min_side = source.width().min(source.height());
            let x = (source.width() - min_side) / 2;
            let y = (source.height() - min_side) / 2;
            source.crop_imm(x, y, min_side, min_side).resize_exact(
                spec.width,
                spec.width,
                FilterType::Lanczos3,
            )
        }
        ResizeKind::FitWidth => source.resize(spec.width, u32::MAX, FilterType::Lanczos3),
    };

    let (buffer, mime, extension) = match spec.format {
        TargetFormat::Webp => {
            let mut cursor = Cursor::new(Vec::new());
            prepared
                .write_to(&mut cursor, ImageFormat::WebP)
                .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
            (cursor.into_inner(), "image/webp", "webp")
        }
        TargetFormat::Jpeg => {
            let mut buffer = Vec::new();
            let mut encoder =
                codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, spec.quality);
            encoder
                .encode_image(&prepared)
                .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
            (buffer, "image/jpeg", "jpg")
        }
        TargetFormat::Png => {
            let mut buffer = Vec::new();
            let encoder = codecs::png::PngEncoder::new(&mut buffer);
            encoder
                .write_image(
                    prepared.to_rgba8().as_raw(),
                    prepared.width(),
                    prepared.height(),
                    ColorType::Rgba8.into(),
                )
                .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
            (buffer, "image/png", "png")
        }
    };

    Ok(Some(OptimizedImage {
        bytes: Bytes::from(buffer),
        mime_type: mime.to_string(),
        extension: extension.to_string(),
        width: prepared.width(),
        height: prepared.height(),
        label: VariantLabel::Width(prepared.width()),
    }))
}
