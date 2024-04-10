use super::{
    handle_generic_command, Command, CommandVals, ControllableModule, Game, GameControlArg,
    GameVal, PatternVals,
};
use crate::games::pong;
use crate::games::snake;
use crate::matrix::*;
use crate::patterns::*;
use is31fl3741::PwmFreq;
use num::FromPrimitive;

/// tag struct for the Led Matrix input module
struct LedMatrixTag;

impl ControllableModule for LedMatrixTag {
    fn parse_command(count: usize, buf: &[u8]) -> Option<Command> {
        parse_module_command_ledmatrix(count, buf)
    }
}

pub fn parse_command_ledmatrix (count: usize, buf: &[u8]) -> Option<Command> {
    super::parse_command::<LedMatrixTag>(count, buf) 
}

pub enum LedMatrixCommand {
    /// Draw black/white on the grid
    Draw([u8; DRAW_BYTES]),
    StageGreyCol(u8, [u8; HEIGHT]),
    SetPwmFreq(PwmFreqArg),
}

#[derive(Copy, Clone, num_derive::FromPrimitive)]
pub enum PwmFreqArg {
    /// 29kHz
    P29k = 0x00,
    /// 3.6kHz
    P3k6 = 0x01,
    /// 1.8kHz
    P1k8 = 0x02,
    /// 900Hz
    P900 = 0x03,
}

impl From<PwmFreqArg> for PwmFreq {
    fn from(val: PwmFreqArg) -> Self {
        match val {
            PwmFreqArg::P29k => PwmFreq::P29k,
            PwmFreqArg::P3k6 => PwmFreq::P3k6,
            PwmFreqArg::P1k8 => PwmFreq::P1k8,
            PwmFreqArg::P900 => PwmFreq::P900,
        }
    }
}

pub fn parse_module_command_ledmatrix(count: usize, buf: &[u8]) -> Option<Command> {
    if count >= 3 && buf[0] == 0x32 && buf[1] == 0xAC {
        let command = buf[2];
        let arg = if count <= 3 { None } else { Some(buf[3]) };

        match FromPrimitive::from_u8(command) {
            Some(CommandVals::Brightness) => Some(if let Some(brightness) = arg {
                Command::SetBrightness(brightness)
            } else {
                Command::GetBrightness
            }),
            Some(CommandVals::Pattern) => match arg.and_then(FromPrimitive::from_u8) {
                // TODO: Convert arg to PatternVals
                Some(PatternVals::Percentage) => {
                    if count >= 5 {
                        Some(Command::Percentage(buf[4]))
                    } else {
                        None
                    }
                }
                Some(PatternVals::Gradient) => Some(Command::Pattern(PatternVals::Gradient)),
                Some(PatternVals::DoubleGradient) => {
                    Some(Command::Pattern(PatternVals::DoubleGradient))
                }
                Some(PatternVals::DisplayLotus) => {
                    Some(Command::Pattern(PatternVals::DisplayLotus))
                }
                Some(PatternVals::ZigZag) => Some(Command::Pattern(PatternVals::ZigZag)),
                Some(PatternVals::FullBrightness) => {
                    Some(Command::Pattern(PatternVals::FullBrightness))
                }
                Some(PatternVals::DisplayPanic) => {
                    Some(Command::Pattern(PatternVals::DisplayPanic))
                }
                Some(PatternVals::DisplayLotus2) => {
                    Some(Command::Pattern(PatternVals::DisplayLotus2))
                }
                None => None,
            },
            Some(CommandVals::Animate) => Some(if let Some(run_animation) = arg {
                Command::SetAnimate(run_animation == 1)
            } else {
                Command::GetAnimate
            }),
            Some(CommandVals::Draw) => {
                if count >= 3 + DRAW_BYTES {
                    let mut bytes = [0; DRAW_BYTES];
                    bytes.clone_from_slice(&buf[3..3 + DRAW_BYTES]);
                    Some(Command::LedMatrixCommand(LedMatrixCommand::Draw(bytes)))
                } else {
                    None
                }
            }
            Some(CommandVals::StageGreyCol) => {
                if count >= 3 + 1 + HEIGHT {
                    let mut bytes = [0; HEIGHT];
                    bytes.clone_from_slice(&buf[4..4 + HEIGHT]);
                    Some(Command::LedMatrixCommand(LedMatrixCommand::StageGreyCol(
                        buf[3], bytes,
                    )))
                } else {
                    None
                }
            }
            Some(CommandVals::DrawGreyColBuffer) => Some(Command::DrawGreyColBuffer),
            Some(CommandVals::StartGame) => match arg.and_then(FromPrimitive::from_u8) {
                Some(GameVal::Snake) => Some(Command::StartGame(Game::Snake)),
                Some(GameVal::Pong) => Some(Command::StartGame(Game::Pong)),
                Some(GameVal::Tetris) => None,
                Some(GameVal::GameOfLife) => {
                    if count >= 5 {
                        FromPrimitive::from_u8(buf[4])
                            .map(|x| Command::StartGame(Game::GameOfLife(x)))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            Some(CommandVals::GameControl) => match arg.and_then(FromPrimitive::from_u8) {
                Some(GameControlArg::Up) => Some(Command::GameControl(GameControlArg::Up)),
                Some(GameControlArg::Down) => Some(Command::GameControl(GameControlArg::Down)),
                Some(GameControlArg::Left) => Some(Command::GameControl(GameControlArg::Left)),
                Some(GameControlArg::Right) => Some(Command::GameControl(GameControlArg::Right)),
                Some(GameControlArg::Exit) => Some(Command::GameControl(GameControlArg::Exit)),
                Some(GameControlArg::SecondLeft) => {
                    Some(Command::GameControl(GameControlArg::SecondLeft))
                }
                Some(GameControlArg::SecondRight) => {
                    Some(Command::GameControl(GameControlArg::SecondRight))
                }
                _ => None,
            },
            Some(CommandVals::GameStatus) => Some(Command::GameStatus),
            Some(CommandVals::AnimationPeriod) => {
                if count == 3 + 2 {
                    let period = u16::from_le_bytes([buf[3], buf[4]]);
                    Some(Command::SetAnimationPeriod(period))
                } else {
                    Some(Command::GetAnimationPeriod)
                }
            }
            Some(CommandVals::PwmFreq) => {
                if let Some(freq) = arg {
                    FromPrimitive::from_u8(freq)
                        .map(LedMatrixCommand::SetPwmFreq)
                        .map(Command::LedMatrixCommand)
                } else {
                    Some(Command::GetPwmFreq)
                }
            }
            Some(CommandVals::DebugMode) => Some(if let Some(debug_mode) = arg {
                Command::SetDebugMode(debug_mode == 1)
            } else {
                Command::GetDebugMode
            }),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(feature = "ledmatrix")]
pub fn handle_command_ledmatrix(
    command: &Command,
    state: &mut LedmatrixState,
    matrix: &mut Foo,
    random: u8,
) -> Option<[u8; 32]> {
    use crate::games::game_of_life;

    match command {
        Command::GetBrightness => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.brightness;
            Some(response)
        }
        Command::SetBrightness(br) => {
            //let _ = serial.write("Brightness".as_bytes());
            set_brightness(state, *br, matrix);
            None
        }
        Command::Percentage(p) => {
            //let p = if count >= 5 { buf[4] } else { 100 };
            state.grid = percentage(*p as u16);
            None
        }
        Command::Pattern(pattern) => {
            //let _ = serial.write("Pattern".as_bytes());
            match pattern {
                PatternVals::Gradient => state.grid = gradient(),
                PatternVals::DoubleGradient => state.grid = double_gradient(),
                PatternVals::DisplayLotus => state.grid = display_lotus(),
                PatternVals::ZigZag => state.grid = zigzag(),
                PatternVals::FullBrightness => {
                    state.grid = percentage(100);
                    set_brightness(state, BRIGHTNESS_LEVELS, matrix);
                }
                PatternVals::DisplayPanic => state.grid = display_panic(),
                PatternVals::DisplayLotus2 => state.grid = display_lotus2(),
                _ => {}
            }
            None
        }
        Command::SetAnimate(a) => {
            state.auto_scroll = *a;
            None
        }
        Command::GetAnimate => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.auto_scroll as u8;
            Some(response)
        }
        Command::LedMatrixCommand(LedMatrixCommand::Draw(vals)) => {
            state.grid = draw(vals);
            None
        }
        Command::LedMatrixCommand(LedMatrixCommand::StageGreyCol(col, vals)) => {
            draw_grey_col(&mut state.col_buffer, *col, vals);
            None
        }
        Command::DrawGreyColBuffer => {
            // Copy the staging buffer to the real grid and display it
            state.grid = state.col_buffer.clone();
            // Zero the old staging buffer, just for good measure.
            state.col_buffer = percentage(0);
            None
        }
        // TODO: Move to handle_generic_command
        Command::IsSleeping => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = match state.sleeping {
                SleepState::Sleeping(_) => 1,
                SleepState::Awake => 0,
            };
            Some(response)
        }
        Command::StartGame(game) => {
            match game {
                Game::Snake => snake::start_game(state, random),
                Game::Pong => pong::start_game(state, random),
                Game::Tetris => {}
                Game::GameOfLife(param) => game_of_life::start_game(state, random, *param),
            }
            None
        }
        Command::GameControl(arg) => {
            match state.game {
                Some(GameState::Snake(_)) => snake::handle_control(state, arg),
                Some(GameState::Pong(_)) => pong::handle_control(state, arg),
                Some(GameState::GameOfLife(_)) => game_of_life::handle_control(state, arg),
                _ => {}
            }
            None
        }
        Command::GameStatus => None,
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
        Command::LedMatrixCommand(LedMatrixCommand::SetPwmFreq(arg)) => {
            state.pwm_freq = *arg;
            matrix.device.set_pwm_freq(state.pwm_freq.into()).unwrap();
            None
        }
        Command::GetPwmFreq => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.pwm_freq as u8;
            Some(response)
        }
        Command::SetDebugMode(arg) => {
            state.debug_mode = *arg;
            None
        }
        Command::GetDebugMode => {
            let mut response: [u8; 32] = [0; 32];
            response[0] = state.debug_mode as u8;
            Some(response)
        }
        _ => handle_generic_command(command),
    }
}
