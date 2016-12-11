//! Statistics for created contigs and collections.
use std::fmt::Display;
/// Create stats for contigs.
pub trait Stats<T: Display> {
    /// Gets stats.
    fn stats(&self) -> T;
    /// Prints stats.
    fn print_stats(&self) {
        print!("{}", self.stats());
    }
}

mod contigs;
mod collections;
pub use self::collections::{Opt, CollectionStats, Counts};
pub use self::contigs::ContigsStats;
