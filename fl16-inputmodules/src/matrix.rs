use crate::animations::*;
use crate::control::ledmatrix::PwmFreqArg;
use crate::games::game_of_life::GameOfLifeState;
use crate::games::pong::PongState;
use crate::games::snake::SnakeState;
use embedded_graphics::prelude::*;

pub const WIDTH: usize = 9;
pub const HEIGHT: usize = 34;
pub const LEDS: usize = WIDTH * HEIGHT;

#[derive(Clone)]
pub struct Grid(pub [[u8; HEIGHT]; WIDTH]);
impl Default for Grid {
    fn default() -> Self {
        Grid([[0; HEIGHT]; WIDTH])
    }
}

impl Grid {
    pub fn rotate(&mut self, rotations: usize) {
        for x in 0..WIDTH {
            self.0[x].rotate_right(rotations);
        }
    }
}
impl DrawTarget for Grid {
    type Color = embedded_graphics::pixelcolor::Gray8;
    type Error = ();
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, brightness) in pixels.into_iter() {
            let Some(row) = self.0.get_mut(y as usize) else {
                continue;
            };
            let Some(pixel) = row.get_mut(x as usize) else {
                continue;
            };
            *pixel = brightness.luma();
        }
        Ok(())
    }
}
impl OriginDimensions for Grid {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

pub struct LedmatrixState {
    /// Currently displayed grid
    pub grid: Grid,
    /// Temporary buffer for building a new grid
    pub col_buffer: Grid,
    /// Automatically advance pixels along the x axis each frame
    pub auto_scroll: bool,
    /// LED brightness out of 255
    pub brightness: u8,
    /// Current sleep state
    pub sleep_state: SleepState,
    /// State of the current game, if any
    pub game: Option<GameState>,
    pub animation_period: u64,
    /// Current LED PWM frequency
    pub pwm_freq: PwmFreqArg,
    /// Whether debug mode is active
    ///
    /// In debug mode:
    /// - Startup is instant, no animation
    /// - Sleep/wake transition is instant, no animation/fading
    /// - No automatic sleeping
    pub debug_mode: bool,
    pub upcoming_frames: Option<Animation>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
/// Whether asleep or not, if asleep contains data to restore previous LED grid
pub enum SleepState {
    Awake,
    Sleeping((Grid, u8)),
}

impl SleepState {
    pub fn is_awake(&self) -> bool {
        match self {
            SleepState::Awake => true,
            SleepState::Sleeping(_) => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SleepReason {
    Command,
    SleepPin,
    Timeout,
    UsbSuspend,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
/// State that's used for each game
pub enum GameState {
    Snake(SnakeState),
    Pong(PongState),
    GameOfLife(GameOfLifeState),
}
