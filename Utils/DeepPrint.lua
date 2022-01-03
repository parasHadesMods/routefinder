function deep_print(t, indent)
  if not indent then indent = 0 end 
  local indentString = ""
  for i = 1, indent do
    indentString = indentString .. "  "
  end 
  for k,v in orderedPairs(t) do
    if type(v) == "table" then
      print(indentString..k)
      deep_print(v, indent + 1)
    else
      print(indentString..k, v)
    end 
  end 
end

