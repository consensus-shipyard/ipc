pub use frc42_hasher as hasher;
pub use frc42_hasher::hash;
pub use frc42_macros::method_hash;

pub mod match_method;
pub mod message;

#[cfg(test)]
mod tests {}
