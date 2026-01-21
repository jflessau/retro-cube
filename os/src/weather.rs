use anyhow::{Context, Result, bail};
use chrono::{Timelike, Utc};
use log::trace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub latitude: f64,
    pub longitude: f64,
    pub generationtime_ms: f64,
    pub utc_offset_seconds: i32,
    pub timezone: String,
    pub timezone_abbreviation: String,
    pub elevation: f64,
    pub hourly_units: HourlyUnits,
    pub hourly: HourlyData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyUnits {
    pub time: String,
    pub temperature_2m: String,
    pub relative_humidity_2m: String,
    pub rain: String,
    pub wind_speed_10m: String,
    pub wind_direction_10m: String,
    pub surface_pressure: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyData {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
    pub relative_humidity_2m: Vec<i32>,
    pub rain: Vec<f64>,
    pub snowfall: Vec<f64>,
    pub wind_speed_10m: Vec<f64>,
    pub wind_direction_10m: Vec<i32>,
    pub showers: Vec<f64>,
    pub surface_pressure: Vec<f64>,
}

pub struct Weather {
    pub temperature: f32,
    pub relative_humidity_percent: u8,
    pub surface_pressure_hpa: f32,

    pub wind_speed_km_h: f32,
    pub wind_direction_deg: f32,
    pub rain_in_x_hours: Option<usize>,
}

pub fn fetch() -> Result<Weather> {
    let date = Utc::now().date_naive();
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude=52.52&longitude=13.41&hourly=temperature_2m,relative_humidity_2m,rain,snowfall,wind_speed_10m,wind_direction_10m,showers,surface_pressure&visibility&start_date=2026-01-04&end_date={date}"
    );
    let response = reqwest::blocking::get(url).context("fails to fetch weather data")?;
    let data: WeatherData = response.json().context("fails to parse weather data")?;
    trace!("weather data {:#?}", data);

    let hour = Utc::now().hour() as usize;

    if hour >= data.hourly.time.len() {
        bail!("hour index out of bounds");
    }

    let next_rain_hour = data
        .hourly
        .rain
        .iter()
        .enumerate()
        .find(|&(n, &rain)| n >= hour && rain > 0.0)
        .map(|(index, _)| index);

    Ok(Weather {
        temperature: data.hourly.temperature_2m[hour] as f32,
        relative_humidity_percent: data.hourly.relative_humidity_2m[hour] as u8,
        surface_pressure_hpa: data.hourly.surface_pressure[hour] as f32,
        wind_speed_km_h: data.hourly.wind_speed_10m[hour] as f32,
        wind_direction_deg: data.hourly.wind_direction_10m[hour] as f32,
        rain_in_x_hours: next_rain_hour.map(|index| index - hour),
    })
}
