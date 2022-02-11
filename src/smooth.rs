/// Helper to linearly ramp a parameter towards a target value
///
/// Useful for implementing filters like [`Gain`](crate::Gain) which have dynamic parameters, where
/// applying changes to parameters directly would cause unpleasant artifacts such as popping.
///
/// # Example
/// ```
/// let mut value = oddio::Smoothed::new(0.0);
/// assert_eq!(value.get(), 0.0);
/// // Changes only take effect after time passes
/// value.set(1.0);
/// assert_eq!(value.get(), 0.0);
/// value.advance(0.5);
/// assert_eq!(value.get(), 0.5);
/// // A new value can be supplied mid-interpolation without causing a discontinuity
/// value.set(1.5);
/// value.advance(0.5);
/// assert_eq!(value.get(), 1.0);
/// value.advance(0.5);
/// assert_eq!(value.get(), 1.5);
/// // Interpolation halts once the target value is reached
/// value.advance(0.5);
/// assert_eq!(value.get(), 1.5);
/// ```
#[derive(Copy, Clone, Default)]
pub struct Smoothed<T> {
    prev: T,
    next: T,
    progress: f32,
}

impl<T> Smoothed<T> {
    /// Create with initial value `x`
    pub fn new(x: T) -> Self
    where
        T: Clone,
    {
        Self {
            prev: x.clone(),
            next: x,
            progress: 1.0,
        }
    }

    /// Advance interpolation by `proportion`. For example, to advance at a fixed sample rate over a
    /// particular smoothing period, pass `sample_interval / smoothing_period`.
    pub fn advance(&mut self, proportion: f32) {
        self.progress = (self.progress + proportion).min(1.0);
    }

    /// Progress from the previous towards the next value
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Set the next value to `x`
    pub fn set(&mut self, value: T)
    where
        T: Interpolate,
    {
        //  IDEA: if we haven't reached destination yet and value is in same
        //  direction, don't set progress to zero. instead draw line between
        //  self.prev and new value, then set progress based on current progress
        //  value converted to this line

        if self.progress < 1. && (value - self.get()).sign() == (value - self.prev).sign() {
            let current = self.get();
            self.next = value;
            self.progress = ((current - self.prev) / (self.next - current)).to_f32();
        } else {
            self.prev = self.get();
            self.next = value;
            self.progress = 0.0;
        }
    }

    /// Get the current value
    pub fn get(&self) -> T
    where
        T: Interpolate,
    {
        self.prev.interpolate(&self.next, self.progress)
    }

    /// Get the value most recently passed to `set`
    pub fn target(&self) -> &T {
        &self.next
    }
}

/// Types that can be linearly interpolated, for use with [`Smoothed`]
pub trait Interpolate:
    core::ops::Sub<Output = Self> + core::ops::Div<Output = Self> + Sized + Copy + Clone
{
    /// Interpolate between `self` and `other` by `t`, which should be in [0, 1]
    fn interpolate(&self, other: &Self, t: f32) -> Self;

    /// Signum
    fn sign(&self) -> f32;

    /// convert to float
    fn to_f32(&self) -> f32;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let diff = other - self;
        self + t * diff
    }

    fn sign(&self) -> f32 {
        if self.is_sign_positive() {
            1.
        } else {
            -1.
        }
    }

    fn to_f32(&self) -> f32 {
        *self
    }
}

#[test]
fn repeated_set() {
    let mut s = Smoothed::new(0f32);

    for _ in 0..1000 {
        s.set(1.);
        s.advance(0.001);
    }

    assert!((s.get() - 1.).abs() < 0.01);
}
