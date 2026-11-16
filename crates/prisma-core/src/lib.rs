//! Logique métier de Prisma, indépendante de toute interface.

pub mod chime;
pub mod domain;
pub mod library;
pub mod playstation;
pub mod steam;
pub mod support;
pub mod switch;
pub mod theme;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
