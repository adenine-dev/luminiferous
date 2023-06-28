use std::{net::TcpStream, process::Command};

use tev_client::{PacketCreateImage, PacketUpdateImage, TevClient};

use crate::maths::{Extent2, UBounds2, UExtent2, UPoint2};

//TODO: tev_client doesn't do vector graphics so maybe this should be custom and show recently updated tiles or something shrug.

/// Wraps over an optional Tev client.
#[derive(Debug)]
pub struct TevReporter {
    client: Option<TevClient>,
}

impl TevReporter {
    pub fn new(mut no_tev: bool) -> Self {
        no_tev = true;
        if no_tev {
            return Self { client: None };
        }

        //TODO: allow other creation methods (wrap with tcp stream)
        // let mut c = Command::new("tev");
        // c.arg("--hostname=127.0.0.1:14159");
        // let client = TevClient::spawn(c);
        // let client = TevClient::spawn_path_default();
        let client = TcpStream::connect("127.0.0.1:14158").map(TevClient::wrap);

        if client.is_err() {
            println!("[WARN]: Tev not connected: {}", client.unwrap_err());

            Self { client: None }
        } else {
            println!("[INFO]: Tev connected");
            Self {
                client: client.ok(),
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    pub fn create_image(&mut self, extent: UExtent2) {
        if let Some(client) = &mut self.client {
            if let Err(err) = client.send(PacketCreateImage {
                image_name: "[Luminiferous Render]",
                grab_focus: false,
                width: extent.x,
                height: extent.y,
                channel_names: &["R", "G", "B"],
            }) {
                println!("[WARN]: Tev disconnected: {err}");
                self.client = None;
            }
        }
    }

    pub fn update_pixels(&mut self, bounds: UBounds2, data: &Vec<f32>) {
        if let Some(client) = &mut self.client {
            if let Err(err) = client.send(PacketUpdateImage {
                image_name: "[Luminiferous Render]",
                grab_focus: false,
                channel_names: &["R", "G", "B"],
                channel_offsets: &[0, 1, 2],
                channel_strides: &[3, 3, 3],
                x: bounds.min.x,
                y: bounds.min.y,
                width: bounds.max.x - bounds.min.x,
                height: bounds.max.y - bounds.min.y,
                data,
            }) {
                println!("[WARN]: Tev disconnected: {err}. No more packets will be sent.");
                dbg!(bounds);
                dbg!(data);
                self.client = None;
            }
        }
    }
}
