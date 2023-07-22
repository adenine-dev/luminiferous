use std::{net::TcpStream, time::Instant};

use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

use crate::prelude::*;

//TODO: tev_client doesn't do vector graphics so maybe this should be custom and show recently updated tiles or something shrug.

#[derive(Debug)]
struct ScheduledUpdate {
    pub bounds: UBounds2,
    pub pixels: Vec<f32>,
}

/// Wraps over an optional Tev client. Note the pixels displayed are *not* the same as the final render,
/// they contain artifacts along tile edges. However it is good enough for a visual preview that the
/// render is doing what you want it to.
#[derive(Debug)]
pub struct TevReporter {
    client: Option<TevClient>,
    last_report: Instant,

    scheduled_updates: Vec<ScheduledUpdate>,
}

const REPORT_DURATION: u128 = 100;

impl TevReporter {
    pub fn new(no_tev: bool, extent: UExtent2) -> Self {
        // let no_tev = true;
        let error_self = Self {
            client: None,
            last_report: Instant::now(),

            scheduled_updates: vec![],
        };

        if no_tev {
            return error_self;
        }

        //TODO: allow other creation methods (wrap with tcp stream)

        // let mut c = Command::new("tev");
        // c.arg("--hostname=127.0.0.1:14159");
        // let client = TevClient::spawn(c);
        // let client = TevClient::spawn_path_default();
        let client = TcpStream::connect("127.0.0.1:14158").map(TevClient::wrap);

        if client.is_err() {
            warnln!("Display server not connected: {}", client.unwrap_err());

            return error_self;
        }

        let mut client = client.unwrap();
        infoln!("Display server connected");
        if let Err(err) = client.send(PacketCreateImage {
            image_name: "[Luminiferous Render]",
            grab_focus: false,
            width: extent.x,
            height: extent.y,
            channel_names: &["R", "G", "B"],
        }) {
            warnln!("Display server disconnected on image create: {err}");
            return error_self;
        }

        Self {
            client: Some(client),
            last_report: Instant::now(),
            scheduled_updates: vec![],
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn should_report(&self) -> bool {
        self.is_connected()
            && Instant::now().duration_since(self.last_report).as_millis() > REPORT_DURATION
    }

    pub fn update_pixels(&mut self, bounds: UBounds2, pixels: Vec<f32>, force: bool) {
        if self.client.is_some() {
            let now = Instant::now();

            self.scheduled_updates
                .push(ScheduledUpdate { bounds, pixels });
            if force || now.duration_since(self.last_report).as_millis() > REPORT_DURATION {
                self.clear_updates()
            }
        }
    }

    pub fn clear_updates(&mut self) {
        if let Some(client) = &mut self.client {
            for update in self.scheduled_updates.iter() {
                let extent = update.bounds.extent();

                self.last_report = Instant::now();
                if let Err(err) = client.send(PacketUpdateImage {
                    image_name: "[Luminiferous Render]",
                    grab_focus: false,
                    channel_names: &["R", "G", "B"],
                    channel_offsets: &[0, 1, 2],
                    channel_strides: &[3, 3, 3],

                    x: update.bounds.min.x,
                    y: update.bounds.min.y,
                    width: extent.x,
                    height: extent.y,
                    data: &update.pixels,
                }) {
                    errorln!("Display server disconnected: {err}. No more packets will be sent.");
                    self.client = None;
                    break;
                }
            }

            self.scheduled_updates.clear();
        }
    }
}
