use std::borrow::Cow;
use std::time::Instant;

use bytes::Bytes;
use image::{
    codecs, imageops::FilterType, ColorType, DynamicImage, ExtendedColorType, ImageEncoder,
    ImageFormat,
};
use opentelemetry::KeyValue;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

use crate::utils::telemetry;
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
    pub quality: Option<u8>,
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

#[derive(Debug)]
struct ImageCharacteristics {
    has_alpha: bool,
    aspect_ratio: f32,
    min_dimension: u32,
}

fn analyze_image(image: &DynamicImage) -> ImageCharacteristics {
    let width = image.width().max(1);
    let height = image.height().max(1);
    let aspect_ratio = width as f32 / height as f32;
    let color = image.color();
    let has_alpha = color.has_alpha();

    ImageCharacteristics {
        has_alpha,
        aspect_ratio,
        min_dimension: width.min(height),
    }
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

#[instrument(skip(config, request), fields(
    input_size = request.bytes.len(),
    reference = ?request.reference,
    outcome,
    bytes_saved,
    variant_count
))]
pub fn optimize(
    config: &OptimizerConfig,
    request: OptimizationRequest<'_>,
) -> Result<OptimizationOutcome, OptimizationError> {
    let metrics = telemetry::image_metrics();
    let start = Instant::now();
    metrics.optimization_requests.add(1, &[]);

    if !config.enabled {
        debug!("Optimization disabled by config");
        tracing::Span::current().record("outcome", "skipped_disabled");
        metrics
            .optimization_skipped
            .add(1, &[KeyValue::new("reason", "disabled")]);
        return Ok(OptimizationOutcome::Skipped(SkipReason::Disabled));
    }

    let probed = match probe(
        request.bytes,
        request.original_mime,
        request.original_extension,
    ) {
        Ok(info) => {
            debug!(
                width = info.width,
                height = info.height,
                format = ?info.format,
                pixels = info.pixel_count,
                "Image probed successfully"
            );
            info
        }
        Err(SkipReason::UnsupportedFormat) => {
            warn!(mime = ?request.original_mime, ext = ?request.original_extension, "Unsupported format");
            tracing::Span::current().record("outcome", "skipped_unsupported");
            metrics
                .optimization_skipped
                .add(1, &[KeyValue::new("reason", "unsupported_format")]);
            return Ok(OptimizationOutcome::Skipped(SkipReason::UnsupportedFormat));
        }
        Err(reason) => {
            warn!(reason = ?reason, "Skipping optimization");
            tracing::Span::current().record("outcome", &format!("skipped_{:?}", reason));
            metrics
                .optimization_skipped
                .add(1, &[KeyValue::new("reason", format!("{:?}", reason))]);
            return Ok(OptimizationOutcome::Skipped(reason));
        }
    };

    if probed.pixel_count > config.max_pixels {
        warn!(
            pixels = probed.pixel_count,
            max_pixels = config.max_pixels,
            "Image exceeds pixel budget"
        );
        tracing::Span::current().record("outcome", "skipped_too_large");
        metrics
            .optimization_skipped
            .add(1, &[KeyValue::new("reason", "exceeds_pixel_budget")]);
        return Ok(OptimizationOutcome::Skipped(SkipReason::ExceedsPixelBudget));
    }

    if should_skip_for_quality(&probed) {
        debug!(bpp = probed.bytes_per_pixel, "Already optimized");
        tracing::Span::current().record("outcome", "skipped_optimized");
        metrics
            .optimization_skipped
            .add(1, &[KeyValue::new("reason", "already_optimized")]);
        return Ok(OptimizationOutcome::Skipped(SkipReason::AlreadyOptimized));
    }

    let strategy = match request.reference.unwrap_or(MediaReference::Post) {
        MediaReference::Category => Strategy::Category,
        MediaReference::User => Strategy::User,
        MediaReference::Post => Strategy::Post,
    };

    let decoded = match image::load_from_memory(request.bytes) {
        Ok(image) => image,
        Err(e) => {
            warn!(error = %e, "Failed to decode image");
            tracing::Span::current().record("outcome", "skipped_decode_failed");
            metrics
                .optimization_skipped
                .add(1, &[KeyValue::new("reason", "decode_failed")]);
            return Ok(OptimizationOutcome::Skipped(SkipReason::DecodeFailed));
        }
    };

    let characteristics = analyze_image(&decoded);
    debug!(
        has_alpha = characteristics.has_alpha,
        aspect_ratio = characteristics.aspect_ratio,
        min_dimension = characteristics.min_dimension,
        "Image characteristics analyzed"
    );

    let plan = strategy.build_plan(&decoded, &probed, &characteristics, config);
    debug!(variant_specs = plan.len(), "Optimization plan created");

    let mut result = OptimizationResult {
        replaced_original: false,
        original: OptimizedImage {
            bytes: request.bytes.clone(),
            mime_type: probed.mime.to_string(),
            extension: probed.extension.to_string(),
            width: probed.width,
            height: probed.height,
            label: VariantLabel::Original,
            quality: None,
        },
        variants: Vec::new(),
    };

    if !config.keep_original {
        if let Some(replacement) =
            strategy.reencode_original(&decoded, &probed, &characteristics, request.bytes.len())?
        {
            result.replaced_original = true;
            result.original = replacement;
        }
    }

    for spec in plan {
        let max_allowed = match spec.kind {
            ResizeKind::ExactSquare => characteristics.min_dimension,
            ResizeKind::FitWidth => probed.width,
        };

        if spec.width >= max_allowed {
            continue;
        }

        if let Some(variant) = encode_variant(&decoded, &spec)? {
            result.variants.push(variant);
        }
    }

    if !result.replaced_original && result.variants.is_empty() {
        debug!("No optimization applied");
        tracing::Span::current().record("outcome", "skipped_no_improvement");
        metrics
            .optimization_skipped
            .add(1, &[KeyValue::new("reason", "no_improvement")]);
        return Ok(OptimizationOutcome::Skipped(SkipReason::AlreadyOptimized));
    }

    let bytes_saved = if result.replaced_original {
        request.bytes.len() as i64 - result.original.bytes.len() as i64
    } else {
        0
    };

    info!(
        replaced_original = result.replaced_original,
        variant_count = result.variants.len(),
        bytes_saved,
        original_size = request.bytes.len(),
        "Optimization completed"
    );

    tracing::Span::current().record("outcome", "optimized");
    tracing::Span::current().record("bytes_saved", bytes_saved);
    tracing::Span::current().record("variant_count", result.variants.len() as i64);

    let duration = start.elapsed().as_millis() as f64;
    metrics.optimization_duration.record(duration, &[]);
    metrics.optimization_success.add(1, &[]);

    if bytes_saved > 0 {
        metrics.bytes_saved.add(bytes_saved as u64, &[]);
    }

    metrics
        .variants_generated
        .add(result.variants.len() as u64, &[]);

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

#[derive(Clone)]
struct VariantSpec {
    width: u32,
    format: TargetFormat,
    quality: u8,
    kind: ResizeKind,
    label: VariantLabel,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum TargetFormat {
    WebpLossless,
    Jpeg,
    Png,
}

#[derive(Clone, Copy)]
enum ResizeKind {
    FitWidth,
    ExactSquare,
}

impl Strategy {
    fn build_plan(
        &self,
        _image: &DynamicImage,
        probed: &ProbedImage,
        characteristics: &ImageCharacteristics,
        config: &OptimizerConfig,
    ) -> Vec<VariantSpec> {
        match self {
            Strategy::User => {
                let format = if characteristics.has_alpha {
                    TargetFormat::Png
                } else {
                    TargetFormat::Jpeg
                };

                [48, 96, 192, 384]
                    .into_iter()
                    .filter(|target| *target < characteristics.min_dimension)
                    .map(|width| VariantSpec {
                        width,
                        format,
                        quality: 82,
                        kind: ResizeKind::ExactSquare,
                        label: VariantLabel::Width(width),
                    })
                    .collect()
            }
            Strategy::Category => {
                let mut variants = Vec::new();
                let is_icon = characteristics.aspect_ratio <= 1.5;

                if is_icon {
                    for width in [128, 256] {
                        if width < characteristics.min_dimension {
                            variants.push(VariantSpec {
                                width,
                                format: TargetFormat::Png,
                                quality: 100,
                                kind: ResizeKind::FitWidth,
                                label: VariantLabel::Width(width),
                            });
                        }
                    }
                } else {
                    let format = if characteristics.has_alpha {
                        TargetFormat::Png
                    } else {
                        TargetFormat::Jpeg
                    };

                    for width in [640, 1280, 1920] {
                        if width < probed.width {
                            variants.push(VariantSpec {
                                width,
                                format,
                                quality: 80,
                                kind: ResizeKind::FitWidth,
                                label: VariantLabel::Width(width),
                            });
                        }
                    }
                }

                variants
            }
            Strategy::Post => {
                let mut variants = Vec::new();
                let format = if characteristics.has_alpha {
                    TargetFormat::Png
                } else {
                    TargetFormat::Jpeg
                };

                for width in [480, 768, 1024, 1600, 2048] {
                    if width < probed.width {
                        variants.push(VariantSpec {
                            width,
                            format,
                            quality: config.default_webp_quality,
                            kind: ResizeKind::FitWidth,
                            label: VariantLabel::Width(width),
                        });
                    }
                }

                if probed.width > 48 {
                    variants.push(VariantSpec {
                        width: 24,
                        format: TargetFormat::Jpeg,
                        quality: 40,
                        kind: ResizeKind::FitWidth,
                        label: VariantLabel::Lqip,
                    });
                }

                variants
            }
        }
    }

    fn reencode_original(
        &self,
        image: &DynamicImage,
        probed: &ProbedImage,
        characteristics: &ImageCharacteristics,
        original_size: usize,
    ) -> Result<Option<OptimizedImage>, OptimizationError> {
        let (format, quality, threshold) = match self {
            Strategy::User => {
                if characteristics.has_alpha {
                    return Ok(None);
                }
                (TargetFormat::Jpeg, 82, 0.05)
            }
            Strategy::Category => {
                let is_icon = characteristics.aspect_ratio <= 1.5;
                if is_icon || characteristics.has_alpha {
                    return Ok(None);
                }
                (TargetFormat::Jpeg, 80, 0.05)
            }
            Strategy::Post => {
                if characteristics.has_alpha {
                    return Ok(None);
                }
                (TargetFormat::Jpeg, 80, 0.05)
            }
        };

        let (buffer, mime, extension) = encode_to_format(image, format, quality)?;

        if !significant_reduction(original_size, buffer.len(), threshold) {
            return Ok(None);
        }

        let quality_opt = match format {
            TargetFormat::Jpeg => Some(quality),
            _ => None,
        };

        Ok(Some(OptimizedImage {
            bytes: Bytes::from(buffer),
            mime_type: mime.to_string(),
            extension: extension.to_string(),
            width: probed.width,
            height: probed.height,
            label: VariantLabel::Original,
            quality: quality_opt,
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
        ResizeKind::FitWidth => {
            let source_width = source.width().max(1);
            let source_height = source.height().max(1);
            if spec.width >= source_width {
                return Ok(None);
            }
            let ratio = source_height as f64 / source_width as f64;
            let target_height = (spec.width as f64 * ratio).round().max(1.0) as u32;
            source.resize(spec.width, target_height, FilterType::Lanczos3)
        }
    };

    let (buffer, mime, extension) = encode_to_format(&prepared, spec.format, spec.quality)?;

    let label = match &spec.label {
        VariantLabel::Width(_) => VariantLabel::Width(prepared.width()),
        VariantLabel::Lqip => VariantLabel::Lqip,
        VariantLabel::Original => VariantLabel::Width(prepared.width()),
    };

    let quality_opt = match spec.format {
        TargetFormat::Jpeg => Some(spec.quality),
        _ => None,
    };

    Ok(Some(OptimizedImage {
        bytes: Bytes::from(buffer),
        mime_type: mime.to_string(),
        extension: extension.to_string(),
        width: prepared.width(),
        height: prepared.height(),
        label,
        quality: quality_opt,
    }))
}

fn encode_to_format(
    image: &DynamicImage,
    format: TargetFormat,
    quality: u8,
) -> Result<(Vec<u8>, &'static str, &'static str), OptimizationError> {
    match format {
        TargetFormat::WebpLossless => {
            let mut buffer = Vec::new();
            let rgba = image.to_rgba8();
            codecs::webp::WebPEncoder::new_lossless(&mut buffer)
                .encode(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ExtendedColorType::Rgba8,
                )
                .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
            Ok((buffer, "image/webp", "webp"))
        }
        TargetFormat::Jpeg => {
            let mut buffer = Vec::new();
            let mut encoder = codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);
            encoder
                .encode_image(image)
                .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
            Ok((buffer, "image/jpeg", "jpg"))
        }
        TargetFormat::Png => {
            let mut buffer = Vec::new();
            let encoder = codecs::png::PngEncoder::new(&mut buffer);
            let rgba = image.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    ColorType::Rgba8.into(),
                )
                .map_err(|err| OptimizationError::EncodeFailed(err.to_string()))?;
            Ok((buffer, "image/png", "png"))
        }
    }
}

fn significant_reduction(original: usize, candidate: usize, threshold: f32) -> bool {
    if candidate >= original {
        return false;
    }

    let reduction = 1.0 - (candidate as f32 / original as f32);
    reduction >= threshold
}
