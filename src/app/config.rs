use bon::Builder;

#[derive(Builder, smart_default::SmartDefault, Debug, Clone)]
pub struct AppConfig {
    #[default(4.0)]
    #[builder(default = 4.0)]
    pub tick_rate: f64,

    #[default(10.0)]
    #[builder(default = 10.0)]
    pub frame_rate: f64,
}
