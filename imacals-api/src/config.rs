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
    }
});
