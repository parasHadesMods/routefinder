function matches(requirements, candidate)
  for k,v in pairs(requirements) do
    if candidate[k] == nil then
      return false
    end 
    if type(v) == "function" then
      if not v(candidate[k]) then
        return false
      end 
    elseif type(v) == "table" then
      if not matches(v, candidate[k]) then
        return false
      end 
    elseif candidate[k] ~= v then
      return false
    end 
  end 
  return true
end

function one_matches(requirements, candidates)
  if type(candidates) ~= "table" then
    return false
  end 
  for _,candidate in pairs(candidates) do
    if matches(requirements, candidate) then
      return true
    end
  end
  return false
end

function matches_table(requirements, candidates)
  if type(candidates) ~= "table" then
    return false
  end
  for k,v in pairs(candidates) do
    if requirements[k] == nil then
      return false
    end
    if requirements[k] ~= v then
      return false
    end
  end
  return true
end

function matches_one(options, candidate)
  for k,v in pairs(options) do
    if v == candidate then
      return true
    end
  end
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
