//! Firmware API - Commands
use num::FromPrimitive;
use rp2040_hal::rom_data::reset_to_usb_boot;

use crate::serialnum::{device_release, is_pre_release};

#[cfg(feature = "b1display")]
use self::b1display::B1DisplayCommand;
#[cfg(feature = "c1minimal")]
use self::c1minimal::C1MinimalCommand;
#[cfg(feature = "ledmatrix")]
use self::ledmatrix::LedMatrixCommand;

#[cfg(feature = "b1display")]
pub mod b1display;
#[cfg(feature = "c1minimal")]
pub mod c1minimal;
#[cfg(feature = "ledmatrix")]
pub mod ledmatrix;

#[repr(u8)]
#[derive(num_derive::FromPrimitive)]
/// All available commands
pub enum CommandVals {
    Brightness = 0x00,
    Pattern = 0x01,
    BootloaderReset = 0x02,
    Sleep = 0x03,
    Animate = 0x04,
    Panic = 0x05,
    Draw = 0x06,
    StageGreyCol = 0x07,
    DrawGreyColBuffer = 0x08,
    SetText = 0x09,
    StartGame = 0x10,
    GameControl = 0x11,
    GameStatus = 0x12,
    SetColor = 0x13,
    DisplayOn = 0x14,
    InvertScreen = 0x15,
    SetPixelColumn = 0x16,
    FlushFramebuffer = 0x17,
    ClearRam = 0x18,
    ScreenSaver = 0x19,
    SetFps = 0x1A,
    SetPowerMode = 0x1B,
    AnimationPeriod = 0x1C,
    PwmFreq = 0x1E,
    DebugMode = 0x1F,
    Version = 0x20,
}

#[derive(num_derive::FromPrimitive)]
pub enum PatternVals {
    Percentage = 0x00,
    Gradient = 0x01,
    DoubleGradient = 0x02,
    DisplayLotus = 0x03,
    ZigZag = 0x04,
    FullBrightness = 0x05,
    DisplayPanic = 0x06,
    DisplayLotus2 = 0x07,
}

pub enum Game {
    Snake,
    Pong,
    Tetris,
    GameOfLife(GameOfLifeStartParam),
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum GameVal {
    Snake = 0,
    Pong = 1,
    Tetris = 2,
    GameOfLife = 3,
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum GameControlArg {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Exit = 4,
    SecondLeft = 5,
    SecondRight = 6,
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum GameOfLifeStartParam {
    CurrentMatrix = 0x00,
    Pattern1 = 0x01,
    Blinker = 0x02,
    Toad = 0x03,
    Beacon = 0x04,
    Glider = 0x05,
    BeaconToadBlinker = 0x06,
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum DisplayMode {
    /// Low Power Mode
    Lpm = 0x00,
    /// High Power Mode
    Hpm = 0x01,
}

// TODO: Reduce size for modules that don't require other commands
pub enum Command {
    /// Get current brightness scaling
    GetBrightness,
    /// Set brightness scaling
    SetBrightness(u8),
    /// Display pre-programmed pattern
    Pattern(PatternVals),
    /// Reset into bootloader
    BootloaderReset,
    /// Light up a percentage of the screen
    Percentage(u8),
    /// Go to sleepe or wake up
    Sleep(bool),
    IsSleeping,
    /// Start/stop animation (vertical scrolling)
    SetAnimate(bool),
    GetAnimate,
    /// Panic. Just to test what happens
    Panic,
    DrawGreyColBuffer,
    StartGame(Game),
    GameControl(GameControlArg),
    GameStatus,
    Version,
    GetColor,
    DisplayOn(bool),
    GetDisplayOn,
    InvertScreen(bool),
    GetInvertScreen,
    SetPixelColumn(usize, [u8; 50]),
    FlushFramebuffer,
    ClearRam,
    ScreenSaver(bool),
    GetScreenSaver,
    SetFps(u8),
    GetFps,
    SetPowerMode(u8),
    GetPowerMode,
    SetAnimationPeriod(u16),
    GetAnimationPeriod,
    GetPwmFreq,
    SetDebugMode(bool),
    GetDebugMode,
    #[cfg(feature = "ledmatrix")]
    LedMatrixCommand(LedMatrixCommand),
    #[cfg(feature = "b1display")]
    B1DisplayCommand(B1DisplayCommand),
    #[cfg(feature = "c1minimal")]
    C1MinimalCommand(C1MinimalCommand),
    _Unknown,
}

pub trait ControllableModule {
    fn parse_command(count: usize, buf: &[u8]) -> Option<Command>;
}

pub fn parse_command<M: ControllableModule>(count: usize, buf: &[u8]) -> Option<Command> {
    if let Some(command) = M::parse_command(count, buf) {
        return Some(command);
    }

    // Parse the generic commands common to all modules
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        //let mut text: String<64> = String::new();
        //writeln!(&mut text, "Command: {command}, arg: {arg}").unwrap();
        //let _ = serial.write(text.as_bytes());
        match FromPrimitive::from_u8(command) {
            Some(CommandVals::Sleep) => Some(if let Some(go_to_sleep) = arg {
                Command::Sleep(go_to_sleep == 1)
            } else {
                Command::IsSleeping
            }),
            Some(CommandVals::BootloaderReset) => Some(Command::BootloaderReset),
            Some(CommandVals::Panic) => Some(Command::Panic),
            Some(CommandVals::Version) => Some(Command::Version),
            _ => None, //Some(Command::Unknown),
        }
    } else {
        None
    }
}

pub fn handle_generic_command(command: &Command) -> Option<[u8; 32]> {
    match command {
        Command::BootloaderReset => {
            //let _ = serial.write("Bootloader Reset".as_bytes());
            reset_to_usb_boot(0, 0);
            None
        }
        Command::Panic => panic!("Ahhh"),
        Command::Version => {
            let mut response: [u8; 32] = [0; 32];
            let bcd_device = device_release().to_be_bytes();
            response[0] = bcd_device[0];
            response[1] = bcd_device[1];
            response[2] = is_pre_release() as u8;
            Some(response)
        }
        _ => None,
    }
}
