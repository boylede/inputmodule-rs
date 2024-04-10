use super::{handle_generic_command, Command, CommandVals, ControllableModule};
use crate::graphics::*;
use core::fmt::{Debug, Write};
use cortex_m::delay::Delay;
use embedded_graphics::Pixel;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Point, RgbColor},
    primitives::Rectangle,
};
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;
use heapless::String;
use num::FromPrimitive;
use st7306::{FpsConfig, PowerMode, ST7306};

/// tag struct for the B1 Display input module
struct B1DisplayTag;

impl ControllableModule for B1DisplayTag {
    fn parse_command(count: usize, buf: &[u8]) -> Option<Command> {
        parse_module_command_b1display(count, buf)
    }
}

pub fn parse_command_b1display (count: usize, buf: &[u8]) -> Option<Command> {
    super::parse_command::<B1DisplayTag>(count, buf) 
}

pub enum B1DisplayCommand {
    SetText(String<64>),
}

#[derive(Clone)]
pub enum SimpleSleepState {
    Awake,
    Sleeping,
}

#[derive(Copy, Clone)]
pub struct ScreenSaverState {
    pub rightwards: i32,
    pub downwards: i32,
}

impl Default for ScreenSaverState {
    fn default() -> Self {
        Self {
            rightwards: 1,
            downwards: 1,
        }
    }
}

pub struct B1DIsplayState {
    pub sleeping: SimpleSleepState,
    pub screen_inverted: bool,
    pub screen_on: bool,
    pub screensaver: Option<ScreenSaverState>,
    pub power_mode: PowerMode,
    pub fps_config: FpsConfig,
    pub animation_period: u64,
}

pub fn parse_module_command_b1display(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        match FromPrimitive::from_u8(command) {
            Some(CommandVals::SetText) => {
                if let Some(arg) = arg {
                    let available_len = count - 4;
                    let str_len = arg as usize;
                    assert!(str_len <= available_len);

                    assert!(str_len < 32);
                    let mut bytes = [0; 32];
                    bytes[..str_len].copy_from_slice(&buf[4..4 + str_len]);

                    let text_str = core::str::from_utf8(&bytes[..str_len]).unwrap();
                    let mut text: String<64> = String::new();
                    writeln!(&mut text, "{}", text_str).unwrap();

                    Some(Command::B1DisplayCommand(B1DisplayCommand::SetText(text)))
                } else {
                    None
                }
            }
            Some(CommandVals::DisplayOn) => Some(if let Some(on) = arg {
                Command::DisplayOn(on == 1)
            } else {
                Command::GetDisplayOn
            }),
            Some(CommandVals::InvertScreen) => Some(if let Some(invert) = arg {
                Command::InvertScreen(invert == 1)
            } else {
                Command::GetInvertScreen
            }),
            Some(CommandVals::SetPixelColumn) => {
                //  3B for magic and command
                //  2B for column (u16)
                // 50B for 400 pixels (400/8=50)
                if count == 3 + 2 + 50 {
                    let column = u16::from_le_bytes([buf[3], buf[4]]);
                    //panic!("SetPixelColumn. Col: {}", column);
                    let mut pixels: [u8; 50] = [0; 50];
                    pixels.clone_from_slice(&buf[5..55]);
                    Some(Command::SetPixelColumn(column as usize, pixels))
                } else {
                    None
                }
            }
            Some(CommandVals::FlushFramebuffer) => Some(Command::FlushFramebuffer),
            Some(CommandVals::ClearRam) => Some(Command::ClearRam),
            Some(CommandVals::ScreenSaver) => Some(if let Some(on) = arg {
                Command::ScreenSaver(on == 1)
            } else {
                Command::GetScreenSaver
            }),
            Some(CommandVals::SetFps) => Some(if let Some(fps) = arg {
                Command::SetFps(fps)
            } else {
                Command::GetFps
            }),
            Some(CommandVals::SetPowerMode) => Some(if let Some(mode) = arg {
                Command::SetPowerMode(mode)
            } else {
                Command::GetPowerMode
            }),
            Some(CommandVals::AnimationPeriod) => {
                if count == 3 + 2 {
                    let period = u16::from_le_bytes([buf[3], buf[4]]);
                    Some(Command::SetAnimationPeriod(period))
                } else {
                    Some(Command::GetAnimationPeriod)
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

pub fn handle_command_b1display<SPI, DC, CS, RST, const COLS: usize, const ROWS: usize>(
    command: &Command,
    state: &mut B1DIsplayState,
    logo_rect: Rectangle,
    disp: &mut ST7306<SPI, DC, CS, RST, COLS, ROWS>,
    delay: &mut Delay,
) -> Option<[u8; 32]>
where
    SPI: spi::Write<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    <SPI as spi::Write<u8>>::Error: Debug,
{
    match command {
        // TODO: Move to handle_generic_command
        Command::IsSleeping => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.sleeping {
                SimpleSleepState::Sleeping => 1,
                SimpleSleepState::Awake => 0,
            };
            Some(response)
        }
        Command::Panic => panic!("Ahhh"),
        Command::B1DisplayCommand(B1DisplayCommand::SetText(text)) => {
            // Turn screensaver off, when drawing something
            state.screensaver = None;

            clear_text(
                disp,
                Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y + logo_rect.size.height as i32),
                Rgb565::WHITE,
            )
            .unwrap();

            draw_text(
                disp,
                text,
                Point::new(LOGO_OFFSET_X, LOGO_OFFSET_Y + logo_rect.size.height as i32),
            )
            .unwrap();
            disp.flush().unwrap();
            None
        }
        Command::DisplayOn(on) => {
            state.screen_on = *on;
            disp.on_off(*on).unwrap();
            None
        }
        Command::GetDisplayOn => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.screen_on as u8;
            Some(response)
        }
        Command::InvertScreen(invert) => {
            state.screen_inverted = *invert;
            disp.invert_screen(state.screen_inverted).unwrap();
            None
        }
        Command::GetInvertScreen => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.screen_inverted as u8;
            Some(response)
        }
        Command::SetPixelColumn(column, pixel_bytes) => {
            // Turn screensaver off, when drawing something
            state.screensaver = None;

            let mut pixels: [bool; 400] = [false; 400];
            for (i, byte) in pixel_bytes.iter().enumerate() {
                pixels[8 * i] = byte & 0b00000001 != 0;
                pixels[8 * i + 1] = byte & 0b00000010 != 0;
                pixels[8 * i + 2] = byte & 0b00000100 != 0;
                pixels[8 * i + 3] = byte & 0b00001000 != 0;
                pixels[8 * i + 4] = byte & 0b00010000 != 0;
                pixels[8 * i + 5] = byte & 0b00100000 != 0;
                pixels[8 * i + 6] = byte & 0b01000000 != 0;
                pixels[8 * i + 7] = byte & 0b10000000 != 0;
            }
            disp.draw_pixels(
                pixels.iter().enumerate().map(|(y, black)| {
                    Pixel(
                        Point::new(*column as i32, y as i32),
                        if *black { Rgb565::BLACK } else { Rgb565::WHITE },
                    )
                }),
                false,
            )
            .unwrap();
            None
        }
        Command::FlushFramebuffer => {
            disp.flush().unwrap();
            None
        }
        Command::ClearRam => {
            // Turn screensaver off, when drawing something
            state.screensaver = None;

            disp.clear_ram().unwrap();
            None
        }
        Command::ScreenSaver(on) => {
            state.screensaver = match (*on, state.screensaver) {
                (true, Some(x)) => Some(x),
                (true, None) => Some(ScreenSaverState::default()),
                (false, Some(_)) => None,
                (false, None) => None,
            };
            None
        }
        Command::GetScreenSaver => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.screensaver.is_some() as u8;
            Some(response)
        }
        Command::SetFps(fps) => {
            if let Some(fps_config) = FpsConfig::from_u8(*fps) {
                state.fps_config = fps_config;
                disp.set_fps(state.fps_config).unwrap();
                // TODO: Need to reinit the display
            }
            None
        }
        Command::GetFps => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.fps_config.as_u8();
            Some(response)
        }
        Command::SetPowerMode(mode) => {
            match mode {
                0 => {
                    state.power_mode = PowerMode::Lpm;
                    disp.switch_mode(delay, state.power_mode).unwrap();
                }
                1 => {
                    state.power_mode = PowerMode::Hpm;
                    disp.switch_mode(delay, state.power_mode).unwrap();
                }
                _ => {}
            }
            None
        }
        Command::GetPowerMode => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.power_mode {
                PowerMode::Lpm => 0,
                PowerMode::Hpm => 1,
            };
            Some(response)
        }
        Command::SetAnimationPeriod(period) => {
            state.animation_period = (*period as u64) * 1_000;
            None
        }
        Command::GetAnimationPeriod => {
            // TODO: Doesn't seem to work when the FPS is 16 or higher
            let mut response: [u8; 32] = [0; 32];
            let period_ms = state.animation_period / 1_000;
            response[0..2].copy_from_slice(&(period_ms as u16).to_le_bytes());
            Some(response)
        }
        _ => handle_generic_command(command),
    }
}
