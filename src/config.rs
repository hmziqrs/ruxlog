pub mod body_limits {
    pub const DEFAULT: usize = 2 * 1024 * 1024; // 2 MiB
    pub const POST: usize = 256 * 1024; // 256 KiB
    pub const MEDIA: usize = 2 * 1024 * 1024; // 2 MiB
}
