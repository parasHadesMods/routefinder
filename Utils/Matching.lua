Path = {
  Entries = {},
  Length = 0
}
function path_push(k)
  Path.Entries[Path.Length] = k
  Path.Length = Path.Length + 1
end
function path_pop()
  Path.Length = Path.Length - 1
  Path.Entries[Path.Length] = nil
end
function path_string()
  if Path.Length <= 0 then
    return ""
  else
    local p = Path.Entries[0]
    for i=1,Path.Length-1 do
      p = p .. "." .. Path.Entries[i]
    end
    return p
  end
end

DebugFalse = false
function debug_false(...)
  if DebugFalse then
    print(path_string(), ...)
  end
end

function matches(requirement, candidate)
  if type(requirement) == "function" then
    if not requirement(candidate) then
      -- debug should be printed by the requirement function
        return false
    end
  elseif type(requirement) == "table" then
    if type(candidate) ~= "table" then
      debug_false("not table")
      return false
    end
    for k,v in pairs(requirement) do
      path_push(k)
      if candidate[k] == nil then
        debug_false("nil")
        path_pop()
        return false
      end
      if not matches(v, candidate[k]) then
        -- debug was printed by recursive call
        path_pop()
        return false
      end
      path_pop()
    end
  elseif candidate ~= requirement then
    debug_false(requirement, candidate)
    return false
  end
  return true
end

function one_matches(requirements, candidates)
  if type(candidates) ~= "table" then
    debug_false("one_matches: not table")
    return false
  end
  for _,candidate in pairs(candidates) do
    if matches(requirements, candidate) then
      return true
    end
  end
  -- call to matches already printed debug
  return false
end

function matches_table(requirements, candidates)
  if type(candidates) ~= "table" then
    debug_false("matches_table: not table")
    return false
  end
  for k,v in pairs(candidates) do
    path_push(k)
    if requirements[k] == nil then
      debug_false("matches_table: requirements nil")
      path_pop()
      return false
    end
    if requirements[k] ~= v then
      debug_false("matches_table:", requirements[k], v)
      path_pop()
      return false
    end
    path_pop()
  end
  return true
end

function matches_one(options, candidate)
  for k,v in pairs(options) do
    if matches(v, candidate) then
      return true
    end
  end
  debug_false("matches_one: no matches")
  return false
end

function filter(requirements, candidates)
  local matched = {}
  for _,candidate in pairs(candidates) do
    if matches(requirements, candidate) then
      table.insert(matched, candidate)
    end
  end
  return matched
end
