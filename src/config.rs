pub mod body_limits {
    pub const DEFAULT: usize = 64 * 1024; // 64 KiB
    pub const POST: usize = 256 * 1024; // 256 KiB
    pub const MEDIA: usize = 2 * 1024 * 1024; // 2 MiB
}
