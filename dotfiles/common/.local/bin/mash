#!/usr/bin/env perl
use strict;
use File::Temp 'tempfile';

# MASH: Mathematica Scripting Hack (use Mathematica like Perl/Python/Ruby/etc)
#  by Daniel Reeves, 1999; rewritten in Perl for Mathematica 6+ in 2008.
# MASH allows you to have a self-contained Mathematica script that can be
#  executed (with arguments) from the command line and used in a pipeline.
# It functions as the interpreter for Mathematica scripts by serving as a proxy
#   between the script and the kernel.  Namely, it does the following:
#  * Takes a Mathematica source file as its first argument (or from stdin if no
#    arguments).
#  * Makes the command line arguments available to the Mathematica code as a
#    list called ARGV (or 'args') and defines convenient functions for printing
#    to stdout and stderr and reading from stdin.  This is all done by
#    evaluating the code in $prescript below.
#  * Evaluates the Mathematica script (printing to stdout and stderr only what
#    it is explicitly told to).
#  * Evaluates the code in $postfix below, including exiting Mathematica.
#  * Additionally, because Mathematica, annoyingly, detaches itself from the
#    parent process (this script), we have to pass along various termination
#    signals explicitly (for example, hitting control-c needs to send SIGINT to
#    Mathematica, which is like "interrupt evaluation" in the frontend).
# To make a Mathematica script foo.m executable:
#  * Make the first line of foo.m be:  #!/usr/bin/env /path/to/mash.pl
#    (or just "#!/usr/bin/env mash" if mash is in $PATH).
#  * Do chmod u+x foo.m

# Possible paths to the Mathematica kernel.  Add yours.
my @mathpath = (
  "/usr/bin/math",
  "/usr/local/bin/math",
  "/Applications/Mathematica.app/Contents/MacOS/MathKernel",
  "/Applications/Mathematica Home Edition.app/Contents/MacOS/MathKernel",
);
my $math;  # The first of the above that actually exists.
for (@mathpath) { if(-e $_) { $math = $_;  last; } }
die "Mathematica kernel not found.\n" unless defined($math);

# Evaluate this in Mathematica before eval'ing the script.
my $prescript = <<'EOF';  # Much of this stuff probably belongs in a package.
ARGV = args = Drop[$CommandLine, 4];        (* Command line args.             *)
pr = WriteString["stdout", ##]&;            (* More                           *)
prn = pr[##, "\n"]&;                        (*  convenient                    *)
perr = WriteString["stderr", ##]&;          (*   print                        *)
perrn = perr[##, "\n"]&;                    (*    statements.                 *)
re = RegularExpression;                     (* I wish mathematica             *)
eval = ToExpression[cat[##]]&;              (*  weren't so damn               *)
EOF = EndOfFile;                            (*   verbose!                     *)
read[] := InputString[""];                  (* Grab a line from stdin.        *)
doList[f_, test_] :=                        (* Accumulate list of what f[]    *)
  Most@NestWhileList[f[]&, f[], test];      (*  returns while test is true.   *)
readList[] := doList[read, #=!=EOF&];       (* Slurp list'o'lines from stdin. *)
cat = StringJoin@@(ToString/@{##})&;        (* Like sprintf/strout in C/C++.  *)
system = Run@cat@##&;                       (* System call.                   *)
backtick = Import[cat["!", ##], "Text"]&;   (* System call; returns stdout.   *)
                                            (* ABOVE: mma-scripting related.  *)
keys[f_, i_:1] :=                           (* BELOW: general utilities.      *)
  DownValues[f, Sort->False][[All,1,1,i]];  (* Keys of a hash/dictionary.     *)
SetAttributes[each, HoldAll];               (* each[pattern, list, body]      *)
each[pat_, lst_, bod_] := ReleaseHold[      (*  converts pattern to body for  *)
  Hold[Cases[Evaluate@lst, pat:>bod];]];    (*   each element of list.        *)
some[f_, l_List] := True ===                (* Whether f applied to some      *)
  Scan[If[f[#], Return[True]]&, l];         (*  element of list is True.      *)
every[f_, l_List] := Null ===               (* Similarly, And @@ f/@l         *)
  Scan[If[!f[#], Return[False]]&, l];       (*  (but with lazy evaluation).   *)
pout = pr;                                   (* [Backward compatibility.]     *)
strout = cat;                                (* [Backward compatibility.]     *)
EOF

# Evaluate this in Mathematica after eval'ing the script.
my $postscript = <<'EOF';
Exit[0];
EOF

# Slurp up the script, either from file or from stdin.
my $script = $ARGV[0];
my @lines;
if(defined($script)) {
  open(F, "<$script") or die qq{Can't open Mathematica script "$script": $!\n};
  @lines = <F>;
  close(F);
} else { @lines = <STDIN>; }

# Feed slurped script plus prescript and postscript to temp file.
shift(@lines) if $lines[0] =~ /^\#\!/;  # exclude the shebang line.
my($tmpfh, $tmpf) = tempfile("mash-XXXX", SUFFIX=>'.m', UNLINK=>1);
print $tmpfh $prescript, "\n\n", @lines, "\n\n", $postscript;

# Open a pipe to Mathematica.
my $cmd = "'$math' -noprompt -run \"<<$tmpf\" " . join(' ', @ARGV);
my $pid = open(F, "$cmd |") or die "Can't open pipe from $cmd: $!";

# Handle interrupt and termination signals.
$SIG{INT} =  sub { kill('INT', $pid); };  # pass along INT (2, eg, ctrl-C)
$SIG{TERM} = sub { kill('TERM', $pid); }; # pass along TERM (15, kill's default)
$SIG{QUIT} = sub { kill('QUIT', $pid); }; # pass along QUIT (3)
$SIG{ABRT} = sub { kill('ABRT', $pid); }; # pass along ABRT (6)
$SIG{HUP} =  sub { kill('HUP', $pid); };  # pass along HUP (1)

# Actually run the Mathematica script, then close the pipe.
print while(<F>);
close(F);
#print STDERR "DEBUG: exiting MASH\n";

#system("cp $tmpf mash-DEBUG.m"); # keep copy of the script with pre/postscript.
