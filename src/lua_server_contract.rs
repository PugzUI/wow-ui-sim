//! Shared validation and GUI wakeup for live simulator IPC commands.

/// Wake the GUI after enqueueing a command, independently of simulation timers.
/// Notify retains a permit when the subscription is between polls; coalescing
/// wakeups is safe because each GUI update drains the command queue.
pub(crate) static COMMAND_READY: tokio::sync::Notify = tokio::sync::Notify::const_new();

pub const MAX_FRAME_ADVANCE_SECONDS: f64 = 60.0;
pub const MAX_FRAME_ADVANCE_STEPS: u32 = 3_600;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FrameTimeAdvancePlan {
    pub(crate) seconds: f64,
    pub(crate) steps: u32,
    pub(crate) step_seconds: f64,
}

impl FrameTimeAdvancePlan {
    pub(crate) fn try_new(seconds: f64, steps: u32) -> Result<Self, String> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("AdvanceFrameTime seconds must be finite and non-negative".to_string());
        }
        if seconds > MAX_FRAME_ADVANCE_SECONDS {
            return Err(format!(
                "AdvanceFrameTime seconds must not exceed {MAX_FRAME_ADVANCE_SECONDS}"
            ));
        }
        if !(1..=MAX_FRAME_ADVANCE_STEPS).contains(&steps) {
            return Err(format!(
                "AdvanceFrameTime steps must be between 1 and {MAX_FRAME_ADVANCE_STEPS}"
            ));
        }

        Ok(Self {
            seconds,
            steps,
            step_seconds: seconds / f64::from(steps),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_time_plan_splits_elapsed_exactly() {
        let plan = FrameTimeAdvancePlan::try_new(0.75, 3).expect("valid plan");
        assert_eq!(plan.steps, 3);
        assert!((plan.step_seconds - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn frame_time_plan_accepts_zero_as_clock_freeze() {
        let plan = FrameTimeAdvancePlan::try_new(0.0, 1).expect("zero freezes time");
        assert_eq!(plan.step_seconds, 0.0);
    }

    #[test]
    fn frame_time_plan_rejects_negative_or_non_finite_seconds() {
        for seconds in [-0.1, f64::INFINITY, f64::NAN] {
            assert!(FrameTimeAdvancePlan::try_new(seconds, 1).is_err());
        }
    }

    #[test]
    fn frame_time_plan_bounds_total_seconds() {
        assert!(FrameTimeAdvancePlan::try_new(MAX_FRAME_ADVANCE_SECONDS, 1).is_ok());
        assert!(FrameTimeAdvancePlan::try_new(MAX_FRAME_ADVANCE_SECONDS + 0.01, 1).is_err());
    }

    #[test]
    fn frame_time_plan_bounds_step_count() {
        assert!(FrameTimeAdvancePlan::try_new(1.0, 0).is_err());
        assert!(FrameTimeAdvancePlan::try_new(1.0, MAX_FRAME_ADVANCE_STEPS).is_ok());
        assert!(FrameTimeAdvancePlan::try_new(1.0, MAX_FRAME_ADVANCE_STEPS + 1).is_err());
    }
}
