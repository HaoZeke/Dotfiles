help([[
The autotools modulefile defines m4, autoconf, automake and libtool
]])
-- 1 --
if (os.getenv("USER") ~= "root") then
append_path("PATH", ".")
end
-- 2 --
load("m4","autoconf","automake","libtool")
