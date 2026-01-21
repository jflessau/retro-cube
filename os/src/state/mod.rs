pub mod event;

use crate::{message, weather};
use anyhow::Result;
pub use event::Event;
mod view;
use chrono::{DateTime, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use embedded_graphics::{
    mono_font::{
        MonoTextStyle,
        ascii::{FONT_6X9, FONT_9X15_BOLD},
    },
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use log::{error, info};
use view::View;

pub struct State {
    view: View,
    last_fetch: Option<DateTime<Utc>>,
    sleep: bool,

    time: NaiveTime,
    timezone: Tz,

    temperature: f32,
    relative_humidity_percent: f32,
    surface_pressure_hpa: f32,
    wind_speed_km_h: f32,
    wind_direction_deg: i32,
    rain_in_x_hours: Option<usize>,

    message: Option<String>,
    current_letter: (usize, DateTime<Utc>),
}

impl State {
    pub fn new() -> Self {
        let timezone = std::env::var("TIMEZONE")
            .unwrap_or("UTC".into())
            .parse::<Tz>()
            .unwrap_or(chrono_tz::UTC);

        info!("Using timezone: {}", timezone);

        State {
            view: View::Clock,
            last_fetch: None,
            sleep: false,

            time: Utc::now().time(),
            timezone,

            temperature: -0.0,
            relative_humidity_percent: 0.0,
            surface_pressure_hpa: 0.0,
            wind_speed_km_h: 0.0,
            wind_direction_deg: 0,
            rain_in_x_hours: None,

            message: None,
            current_letter: (0, Utc::now()),
        }
    }

    pub fn fetch(&mut self) {
        let seconds_since_last_fetch = self
            .last_fetch
            .map(|t| Utc::now().signed_duration_since(t).num_seconds())
            .unwrap_or(i64::MAX);

        let refetch_interval_seconds = std::env::var("REFETCH_INTERVAL_SECONDS")
            .unwrap_or("10".into())
            .parse::<i64>()
            .unwrap_or(10);

        println!("{seconds_since_last_fetch}, {refetch_interval_seconds}");
        if seconds_since_last_fetch > refetch_interval_seconds {
            info!("fetch data with interval {refetch_interval_seconds} s");
            self.last_fetch = Some(Utc::now());

            // weather

            match weather::fetch() {
                Ok(w) => {
                    self.temperature = w.temperature;
                    self.relative_humidity_percent = w.relative_humidity_percent as f32;
                    self.surface_pressure_hpa = w.surface_pressure_hpa;
                    self.wind_speed_km_h = w.wind_speed_km_h;
                    self.wind_direction_deg = w.wind_direction_deg as i32;
                    self.rain_in_x_hours = w.rain_in_x_hours;
                }
                Err(err) => {
                    error!("failed to fetch weather data: {err:?}");
                }
            }

            // message

            match message::fetch() {
                Ok(msg) => self.message = Some(msg),
                Err(err) => error!("failed to fetch message data: {err:?}"),
            }
        }
    }

    pub fn update<D>(&mut self, display: &mut D, event: Event)
    where
        D: DrawTarget<Color = BinaryColor>,
        D::Error: Send + Sync + core::fmt::Debug + 'static,
    {
        match event {
            Event::Tick => {
                std::thread::sleep(std::time::Duration::from_millis(10));
                self.time = Utc::now().time();
                self.fetch();

                if let Err(err) = self.render(display) {
                    error!("renderer failed: {err:?}");
                }
            }
            Event::NavigateDown => {
                self.view = self.view.next();
                self.current_letter = (0, Utc::now());
            }
            Event::NavigateUp => {
                self.view = self.view.previous();
                self.current_letter = (0, Utc::now());
            }
            Event::ToggleSleep => {
                info!("toggle sleep mode");
                self.sleep = !self.sleep;
            }
        }
    }

    fn render<D>(&mut self, display: &mut D) -> Result<()>
    where
        D: DrawTarget<Color = BinaryColor>,
        D::Error: Send + Sync + core::fmt::Debug + 'static,
    {
        if self.sleep {
            display
                .clear(BinaryColor::Off)
                .map_err(|e| anyhow::anyhow!("clear failed: {:?}", e))?;
            return Ok(());
        }

        match &self.view {
            View::Clock => {
                // frame

                Rectangle::new(Point::new(0, 0), Size::new(128, 64))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
                Rectangle::new(Point::new(4, 4), Size::new(120, 56))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                // text

                let local_time = Utc::now().with_timezone(&self.timezone);
                let date = format!("{}", local_time.date_naive().format("%Y-%m-%d"));
                Text::new(
                    &date,
                    Point::new(32, 16),
                    MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                let time = format!("{}", local_time.time().format("%H:%M"));
                Text::new(
                    &time,
                    Point::new(40, 35),
                    MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                // second indicator

                let start = Point::new(10, 48);
                let length = 108;
                let height = 1;
                Line::new(start, Point::new(start.x + length, start.y))
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, height))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
                let seconds = (local_time.second() + 1) as f32 / 60.0;
                let x = start.x + (length as f32 * seconds) as i32;
                Circle::with_center(Point::new(x, start.y), height + 4)
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
            }
            View::Weather => {
                // frame

                Rectangle::new(Point::new(0, 0), Size::new(128, 64))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
                Rectangle::new(Point::new(2, 2), Size::new(124, 60))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                let x = 6;
                let row_one_y = 12;
                let row_two_y = 24;
                let row_three_y = 44;
                let row_four_y = 56;

                Text::new(
                    &format!(
                        "{:.0}C  {:.0}% {:.0} hpa",
                        self.temperature, self.relative_humidity_percent, self.surface_pressure_hpa
                    ),
                    Point::new(x, row_one_y),
                    MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                Text::new(
                    &format!(
                        "{:.0}km/h ({})",
                        self.wind_speed_km_h,
                        match self.wind_direction_deg {
                            d if (337..=360).contains(&d) || (0..=22).contains(&d) => "N",
                            d if (23..=67).contains(&d) => "NE",
                            d if (68..=112).contains(&d) => "E",
                            d if (113..=157).contains(&d) => "SE",
                            d if (158..=202).contains(&d) => "S",
                            d if (203..=247).contains(&d) => "SW",
                            d if (248..=292).contains(&d) => "W",
                            d if (293..=336).contains(&d) => "NW",
                            _ => "N/A",
                        },
                    ),
                    Point::new(x, row_two_y),
                    MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                Line::new(Point::new(x, row_two_y + 8), Point::new(120, row_two_y + 8))
                    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                Text::new(
                    "Precipitation:",
                    Point::new(x, row_three_y),
                    MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                Text::new(
                    &(match self.rain_in_x_hours {
                        Some(hours) => format!("{hours}h"),
                        None => "---".to_string(),
                    })
                    .to_string(),
                    Point::new(x, row_four_y),
                    MonoTextStyle::new(&FONT_6X9, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
            }
            View::Mailbox => {
                // frame

                Rectangle::new(Point::new(0, 0), Size::new(128, 8))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
                Rectangle::new(Point::new(0, 56), Size::new(128, 8))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                // text

                let text = self.message.as_deref().unwrap_or("No message available.");
                let ms_since = Utc::now()
                    .signed_duration_since(self.current_letter.1)
                    .num_milliseconds();

                if (self.current_letter.0 != 0 && ms_since > 200) || ms_since > 1500 {
                    let idx = (self.current_letter.0 + 1).min(text.len());
                    if idx >= text.len() {
                        self.current_letter = (0, Utc::now());
                    } else {
                        self.current_letter = (idx, Utc::now());
                    }
                }

                let text_slice = &text[self.current_letter.0..text.len().min(20)];

                Text::new(
                    "Message:",
                    Point::new(6, 26),
                    MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                Text::new(
                    text_slice,
                    Point::new(6, 44),
                    MonoTextStyle::new(&FONT_9X15_BOLD, BinaryColor::On),
                )
                .draw(display)
                .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;

                // cutoff

                Rectangle::new(Point::new(122, 32), Size::new(128, 15))
                    .into_styled(PrimitiveStyle::with_fill(BinaryColor::Off))
                    .draw(display)
                    .map_err(|e| anyhow::anyhow!("draw failed: {:?}", e))?;
            }
        }

        Ok(())
    }
}
