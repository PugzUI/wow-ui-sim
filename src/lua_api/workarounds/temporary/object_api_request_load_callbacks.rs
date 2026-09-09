//! Temporary synchronous ObjectAPI request-load callbacks.
//!
//! Blizzard_ObjectAPI registers listener objects for async item/spell cache
//! loads. The simulator does not model those caches yet, so item and spell
//! request-load calls immediately flush their listener callbacks.

const OBJECT_API_REQUEST_LOAD_CALLBACKS_LUA: &str = r#"
local function __wow_fire_listener_callbacks(listener, id)
  if listener ~= nil and type(listener.FireCallbacks) == "function" then
    listener:FireCallbacks(id)
  end
end

if C_Item ~= nil and type(rawget(C_Item, "RequestLoadItemDataByID")) ~= "function" then
  function C_Item.RequestLoadItemDataByID(itemID)
    __wow_fire_listener_callbacks(ItemEventListener, itemID)
    return true
  end
end

if C_Spell ~= nil and type(rawget(C_Spell, "RequestLoadSpellData")) ~= "function" then
  function C_Spell.RequestLoadSpellData(spellID)
    __wow_fire_listener_callbacks(SpellEventListener, spellID)
    return true
  end
end

-- Mists can skip Blizzard_ObjectAPI while WeakAuras still consumes its
-- nonvisual Spell object. Keep this contract backed by the simulator's
-- C_Spell surface; a real Blizzard definition wins when present.
if WOW_PROJECT_ID == WOW_PROJECT_MISTS and rawget(_G, "Spell") == nil then
  SpellMixin = SpellMixin or {}

  function SpellMixin:SetSpellID(spellID)
    self.spellID = tonumber(spellID)
  end

  function SpellMixin:GetSpellID()
    return self.spellID
  end

  function SpellMixin:Clear()
    self.spellID = nil
  end

  function SpellMixin:IsSpellEmpty()
    return self.spellID == nil or self.spellID == 0
  end

  function SpellMixin:IsSpellDataCached()
    return not self:IsSpellEmpty()
      and type(C_Spell) == "table"
      and type(C_Spell.IsSpellDataCached) == "function"
      and C_Spell.IsSpellDataCached(self.spellID) or false
  end

  function SpellMixin:GetSpellName()
    return C_Spell.GetSpellName(self.spellID)
  end

  function SpellMixin:GetSpellTexture()
    return C_Spell.GetSpellTexture(self.spellID)
  end

  function SpellMixin:GetSpellSubtext()
    if type(C_Spell.GetSpellSubtext) == "function" then
      return C_Spell.GetSpellSubtext(self.spellID)
    end
    return ""
  end

  function SpellMixin:GetSpellDescription()
    return C_Spell.GetSpellDescription(self.spellID)
  end

  local function spellCancelHandle()
    local cancelled = false
    local handle = {}
    local function cancel()
      if cancelled then
        return false
      end
      cancelled = true
      return true
    end
    setmetatable(handle, { __call = cancel })
    handle.Cancel = cancel
    handle.CancelCallback = cancel
    return handle
  end

  function SpellMixin:ContinueOnSpellLoad(callback)
    if type(callback) == "function" then
      callback(self)
    end
  end

  function SpellMixin:ContinueWithCancelOnSpellLoad(callback)
    local handle = spellCancelHandle()
    if type(callback) == "function" then
      callback(self)
      handle.Cancel = function() return false end
      handle.CancelCallback = handle.Cancel
      setmetatable(handle, { __call = handle.Cancel })
    end
    return handle
  end

  Spell = SpellMixin
  function Spell:CreateFromSpellID(spellID)
    local spell = CreateFromMixins(SpellMixin)
    spell:SetSpellID(spellID)
    return spell
  end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(OBJECT_API_REQUEST_LOAD_CALLBACKS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_item_and_spell_request_load_callbacks_when_missing() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            rawset(C_Item, "RequestLoadItemDataByID", nil)
            rawset(C_Spell, "RequestLoadSpellData", nil)
            item_fired = nil
            spell_fired = nil
            ItemEventListener = {
              FireCallbacks = function(self, itemID)
                item_fired = itemID
              end,
            }
            SpellEventListener = {
              FireCallbacks = function(self, spellID)
                spell_fired = spellID
              end,
            }
            "#,
        )
        .expect("fixture should reset request-load callbacks");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("request-load callbacks should apply");
        }

        let (item_result, item_fired, spell_result, spell_fired): (bool, i32, bool, i32) = env
            .eval(
                r#"
                local itemResult = C_Item.RequestLoadItemDataByID(6948)
                local spellResult = C_Spell.RequestLoadSpellData(35395)
                return itemResult, item_fired, spellResult, spell_fired
                "#,
            )
            .expect("request-load callbacks should run");

        assert!(item_result);
        assert_eq!(item_fired, 6948);
        assert!(spell_result);
        assert_eq!(spell_fired, 35395);
    }

    #[test]
    fn preserves_existing_request_load_callbacks() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_Item.RequestLoadItemDataByID = function()
              return "item-existing"
            end
            C_Spell.RequestLoadSpellData = function()
              return "spell-existing"
            end
            "#,
        )
        .expect("fixture should install existing callbacks");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("request-load callbacks should apply");
        }

        let (item_result, spell_result): (String, String) = env
            .eval(
                r#"
                return C_Item.RequestLoadItemDataByID(1), C_Spell.RequestLoadSpellData(2)
                "#,
            )
            .expect("existing callbacks should run");

        assert_eq!(item_result, "item-existing");
        assert_eq!(spell_result, "spell-existing");
    }

    #[test]
    fn installs_mists_spell_object_contract() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            WOW_PROJECT_MISTS = 19
            WOW_PROJECT_ID = WOW_PROJECT_MISTS
            Spell = nil
            SpellMixin = nil
            "#,
        )
        .expect("fixture should select Mists and clear ObjectAPI globals");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("ObjectAPI compatibility should apply");
        }

        let (known_id, known_empty, known_name, known_icon, known_description, empty): (
            i32,
            bool,
            String,
            i32,
            String,
            bool,
        ) = env
            .eval(
                r#"
                local known = Spell:CreateFromSpellID(116)
                local _, icon = known:GetSpellTexture()
                local empty = Spell:CreateFromSpellID(nil)
                return known:GetSpellID(), known:IsSpellEmpty(), known:GetSpellName(),
                       icon, known:GetSpellDescription(), empty:IsSpellEmpty()
                "#,
            )
            .expect("Spell object accessors should be callable");

        assert_eq!(known_id, 116);
        assert!(!known_empty);
        assert_ne!(known_name, "Unknown");
        assert!(known_icon > 0);
        assert!(!known_description.is_empty());
        assert!(empty);
    }

    #[test]
    fn mists_spell_callbacks_are_synchronous_and_cancelled_after_fire() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            WOW_PROJECT_MISTS = 19
            WOW_PROJECT_ID = WOW_PROJECT_MISTS
            Spell = nil
            SpellMixin = nil
            "#,
        )
        .expect("fixture should select Mists and clear ObjectAPI globals");
        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("ObjectAPI compatibility should apply");
        }

        let (fired, cancel_call, cancel_method, cancel_alias): (bool, bool, bool, bool) = env
            .eval(
                r#"
                local fired = false
                local spell = Spell:CreateFromSpellID(116)
                local cancel = spell:ContinueWithCancelOnSpellLoad(function(object)
                    fired = object:GetSpellID() == 116
                end)
                return fired, cancel(), cancel:Cancel(), cancel:CancelCallback()
                "#,
            )
            .expect("Spell callback and cancellation aliases should be callable");

        assert!(fired);
        assert!(!cancel_call);
        assert!(!cancel_method);
        assert!(!cancel_alias);
    }
}
