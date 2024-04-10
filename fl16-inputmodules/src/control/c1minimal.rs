use super::{handle_generic_command, Command, CommandVals, ControllableModule};
use num::FromPrimitive;
use smart_leds::{SmartLedsWrite, RGB8};

/// tag struct for the C1 Minimal input module
struct C1MinimalTag;

impl ControllableModule for C1MinimalTag {
    fn parse_command(count: usize, buf: &[u8]) -> Option<Command> {
        parse_module_command_c1minimal(count, buf)
    }
}

pub fn parse_command_c1minimal (count: usize, buf: &[u8]) -> Option<Command> {
    super::parse_command::<C1MinimalTag>(count, buf) 
}

pub enum C1MinimalCommand {
    SetColor(RGB8),
}

pub struct C1MinimalState {
    pub sleeping: SimpleSleepState,
    pub color: RGB8,
    pub brightness: u8,
}

#[derive(Clone)]
pub enum SimpleSleepState {
    Awake,
    Sleeping,
}

pub fn handle_command_c1minimal(
    command: &Command,
    state: &mut C1MinimalState,
    ws2812: &mut impl SmartLedsWrite<Color = RGB8, Error = ()>,
) -> Option<[u8; 32]> {
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
        Command::GetBrightness => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.brightness;
            Some(response)
        }
        Command::SetBrightness(br) => {
            //let _ = serial.write("Brightness".as_bytes());
            state.brightness = *br;
            ws2812
                .write(smart_leds::brightness(
                    [state.color].iter().cloned(),
                    state.brightness,
                ))
                .unwrap();
            None
        }
        Command::GetColor => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.color.r;
            response[1] = state.color.g;
            response[2] = state.color.b;
            Some(response)
        }
        Command::C1MinimalCommand(C1MinimalCommand::SetColor(color)) => {
            state.color = *color;
            ws2812
                .write(smart_leds::brightness(
                    [*color].iter().cloned(),
                    state.brightness,
                ))
                .unwrap();
            None
        }
        // TODO: Make it return something
        _ => handle_generic_command(command),
    }
}

pub fn parse_module_command_c1minimal(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        match FromPrimitive::from_u8(command) {
            Some(CommandVals::Brightness) => Some(if let Some(brightness) = arg {
                Command::SetBrightness(brightness)
            } else {
                Command::GetBrightness
            }),
            Some(CommandVals::SetColor) => {
                if count >= 6 {
                    let (red, green, blue) = (buf[3], buf[4], buf[5]);
                    Some(Command::C1MinimalCommand(C1MinimalCommand::SetColor(
                        RGB8::new(red, green, blue),
                    )))
                } else if arg.is_none() {
                    Some(Command::GetColor)
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
