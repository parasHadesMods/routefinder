require "../Utils/LazyDeepCopyTable"

function assertEqual(expected, actual)
    if not rawequal(expected, actual) then
        error("\n  Expected: " .. tostring(expected) .. "\n  Actual: " .. tostring(actual))
    end
end

Test = {
    Red = 0,
    [1] = "Orange",
    [2] = "Yellow",
    Nested = {
        [3] = "Green",
        Blue = 4
    }
}
-- table is stub until touched
Copy = LazyDeepCopyTable(Test)
assertEqual(Test, rawget(Copy, "__target"))

-- table is unstub on get
Copy = LazyDeepCopyTable(Test)
local tmp = Copy.Red
assertEqual(0, rawget(Copy, "Red"))
