use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[warn(non_snake_case)]
pub struct CronListResponse {
    pub cron: String,
    pub description: String,
}

impl CronListResponse {
    pub fn new(cron: String, description: String) -> CronListResponse {
        CronListResponse { cron, description }
    }
}
