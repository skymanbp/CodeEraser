//! The language and product definitions the core holds (plan v2.32 step
//! 1; design booklet docs/reference/authority-track.md §4): `tables/1`
//! answers them as one package, proto 7.7.0. This side does not read
//! that package yet — the consumers switch, and the definition text
//! here is deleted, in step 2. Until then `native` renders today's
//! Rust tables in the package's shape, so the equivalence gate can
//! hold the core's transcription to the text it was taken from.

pub mod native;
