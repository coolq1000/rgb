use super::timer::Timer;

#[derive(Default)]
pub struct Length {
    pub timer: Timer,
    pub enable: bool,
}
