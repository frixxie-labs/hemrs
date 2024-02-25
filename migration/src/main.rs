use anyhow::{anyhow, Error};
use chrono::DateTime;
use env_data::EnvDataEntry;
use rayon::prelude::*;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EnvDataEntryOld {
    ts: i64,
    room: String,
    temperature: f64,
    humidity: f64,
}

impl TryFrom<EnvDataEntryOld> for EnvDataEntry {
    type Error = Error;

    fn try_from(value: EnvDataEntryOld) -> Result<Self, Self::Error> {
        Ok(EnvDataEntry {
            ts: DateTime::from_timestamp(value.ts, 0).ok_or(anyhow!("Failed to parse ts"))?,
            room: value.room,
            temperature: value.temperature as f32,
            humidity: value.humidity as f32,
        })
    }
}

fn get_data_from_src(client: &Client, src_uri: &str) -> Vec<EnvDataEntry> {
    let resp = client.get(src_uri).send().unwrap();
    let text = resp.text().unwrap();
    let data: Vec<EnvDataEntryOld> = serde_json::from_str(&text).unwrap();
    data.into_par_iter()
        .map(|e| EnvDataEntry::try_from(e).unwrap())
        .collect()
}

fn post_data_to_dest(client: Client, data: EnvDataEntry, dest_uri: &str) -> Result<(), Error> {
    client
        .post(dest_uri)
        .json(&data)
        .send()?
        .error_for_status()?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let bclient = Client::new();
    let src_uri = "http://server:65534/";
    let dest_uri = "http://server:65533/entry";

    println!("Getting data from {}", src_uri);
    let data = get_data_from_src(&bclient, src_uri);

    println!("Start posting to dest {}", dest_uri);
    data.into_par_iter().for_each(|v| {
        let _ = post_data_to_dest(bclient.clone(), v, dest_uri);
    });
    println!("Finished posting to dest {}", dest_uri);
    Ok(())
}
