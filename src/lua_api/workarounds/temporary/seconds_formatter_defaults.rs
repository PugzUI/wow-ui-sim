//! Temporary `SecondsFormatter` utility defaults.
//!
//! The simulator does not load a complete Blizzard formatter utility surface
//! yet. Keep the small fallback shape explicit here instead of central runtime
//! bootstrap code.

const SECONDS_FORMATTER_DEFAULTS_LUA: &str = r#"
if type(SecondsFormatter) ~= "table" then
  SecondsFormatter = {
    Abbreviation = { None = 0 },
    Interval = { Minutes = 60 },
  }
end

if type(SecondsFormatter.IntervalDescription) ~= "table" then
  SecondsFormatter.IntervalDescription = {
    { formatString = { "%d sec" } },
    { formatString = { "%d min" } },
    { formatString = { "%d hr" } },
    { formatString = { "%d day" } },
  }
end

if type(SecondsFormatterMixin) ~= "table" then
  SecondsFormatterMixin = {}
end

local function __wow_seconds_formatter_interval_floor(interval)
  if interval == SecondsFormatter.Interval.Days or interval == 86400 then
    return 86400
  end
  if interval == SecondsFormatter.Interval.Hours or interval == 3600 then
    return 3600
  end
  if interval == SecondsFormatter.Interval.Minutes or interval == 60 then
    return 60
  end
  return 1
end

local function __wow_seconds_formatter_unit(seconds, minInterval)
  local minimum = __wow_seconds_formatter_interval_floor(minInterval)
  if seconds >= 86400 and minimum <= 86400 then
    return math.floor(seconds / 86400), "day"
  end
  if seconds >= 3600 and minimum <= 3600 then
    return math.floor(seconds / 3600), "hour"
  end
  if seconds >= 60 and minimum <= 60 then
    return math.floor(seconds / 60), "minute"
  end
  if minimum >= 86400 then
    return 0, "day"
  end
  if minimum >= 3600 then
    return 0, "hour"
  end
  if minimum >= 60 then
    return 0, "minute"
  end
  return math.max(0, math.floor(seconds)), "second"
end

local function __wow_seconds_formatter_format_unit(amount, unit)
  local singular = unit
  local plural = singular .. "s"
  return string.format("%d |4%s:%s;", amount, singular, plural)
end

if type(SecondsFormatterMixin.Init) ~= "function" then
  function SecondsFormatterMixin:Init(approximationSeconds, defaultAbbreviation, roundUpLastUnit, convertToLower, roundUpIntervals)
    self:SetApproximationSeconds(approximationSeconds or 0)
    self:SetMinInterval(SecondsFormatter.Interval.Seconds)
    self:SetDefaultAbbreviation(defaultAbbreviation or SecondsFormatter.Abbreviation.None)
    self:SetCanRoundUpLastUnit(roundUpLastUnit or false)
    self:SetConvertToLower(convertToLower or false)
    self:SetCanRoundUpIntervals(roundUpIntervals or false)
  end
end

if type(SecondsFormatterMixin.SetStripIntervalWhitespace) ~= "function" then
  function SecondsFormatterMixin:SetStripIntervalWhitespace(strip)
    self.stripIntervalWhitespace = strip
  end
end

if type(SecondsFormatterMixin.GetStripIntervalWhitespace) ~= "function" then
  function SecondsFormatterMixin:GetStripIntervalWhitespace()
    return self.stripIntervalWhitespace
  end
end

if type(SecondsFormatterMixin.SetConvertToLower) ~= "function" then
  function SecondsFormatterMixin:SetConvertToLower(convertToLower)
    self.convertToLower = convertToLower
  end
end

if type(SecondsFormatterMixin.SetDefaultAbbreviation) ~= "function" then
  function SecondsFormatterMixin:SetDefaultAbbreviation(defaultAbbreviation)
    self.defaultAbbreviation = defaultAbbreviation
  end
end

if type(SecondsFormatterMixin.SetApproximationSeconds) ~= "function" then
  function SecondsFormatterMixin:SetApproximationSeconds(approximationSeconds)
    self.approximationSeconds = approximationSeconds
  end
end

if type(SecondsFormatterMixin.SetCanRoundUpLastUnit) ~= "function" then
  function SecondsFormatterMixin:SetCanRoundUpLastUnit(roundUpLastUnit)
    self.roundUpLastUnit = roundUpLastUnit
  end
end

if type(SecondsFormatterMixin.SetCanRoundUpIntervals) ~= "function" then
  function SecondsFormatterMixin:SetCanRoundUpIntervals(roundUpIntervals)
    self.roundUpIntervals = roundUpIntervals
  end
end

if type(SecondsFormatterMixin.GetDesiredUnitCount) ~= "function" then
  function SecondsFormatterMixin:SetDesiredUnitCount(unitCount)
    self.unitCount = unitCount
  end

  function SecondsFormatterMixin:GetDesiredUnitCount(_seconds)
    return 1
  end
end

if type(SecondsFormatterMixin.SetMinInterval) ~= "function" then
  function SecondsFormatterMixin:SetMinInterval(interval)
    self.minInterval = interval
  end
end

if type(SecondsFormatterMixin.GetMinInterval) ~= "function" then
  function SecondsFormatterMixin:GetMinInterval(_seconds)
    return SecondsFormatter.Interval.Minutes
  end
end

if type(SecondsFormatterMixin.Format) ~= "function" then
  function SecondsFormatterMixin:Format(seconds)
    if seconds == nil then
      return ""
    end
    local amount, unit = __wow_seconds_formatter_unit(math.ceil(seconds), self:GetMinInterval(seconds))
    return __wow_seconds_formatter_format_unit(amount, unit)
  end
end

if type(SecondsFormatterConstants) ~= "table" then
  SecondsFormatterConstants = {
    ZeroApproximationThreshold = 0,
    ConvertToLower = true,
    DontConvertToLower = false,
    RoundUpLastUnit = true,
    DontRoundUpLastUnit = false,
    RoundUpIntervals = true,
    DontRoundUpIntervals = false,
  }
end

if type(SecondsFormatter.Abbreviation) ~= "table" then
  SecondsFormatter.Abbreviation = {}
end
SecondsFormatter.Abbreviation.None = SecondsFormatter.Abbreviation.None or 1
SecondsFormatter.Abbreviation.Truncate = SecondsFormatter.Abbreviation.Truncate or 2
SecondsFormatter.Abbreviation.OneLetter = SecondsFormatter.Abbreviation.OneLetter or 3

if type(SecondsFormatter.Interval) ~= "table" then
  SecondsFormatter.Interval = {}
end
SecondsFormatter.Interval.Seconds = SecondsFormatter.Interval.Seconds or 1
SecondsFormatter.Interval.Minutes = SecondsFormatter.Interval.Minutes or 2
SecondsFormatter.Interval.Hours = SecondsFormatter.Interval.Hours or 3
SecondsFormatter.Interval.Days = SecondsFormatter.Interval.Days or 4

if type(SecondsFormatterMixin.GetDefaultAbbreviation) ~= "function" then
  function SecondsFormatterMixin:GetDefaultAbbreviation()
    return self.defaultAbbreviation or SecondsFormatter.Abbreviation.None
  end
end

if type(SecondsFormatterMixin.GetApproximationSeconds) ~= "function" then
  function SecondsFormatterMixin:GetApproximationSeconds()
    return self.approximationSeconds or 0
  end
end

if type(SecondsFormatterMixin.CanRoundUpLastUnit) ~= "function" then
  function SecondsFormatterMixin:CanRoundUpLastUnit()
    return not not self.roundUpLastUnit
  end
end

if type(SecondsFormatterMixin.CanRoundUpIntervals) ~= "function" then
  function SecondsFormatterMixin:CanRoundUpIntervals()
    return not not self.roundUpIntervals
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SECONDS_FORMATTER_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_seconds_formatter_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("seconds formatter defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                local formatter = CreateFromMixins(SecondsFormatterMixin)
                formatter:Init(60, SecondsFormatter.Abbreviation.None, true)
                formatter:SetStripIntervalWhitespace(true)
                formatter:SetConvertToLower(true)
                formatter:SetDefaultAbbreviation(SecondsFormatter.Abbreviation.None)
                formatter:SetApproximationSeconds(30)
                formatter:SetCanRoundUpLastUnit(true)
                formatter:SetCanRoundUpIntervals(true)
                formatter:SetDesiredUnitCount(2)
                formatter:SetMinInterval(SecondsFormatter.Interval.Minutes)
                if formatter:GetStripIntervalWhitespace() ~= true then return "strip" end
                if formatter:GetDefaultAbbreviation() ~= SecondsFormatter.Abbreviation.None then return "abbreviation" end
                if formatter:GetApproximationSeconds() ~= 30 then return "approximation" end
                if formatter:CanRoundUpLastUnit() ~= true then return "round_unit" end
                if formatter:CanRoundUpIntervals() ~= true then return "round_intervals" end
                if formatter:GetDesiredUnitCount(120) ~= 1 then return "unit_count" end
                if formatter:GetMinInterval(120) ~= 60 then return "min_interval" end
                if formatter:Format(90) ~= "1 |4minute:minutes;" then return "format" end
                if SecondsFormatterConstants.ZeroApproximationThreshold ~= 0 then return "constants" end
                return "ok"
                "#,
            )
            .expect("seconds formatter defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_seconds_formatter_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            SecondsFormatter = { Interval = { Minutes = 120 } }
            SecondsFormatterMixin = {
              Format = function(_self, _seconds) return "existing" end,
            }
            "#,
        )
        .expect("fixture should install existing formatter members");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("seconds formatter defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if SecondsFormatter.Interval.Minutes ~= 120 then return "overwrote_formatter" end
                if SecondsFormatterMixin:Format(90) ~= "existing" then return "overwrote_format" end
                if type(SecondsFormatterMixin.Init) ~= "function" then return "missing_init" end
                return "ok"
                "#,
            )
            .expect("seconds formatter preservation probe should run");

        assert_eq!(result, "ok");
    }
}
