use std::{net::TcpStream, time::Instant};

use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

use crate::prelude::*;

//TODO: tev_client doesn't do vector graphics so maybe this should be custom and show recently updated tiles or something shrug.
//TODO: don't send the entire pixel buffer every time

/// Wraps over an optional Tev client.
#[derive(Debug)]
pub struct TevReporter {
    client: Option<TevClient>,
    last_report: Instant,

    pixels: Vec<f32>,

    extent: UExtent2,
}

const REPORT_DURATION: u128 = 100;

impl TevReporter {
    pub fn new(no_tev: bool, extent: UExtent2) -> Self {
        let error_self = Self {
            client: None,
            last_report: Instant::now(),
            pixels: vec![],
            // rendered_bounds: UBounds2::default(),
            extent: UExtent2::default(),
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
            pixels: vec![0.0; 3 * extent.x as usize * extent.y as usize],
            // rendered_bounds: UBounds2::default(),
            extent,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn should_report(&self) -> bool {
        // return false;
        self.is_connected()
            && Instant::now().duration_since(self.last_report).as_millis() > REPORT_DURATION
    }

    pub fn update_pixels(&mut self, bounds: UBounds2, pixels: Vec<f32>, force: bool) {
        if let Some(client) = &mut self.client {
            for y in bounds.min.y..bounds.max.y {
                for x in bounds.min.x..bounds.max.x {
                    let i_s = (x + y * self.extent.x) as usize * 3;
                    let i_o =
                        ((x - bounds.min.x) + (y - bounds.min.y) * bounds.width()) as usize * 3;
                    self.pixels[i_s] = pixels[i_o];
                    self.pixels[i_s + 1] = pixels[i_o + 1];
                    self.pixels[i_s + 2] = pixels[i_o + 2];
                }
            }
            let now: Instant = Instant::now();

            if force || now.duration_since(self.last_report).as_millis() > REPORT_DURATION {
                self.last_report = Instant::now();
                if let Err(err) = client.send(PacketUpdateImage {
                    image_name: "[Luminiferous Render]",
                    grab_focus: false,
                    channel_names: &["R", "G", "B"],
                    channel_offsets: &[0, 1, 2],
                    channel_strides: &[3, 3, 3],

                    x: 0,
                    y: 0,
                    width: self.extent.x,
                    height: self.extent.y,
                    data: &self.pixels,
                }) {
                    errorln!("Display server disconnected: {err}. No more packets will be sent.");
                    self.client = None;
                }
            }
        }
    }
}
