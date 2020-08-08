help([[
The autotools modulefile defines m4, autoconf, automake and libtool
]])
-- 1 --
if (os.getenv("USER") ~= "root") then
append_path("PATH", ".")
end
-- 2 --
load("autotools/m4/1.4.14","autotools/autoconf/2.64","autotools/automake/1.11.1","autotools/libtool/2.4")
