use std::env;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub app_url: String,
    pub app_port: u16,
    pub cpu_count: usize,
    pub database_url: String,
    pub app_secret: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
}

pub static ENV: LazyLock<Config> = LazyLock::new(|| {
    // 3032 everywhere else too (Dockerfile EXPOSE, compose, the dashboard's /api proxy target).
    let port = env::var("APP_PORT").unwrap_or("3032".to_string());
    let cpu_count = env::var("CPU_COUNT").unwrap_or(num_cpus::get().to_string());

    Config {
        app_env: env::var("APP_ENV").unwrap_or("local".to_string()),
        app_url: env::var("APP_URL").unwrap_or("http://localhost".to_string()),
        app_port: port.parse::<u16>().unwrap_or_else(|_| panic!("ERROR: APP_PORT must be a valid u16 number")),
        cpu_count: cpu_count.parse::<usize>().unwrap_or_else(|_| panic!("ERROR: CPU_COUNT must be a valid number")),
        database_url: env::var("DATABASE_URL").unwrap_or_else(|_| panic!("ERROR: DATABASE_URL must be set")),
        app_secret: env::var("APP_SECRET").unwrap_or_else(|_| panic!("ERROR: APP_SECRET must be set")),
        s3_endpoint: env::var("S3_ENDPOINT").unwrap_or_default(),
        s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| panic!("ERROR: S3_BUCKET must be set")),
        s3_access_key: env::var("S3_ACCESS_KEY").unwrap_or_else(|_| panic!("ERROR: S3_ACCESS_KEY must be set")),
        s3_secret_key: env::var("S3_SECRET_KEY").unwrap_or_else(|_| panic!("ERROR: S3_SECRET_KEY must be set")),
        s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
    }
});
