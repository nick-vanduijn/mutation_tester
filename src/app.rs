use crate::config::AppConfig;
use crate::database::DatabasePool;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub db: DatabasePool,
    pub config: AppConfig,
}
