use std::process::Command;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use log::info;

lazy_static! {
    static ref SX1302_RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref SX1302_POWER_EN: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref SX1261_RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref AD5338R_RESET: Mutex<Option<LineHandle>> = Mutex::new(None);
    static ref RESET_COMMANDS: Mutex<Option<Vec<(String, Vec<String>)>>> = Mutex::new(None);
}

#[derive(Default)]
pub struct Configuration {
    pub sx130x_reset: Option<(String, u32)>,
    pub sx1302_power_en: Option<(String, u32)>,
    pub sx1261_reset: Option<(String, u32)>,
    pub ad5338r_reset: Option<(String, u32)>,
    pub reset_commands: Option<Vec<(String, Vec<String>)>>,
}

pub fn setup_pins(config: Configuration) -> Result<()> {
    if let Some(sx1302_reset) = config.sx130x_reset {
        info!(
            "Configuring reset pin, dev: {}, pin: {}",
            sx1302_reset.0, sx1302_reset.1
        );

        let mut chip = Chip::new(sx1302_reset.0)?;
        let line = chip.get_line(sx1302_reset.1)?;
        let mut sx1302_reset = SX1302_RESET.lock().unwrap();
        *sx1302_reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "sx130x_reset")?);
    }

    if let Some(sx1302_power_en) = config.sx1302_power_en {
        info!(
            "Configuring sx1302 power enable pin, dev: {}, pin: {}",
            sx1302_power_en.0, sx1302_power_en.1
        );

        let mut chip = Chip::new(sx1302_power_en.0)?;
        let line = chip.get_line(sx1302_power_en.1)?;
        let mut sx1302_power_en = SX1302_POWER_EN.lock().unwrap();
        *sx1302_power_en = Some(line.request(LineRequestFlags::OUTPUT, 0, "sx1302_power_en")?);
    }

    if let Some(sx1261_reset) = config.sx1261_reset {
        info!(
            "Configuring sx1261 reset pin, dev: {}, pin: {}",
            sx1261_reset.0, sx1261_reset.1
        );

        let mut chip = Chip::new(sx1261_reset.0)?;
        let line = chip.get_line(sx1261_reset.1)?;
        let mut sx1261_reset = SX1261_RESET.lock().unwrap();
        *sx1261_reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "sx1261_reset")?);
    }

    if let Some(ad5338r_reset) = config.ad5338r_reset {
        info!(
            "Configuring ad5338r reset pin, dev: {}, pin: {}",
            ad5338r_reset.0, ad5338r_reset.1
        );

        let mut chip = Chip::new(ad5338r_reset.0)?;
        let line = chip.get_line(ad5338r_reset.1)?;
        let mut ad5338r_reset = AD5338R_RESET.lock().unwrap();
        *ad5338r_reset = Some(line.request(LineRequestFlags::OUTPUT, 0, "ad5338r_reset")?);
    }

    if let Some(reset_commands) = config.reset_commands {
        info!("Configuring raw reset commands");

        let mut reset_commands_m = RESET_COMMANDS.lock().unwrap();
        *reset_commands_m = Some(reset_commands);
    }

    Ok(())
}

pub fn reset() -> Result<()> {
    let sx1302_power_en = SX1302_POWER_EN.lock().unwrap();
    if sx1302_power_en.is_some() {
        let sx1302_power_en = sx1302_power_en.as_ref().unwrap();

        info!("Enabling concentrator power");

        sx1302_power_en.set_value(1)?;
        sleep(Duration::from_millis(100));
    }

    let sx1302 = SX1302_RESET.lock().unwrap();
    if sx1302.is_some() {
        let sx1302 = sx1302.as_ref().unwrap();

        info!("Triggering sx1302 reset");

        sx1302.set_value(1)?;
        sleep(Duration::from_millis(100));
        sx1302.set_value(0)?;
        sleep(Duration::from_millis(100));
    }

    let sx1261_reset = SX1261_RESET.lock().unwrap();
    if sx1261_reset.is_some() {
        let sx1261_reset = sx1261_reset.as_ref().unwrap();

        info!("Triggering sx1261 reset");

        sx1261_reset.set_value(0)?;
        sleep(Duration::from_millis(100));
        sx1261_reset.set_value(1)?;
        sleep(Duration::from_millis(100));
    }

    let ad5338r_reset = AD5338R_RESET.lock().unwrap();
    if ad5338r_reset.is_some() {
        let ad5338r_reset = ad5338r_reset.as_ref().unwrap();

        info!("Triggering AD5338R reset");
        ad5338r_reset.set_value(0)?;
        sleep(Duration::from_millis(100));
        ad5338r_reset.set_value(1)?;
        sleep(Duration::from_millis(100));
    }

    let reset_commands = RESET_COMMANDS.lock().unwrap();
    if reset_commands.is_some() {
        let reset_commands = reset_commands.as_ref().unwrap();

        for (cmd, args) in reset_commands {
            info!(
                "Executing reset command, command: {}, args: {:?}",
                cmd, args
            );

            Command::new(cmd).args(args).output()?;
            sleep(Duration::from_millis(100));
        }
    }

    Ok(())
}
