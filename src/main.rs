mod cmap;
mod led;

use cmap::COLORMAP;
use led::{LEDConfig, LEDControler};

use std::env;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{buffer::SamplesBuffer, Device, OutputStream};

use std::collections::HashMap;
use std::fs;
use std::io;

use anyhow::{anyhow, Context, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use serde::Deserialize;

use librespot::{
    audio::AudioPacket,
    core::{
        authentication::Credentials, cache::Cache, config::SessionConfig, session::Session,
        spotify_id::SpotifyId,
    },
    playback::{audio_backend, config::PlayerConfig, player::Player},
};

const DATA_DIR: &str = "/etc/lightbox/";
const SPOTIFY_SAMPLE_RATE: u32 = 44100;

#[derive(Deserialize)]
struct AppConfig {
    pub spotify_username: String,
    pub spotify_password: String,
    pub audio_device: Option<String>,
}

impl AppConfig {
    fn load(path: &Path) -> Result<AppConfig> {
        Figment::new()
            .merge(Toml::file(path))
            .extract()
            .context("Erro loading config file")
    }
}

fn select_device(device_name: Option<String>) -> Result<Option<Device>> {
    let host = cpal::default_host();
    let devices: Vec<_> = host
        .devices()
        .with_context(|| "Error listing devices")?
        .map(Box::new)
        .collect();

    if let Some(target_name) = &device_name {
        for device in devices {
            let name = device.name().unwrap_or_else(|_| "".to_owned());
            if name.contains(target_name.as_str()) {
                println!("Selected device: {}", name);
                return Ok(Some(Device::from(*device)));
            }
        }
        println!("No device named {} found.", target_name);
    } else {
        println!("Available devices:");
        for device in devices.iter() {
            let name = device.name().context("Error getting device name")?;
            print!("'{}' configs: [", name);

            match device.supported_output_configs() {
                Ok(configs) => {
                    println!("");
                    for config in configs {
                        if config.channels() > 5 {
                            println!("  ...");
                            break;
                        }
                        println!(
                            "  {{nchans: {}, s_rate: {}-{}, sample_type: {:?}}},",
                            config.channels(),
                            config.min_sample_rate().0,
                            config.max_sample_rate().0,
                            config.sample_format()
                        );
                    }
                    println!("]");
                }
                Err(_) => println!(" Could not retrieve configs ]"),
            }
        }
    }

    Ok(None)
}

struct Distributer {
    output_dev: rodio::Sink,
    led_controller: LEDControler,
}

impl Distributer {
    fn new(output_dev: rodio::Sink, led_controller: LEDControler) -> Self {
        Distributer {
            output_dev,
            led_controller,
        }
    }
}

impl audio_backend::Sink for Distributer {
    fn start(&mut self) -> io::Result<()> {
        println!("Starting playback");
        Ok(())
    }

    fn stop(&mut self) -> io::Result<()> {
        println!("Stopping playback");
        Ok(())
    }

    fn write(&mut self, data: &AudioPacket) -> io::Result<()> {
        println!("Got {} bytes of data", data.samples().len());
        let source = SamplesBuffer::new(2, SPOTIFY_SAMPLE_RATE, data.samples());
        println!("Feeding audio");
        self.output_dev.append(source);
        println!("Feeding LEDs");
        self.led_controller.feed_samples(data.samples());

        // Chunk sizes seem to be about 256 to 3000 ish items long.
        // Assuming they're on average 1628 then a half second buffer is:
        // 44100 elements --> about 27 chunks
        while self.output_dev.len() > 26 {
            // sleep and wait for rodio to drain a bit
            print!("{},", self.output_dev.len());
            thread::sleep(Duration::from_millis(10));
        }
        println!("Done sleep");
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        println!("Error running lightrbox: {:?}", e);
    }
}

async fn run() -> Result<()> {
    let led_config = LEDConfig {
        led_count: 144,
        average_count: 1,
        color_map: &COLORMAP,
    };

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} TRACK", args[0]);
        return Ok(());
    }
    let data_dir = PathBuf::from(DATA_DIR);
    fs::create_dir_all(&data_dir).context("Error creating config directory")?;

    let config = AppConfig::load(&data_dir.join("config.toml"))?;
    let credentials = Credentials::with_password(config.spotify_username, config.spotify_password);

    let track = SpotifyId::from_base62(&args[1]).expect("Unable to decode spotify track ID");

    // Spotify connect
    println!("Connecting ..");
    let session_config = SessionConfig::default();
    let player_config = PlayerConfig::default();
    let cache = Cache::new(
        Some(&data_dir.join("info_cache")),
        Some(&data_dir.join("audio_cache")),
    )
    .context("Error creating spotify cache")?;
    let session = Session::connect(session_config, credentials, Some(cache)).await?;

    // Open output stream
    let device = match select_device(config.audio_device)? {
        Some(d) => d,
        None => {
            println!("No device selected. Exiting");
            return Ok(());
        }
    };
    let (_stream, stream_handle) =
        OutputStream::try_from_device(&device).context("Error opening output stream")?;
    let sink = rodio::Sink::try_new(&stream_handle).context("Error opening output sink")?;

    let (mut player, _) = Player::new(player_config, session.clone(), None, move || {
        Box::new(Distributer::new(sink, LEDControler::new(led_config)))
    });

    player.load(track, true, 0);

    println!("Playing...");
    player.await_end_of_track().await;

    println!("Done");
    Ok(())
}
