//
// SPDX-FileCopyrightText: The Android Open Source Project
// SPDX-FileCopyrightText: Shirayuki39
// SPDX-License-Identifier: Apache-2.0
//
//! This module implements the ILights AIDL interface

use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::sync::Mutex;

use log::{error, info};

use android_hardware_light::aidl::android::hardware::light::{
    HwLight::HwLight, HwLightState::HwLightState, ILights::ILights, LightType::LightType,
};

use binder::{ExceptionCode, Interface, Status};

const LCD_LED_PATH: &str = "/sys/class/leds/lcd-backlight/";
const BRIGHTNESS_FILE: &str = "brightness";
const MAX_BRIGHTNESS_FILE: &str = "max_brightness";

struct Light {
    hw_light: HwLight,
    state: HwLightState,
}

/// Defined so we can implement the ILights AIDL interface.
pub struct LightsService {
    lights: Mutex<HashMap<i32, Light>>,
}

impl Interface for LightsService {}

impl LightsService {
    fn new(hw_lights: impl IntoIterator<Item = HwLight>) -> Self {
        let mut lights_map = HashMap::new();

        for hw_light in hw_lights {
            lights_map.insert(
                hw_light.id,
                Light { hw_light, state: Default::default() },
            );
        }

        Self { lights: Mutex::new(lights_map) }
    }
}

impl Default for LightsService {
    fn default() -> Self {
        let backlight = HwLight {
            id: LightType::BACKLIGHT.0 as i32,
            ordinal: 0,
            r#type: LightType::BACKLIGHT,
        };
        Self::new(vec![backlight])
    }
}

impl ILights for LightsService {
    fn setLightState(&self, id: i32, state: &HwLightState) -> binder::Result<()> {
        info!("Lights setting state for id={} to color {:x}", id, state.color);

        let mut lights = self.lights.lock().unwrap();
        if let Some(light) = lights.get_mut(&id) {
            if light.hw_light.r#type == LightType::BACKLIGHT {
                let max_brightness = match get_max_brightness() {
                    Ok(val) => val,
                    Err(e) => {
                        error!("Failed to read max_brightness: {}. Using default 255.", e);
                        255
                    }
                };

                let brightness = get_brightness_from_state(state);
                let scaled_brightness = scale_brightness(brightness, max_brightness);

                if let Err(e) = set_brightness(scaled_brightness) {
                    error!("Failed to set brightness: {}", e);
                    let msg = format!("Failed to write brightness to sysfs: {}", e);
                    let c_msg = CString::new(msg).unwrap();
                    return Err(Status::new_exception(
                        ExceptionCode::TRANSACTION_FAILED,
                        Some(&c_msg),
                    ));
                }
            }

            light.state = *state;
            Ok(())
        } else {
            Err(Status::new_exception(ExceptionCode::UNSUPPORTED_OPERATION, None))
        }
    }

    fn getLights(&self) -> binder::Result<Vec<HwLight>> {
        info!("Lights reporting supported lights");
        Ok(self.lights.lock().unwrap().values().map(|light| light.hw_light).collect())
    }
}

fn get_max_brightness() -> std::io::Result<u32> {
    let path = format!("{}{}", LCD_LED_PATH, MAX_BRIGHTNESS_FILE);
    let content = fs::read_to_string(path)?;
    content.trim().parse::<u32>().map_err(|e| {
        error!("Failed to parse max_brightness: {}", e);
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })
}

fn set_brightness(value: u32) -> std::io::Result<()> {
    let path = format!("{}{}", LCD_LED_PATH, BRIGHTNESS_FILE);
    fs::write(path, value.to_string())
}

fn get_brightness_from_state(state: &HwLightState) -> u32 {
    let color = state.color as u32;
    let alpha = (color >> 24) & 0xFF;

    if alpha == 0 {
        return 0;
    }

    let red = (color >> 16) & 0xFF;
    let green = (color >> 8) & 0xFF;
    let blue = color & 0xFF;

    let red = red * alpha / 0xFF;
    let green = green * alpha / 0xFF;
    let blue = blue * alpha / 0xFF;

    (77 * red + 150 * green + 29 * blue) >> 8
}

fn scale_brightness(brightness: u32, max_brightness: u32) -> u32 {
    if brightness == 0 {
        return 0;
    }

    (brightness - 1) * (max_brightness - 1) / (0xFF - 1) + 1
}
