use thiserror::Error;

pub type DbResult<T> = Result<T, crate::errors::DbError>;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database file not found, tried looking at {not_found_at}")]
    NotFound { not_found_at: String },
    #[error("Seaorm Error")]
    SeaOrm(#[from] sea_orm::error::DbErr),
}
