use std::time::Duration;

use anyhow::Result;
use dht11::Dht11;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::PinDriver, peripherals::Peripherals},
    http::{
        client::{EspHttpConnection, Request},
        Method,
    },
    nvs::EspDefaultNvsPartition,
    sys,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::{info, warn};

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let pin15 = PinDriver::input_output_od(peripherals.pins.gpio15)?;

    let mut dht11 = Dht11::new(pin15);

    let mut delay = esp_idf_svc::hal::delay::Delay::default();

    let mut current_temperature = 0;
    let mut current_humidity = 0;
    loop {
        match dht11.perform_measurement(&mut delay) {
            Ok(res) => {
                if current_temperature != res.temperature || current_humidity != current_humidity {
                    current_temperature = res.temperature;
                    current_humidity = res.humidity;
                    let dht11_entry = sensors::Dht11::new(
                        "Testing".to_string(),
                        (f32::from(res.temperature) as f32) / 10.0,
                        (f32::from(res.humidity) as f32) / 10.0,
                    );
                    info!("Current reading: {}", dht11_entry);
                    match connect_wifi(&mut wifi) {
                        Ok(_) => loop {
                            match post_reading(dht11_entry.clone()) {
                                Ok(_) => break,
                                Err(e) => {
                                    warn!("Failed with error: {}", e);
                                    continue
                                },
                            };
                        },
                        Err(_) => {
                            warn!("Failed to connect to wifi");
                        }
                    };
                    wifi.disconnect()?;
                }
            }
            Err(e) => warn!("Failed to read! error: {:?}", e),
        }

        std::thread::sleep(Duration::from_secs(120));
    }
}

fn post_reading(payload: sensors::Dht11) -> Result<()> {
    let mut connection = EspHttpConnection::new(&Default::default())?;
    let request_payload = serde_json::to_string(&payload)?.into_bytes();
    let headers = [
        ("content-type", "application/json"),
        ("content-length", &format!("{}", request_payload.len())),
    ];
    connection.initiate_request(Method::Post, "http://pimaster.lan:65534/", &headers)?;
    let mut request = Request::wrap(connection);
    request.write(&request_payload)?;
    request.submit()?;
    info!("Finished sending request returning...");
    Ok(())
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2WPA3Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
