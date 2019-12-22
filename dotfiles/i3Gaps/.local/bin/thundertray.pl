#!/usr/bin/perl
# This is totally borrowed from https://github.com/hanspr/ThunderTray

no warnings 'once';
no strict 'subs';
use utf8;
use Mozilla::Mork;
use Glib qw/TRUE FALSE/;
use Gtk3 -init;
use MIME::Base64;
use Cwd qw(abs_path);
use GD;

our (%icon,$icon,$eventbox,$tray,$NEW,$DIR,$FONT,$FONT_PATH,$TBW,$OFFSET,$emailchk,$MSEC,$IGNORE_CLICK,$DEBUG,$SCAN_ALL,$IGNORE_BOXES);
our ($LSTATUS,%KCOUNT,%LSTAT,@INBOX,$START);

# Begin Constants: Edit if auto setup does not work for you
$DIR = "";      #/home/MYUSER/.thunderbird/PROFILE.default;
$FONT_PATH =""; #/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf
$OFFSET = 0;
$MSEC = 1000;
$SCAN_ALL = 0;  #0 - Only INBOX, 1 - All boxes found
$IGNORE_BOXES = "sent,trash,spam,template,draft,junk,deleted,local"; # Comma separated list of words to use to ignore those names in boxes, is applied to the full path of the box
$DEBUG = 0; 	# 0-No debug, 1-Debug, 2-Debug and stop after scanning boxes
# End Constants

$NEW = -1;
$START = int(3000/$MSEC);
build_start();
$tray = Gtk3::StatusIcon->new();
$tray->set_from_pixbuf($icon{'tbrv'});
$tray->set_title("ThunderTray");
$tray->set_tooltip_text("Loading ...");
$tray->signal_connect ("button-release-event" => \&click);
$emailchk = Glib::Timeout->add($MSEC,\&CheckMail);
Gtk3->main;

sub CheckMail {
	my (@mails,%IDS,$id,$status,$new,$lmt,$x);

	if ($START > 0) {
		# Wait 3 seconds on start up, before scanning boxes
		$START--;
		return 1;
	}
	open(OLDER, ">&STDERR");
	open(STDERR,"> /dev/null");
	$new = ReadBoxes();
	open(STDERR, ">&OLDER");
	if ($new!=$NEW) {
		# Set Tray Icon number of messages
		setStatus($new,1);
		$NEW = $new;
	} else {
		setStatus($new,0);
	}
	if ($DEBUG){print "NEW=$new\n";if ($DEBUG==2){exit_it();}}
	return 1;
}

sub click {
	my ($map,$hidden,$start);

	if ($_[ 1 ]->button == 1) {
		#left mouse button
		if ($IGNORE_CLICK) {
			# Ignore clicks until Thunderbird starts
			return 1;
		}
		if ($LSTATUS eq 'C') {
			# Thunderbird is closed, Start
			$IGNORE_CLICK = 1;
			system "thunderbird & > /dev/null";
			$IGNORE_CLICK = 0;
		} elsif ($LSTATUS eq 'V') {
			# Is Visible, focus or hide
			$hidden = `xwininfo -all -id $TBW | grep 'Hidden'`;
			if ($hidden) {
				system "xdotool windowactivate $TBW";
			} else {
				system "xdotool windowunmap $TBW";
			}
		} else {
			# Unhide
			system "xdotool windowmap $TBW";
		}
		setStatus($NEW,0);
	} elsif ($_[ 1 ]->button == 2) {
		#middle mouse button
	} elsif ($_[ 1 ]->button == 3) {
		#right mouse button
		exit_it();
	}
	return 1;
}

sub exit_it {
	my ($map);

	if ($TBW) {
		$map = `xwininfo -id $TBW | grep 'IsViewable'`;
		if (!$map) {
			system "xdotool windowmap $TBW";
		}
	}
	Gtk3->main_quit;
	return 0;
}

sub setStatus {
	my ($new,$chk) = @_;
	my ($open,$map,$status);

	$open = `xdotool search --name 'Mozilla Thunderbird'`;
	chop $open;
	if ((!$open)||(!$TBW)||($IGNORE_CLICK)) {
		if (!$open) {
			select(undef, undef, undef, 0.25);
			$open = `xdotool search --name 'Mozilla Thunderbird'`;
			chop $open;
		}
		if (!$open) {
			$status = "C";
			$TBW = 0;
		} elsif ((!$TBW)||($TBW!=$open)) {
			$TBW = $open;
		}
	}
	if ($TBW) {
		$map = `xwininfo -id $TBW | grep 'IsViewable'`;
		if ($map) {
			$status = "V";
		} else {
			$status = "H";
		}
	}
	if (($LSTATUS eq $status)&&(!$chk)) {
		return 0;
	}
	if ($DEBUG) {print "STATUS CHANGE : $LSTATUS -> $status / $chk / $new\n";}
	$LSTATUS = $status;
	if ($new == 0) {
		if ($status eq 'V') {
			$tray->set_from_pixbuf($icon{'tbrv'});
		} elsif ($status eq 'H') {
			$tray->set_from_pixbuf($icon{'tbrh'});
		} elsif ($status eq 'C') {
			$tray->set_from_pixbuf($icon{'tbrx'});
		}
		$tray->set_tooltip_text("No mail");
	} else {
		my ($x,$img,$w,$b,$loader);
		my $pt = 12;
		if (length($new)>2) {$pt=9;}
		$img = new GD::Image(24,24);
		$w = $img->colorAllocate(255,255,255);
		if ($status eq 'C') {
			$b = $img->colorAllocate(255,0,0);
		} else {
			$b = $img->colorAllocate(0,0,0);
		}
		$img->fillToBorder(0,0,$w,$w);
		$x = int(12-(length($new)*$pt)/2);
		if ($x < 0) { $x = 0; }
		if ($FONT_PATH) {
			$img->stringFT($b,$FONT_PATH,$pt,0,$x,18,$new);
		} else {
			$img->string(gdGiantFont,$x,5,$new,$b);
		}
		$loader = Gtk3::Gdk::PixbufLoader->new();
		$loader->write([unpack 'C*', $img->png]);
		$loader->close();
		$tray->set_from_pixbuf($loader->get_pixbuf);
		$tray->set_tooltip_text("You got mail!");
	}
}

sub ReadBoxes {
	my ($box,$count,$MorkDetails,$results,@results,$r);

	if (!@INBOX) {
		# Load path to INBOX only once
		if ($DEBUG) {print qq|LOAD INBOX paths\n|;}
		loadBoxes("$DIR/ImapMail");
		loadBoxes("$DIR/Mail");
	}
	if ($DEBUG==2) {foreach my $k (@INBOX) {print "BOX:$k\n";}}
	$count=0;
	foreach $box (@INBOX) {
		if ($DEBUG) {print qq|Read : $box\n|;}
		my @stats = stat($box);
		if ($LSTAT{$box} eq "$stats[7]$stats[9]") {
			if ($DEBUG) {print qq|  No CHANGE : $KCOUNT{$box}\n|;}
			$count += $KCOUNT{$box};
		} else {
			my $new = 0;
			if ($DEBUG) {print "  Read BOX : ";}
			$LSTAT{$box} = "$stats[7]$stats[9]";
			$MorkDetails = Mozilla::Mork->new($box);
			$results = $MorkDetails->ReturnReferenceStructure();
			foreach $r (@$results) {
#				if ($DEBUG==2) {foreach my $k (sort keys %$r) {print "  :: $k=$$r{$k}\n";}}
				$new += $$r{'unreadChildren'};
			}
			if ($DEBUG) {print "$new\n";}
			$KCOUNT{$box} = $new;
			$count += $new;
		}
	}
	return $count;
}

sub loadBoxes {
	my ($dir,$depth) = @_;
	my ($BOXES,$box);

	if ((!$SCAN_ALL)&&($depth>1)) {
		# If not scan all, don't go deeper
		return 0;
	}
	opendir($BOXES,$dir);
	while ($box = readdir($BOXES)) {
		if ($box =~ /^\./) {
			next;
		} elsif (($box =~ /INBOX\.msf/i)||(($SCAN_ALL)&&($box =~ /\.msf$/))) {
			if (($SCAN_ALL)&&($IGNORE_BOXES)&&("$dir/$box" =~ /$IGNORE_BOXES/i)) {
				if ($DEBUG) {print "Ignored : $dir/$box\n";}
				next;
			}
			# Check size and timestamp of current BOX
			if ($DEBUG) {print qq|Process : $dir/$box\n|;}
			push @INBOX,"$dir/$box";
			if (!$SCAN_ALL) {
				last;
			}
		} elsif (-d "$dir/$box") {
			$depth++;
			loadBoxes("$dir/$box",$depth);
			$depth--;
		}
	}
	closedir($BOXES);
}

sub build_start {
	my (%data);

	$data{'tbrv'} = decode_base64('iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAHEQAABxEBsd7MgQAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAASzSURBVEiJnVVbbJRFFP5m/r97bbul7Pa6tBSw0kIptoIKoqJQkQdEEh7kgWBMCDFIIIbAi0kpkVQg8UGJRjEmigajBEwVA15SIFJACi0B2sACrb1tt3spu/vv5d+dOT60hZbdQuCbTDI5Ofm+M+d8k2F4CJY1+KuYwlZDyjoAJYyxfIDCJGmQgdqI6BizyOMntxdok3GwdMHX9vgqueD1AK19WAEjoEAyrn1CqtjXXF8We6TAsgb/RjAcAKA+mvw+pNCvhYdurrlwYMmNSQWWNfg/JWDz4xCPB0nh0XovrTx/8PXWsRgfOyzd7dsmgc0E4Ek3uJJnLp53rHp9U/EEgaUNgWom2b4nZh63OTc6Mx0zv3rgBrKRQIrCEqLA1A0aXVYT7p0fZ2VkOVZUrft2JQDwlxr8VSSxgghYa9vBd1VsRK7Bg6mZDPNsLhABJqYhWw0ARCDCIzfAmTWvahsApjKi1QSAgTDXfod1ujRkMB0DIYIdYWyd9SGedbTBoBISUkFPuASuYBluh2eh/e5CePWCtAPPsDheKF64PlcdeUQcdqMbM/PjkFSCVTiFRGIAy4uakWXlYIxhxHAEm6Ubc/O6ATTjVP917L26M60AU1RrzvTldVwSm0YAMngcisJQUSJg0y9g8ZQ/kJ3JwdI+RUAIoASnobJoekcxBVANszjA8gEpA3ou9KQKoxJBdZELHXeiSApKIQ5pAlddGv65PIxgOIYiY1f6WYBBUc0FKkmKEoNhy+xGuCN2ZFs8yMlSUFpkwrVbGqrLMyEkYcCjo2cwDlVhKHQYMcNphsnAAY9pdLATQVKAZDLOJeAmcJ5n9uKpqZ57CTOcJiQShEsdIZxpvYuglsTsGRbUVGbCmW+AxcShkxF9kSI8l9uc0iJJAkkRc3MQXQIB14crUvo9/+lM5NsNWDTfhsqZVkzJUqHw+0n+eC52Vu5CreNKyhAooSM6dOM0J8JxAtDifZGknKigqgzFDiMMGeknXWQZwCLnVVwZrk25QSIe7Ov6s76da7r4RUrhawssYKtO/Oirb93R2eabF5bE05I+iJgw4ryndsKARSKOqPfmUQBxZfDs/kTdqg3lQzFrTRKq5T9tmv30wBJ2pGuN6I6Uhw2KzvJNgypnMq3AX/2v4JR78bgIIaF5fUOt320Iuy8PqwBQW4qtEXF3XVfQZgaAHGM84/2adrT0OgxW62L83FMHkiE4LTexwP43TEoUACCJ47DrrVHjj0DEQxTqv9g4cPnrHmDcf7Dv8xOvtgTKT/aFsxQAWD+nA5tquyFJQlUUJJICt/1GtPROgabHkGPogy+i4AfXmvvkuoaIu+3Q9UNvvgtAnyAAAO1Ne2uEZdrvnf7cPDOP4flSDQXZqSZPCoGmG4VoPPfMvbaIeIgig1cOdXz/3iagPzKWm2KPi0c+KixzOg7m2CwrOU/vnt9cxfj4bAViQoFMxiGiAV+w7989t4++89lY5ZMKjKHlp11vTC90bLFlm5eqqmJUOIcgji8ulqHTa0QO9+FWwBoNDXub2s78ur2/9cteAClOmFRgDPs/eNteXVX6stlkmqOqfGo4FtP8vuCgx6ud27z7m/bRitNbDMD/d+Nupyos3ZQAAAAASUVORK5CYII=');
	$data{'tbrh'} = decode_base64('iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAAcRAAAHEQGx3syBAAAAB3RJTUUH4wYYEAoLN/LbLgAABDpJREFUSMetlktslFUUx3/3fo95dWb67lDaGoUY0RAMugAjvhZEuyDERHcmJgTDwhiNup+VISHGha+FJCIhuiJAVIwkRkoFXQBKVRDCq7R0Zjrt9DHPb77HcUEKrTPlof6Tm9yce8/5n3PPOTlXcRvsHaqtRautiGwGBhT0ACURySn4TZCDypLDrzwRKy9nQzUTfjFce1iJSoO8xJ0xE3j1D9Cy69VnkrU7Euwdqr2G4mPA5B4ggf9nvTL94vbBgQvLEuwdqn0o8Dr/FiKT9fmJwe1bVp1aEOmFzZ5jtbf+k3EApbqtROrgpwcurFxCsGfIWaeEXfwPUMros2Ptn/0jAtkpYCgCv8WcRQABbJOb+3tZhh17/pP9I4MAxudDtbVKeB9gbeR7tb7jtBqrrsaybLpDBeacCJaqY+s6XmAuV3gNcSht9Kxf3bLPBLYKoBC6ozMqX3DReBQdiFJnY8cP9EazGFoIRDNXT1Jw2ijU28nV+ij7LU0ptBXbGF+5rt280USKqFGivcVHSPIQo/h+kVXxK4RstajghG5rlu7YLHCFq8U8P01uah6D1rFw66rNWoR+AEN5KAVdyYCQP85A5NIi480qEpJcReMuc0ODNlZrUD0gQdWP4gcaQ7mk4gXyMy6BNKrVXWGy4HItU8OpeyTMWURoXIDWVspEpCpgb+gcpuTGCFklwraiNW4yWaiT6rARgWLZZ67soZUiHjNoS5iYhoKyuUyEgkjgaIGsoHTMqtARLd280JYwCQLITNUZzdRw3ICuNoveLptEzMAyFT4m826cvsiVxnKVgEC8rAZOI5CvdTZ4keqwaYka9KfCdLVZhG2NWpSWqhdhU9dRemO5xmYIfLzy1DEtwmGAsfJ9IrI0qVpDPGpg6OZpjFtF+hM5srXehjPfc67PXj56Rrt+cEgkmM7UVqovL748/WNm01/Zaqokd9VQ4InJeKl3qfOBh1eZPgA4CuDQiand+Vps24KSqQLXUIH0xbPV+1suh1dEJkJaBU0JLs4/wPHchqXeO6Xp4ujPj72zY8uoBuht5c1W26kuuBA2POvJvgnbUJGkbQ+Ezs4/xe+zGxktP4gn1q1KQfFHYc2Stw88R5zixM5S9vTYknlw4uSl58aqnUfm67YB8Gh3nsd7525oKYUEwkzVYGw+gut7hI15Kq5mZHrNLULfxS1l9uVHvtqWTqfrDQMnd+H4erGS301VI92m8uhvdYmGpGkbn59qYXh8xU1R4DnilnP7pka+2ZFOv11ZdmRmzg2vaEvEdofC1uByeb5YSDB8rQtPFBL4iFuddorX35s5d/CjBc9vO/QBxs8OvdAWj70RCpnPaq1DSikExcnrreQrJmFVYaZqV51a5evs6Pl3i5lT4+l0OrirX8VinDiyvzPVk3zaMs1HtFYddc8rVytOrlxxf/l26NczQL2Z4QX8DZKuC1rmjAtSAAAAAElFTkSuQmCC');
	$data{'tbrx'} = decode_base64('iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAAcRAAAHEQGx3syBAAAAB3RJTUUH4wgBEAIWDgybiwAABONJREFUSMellXto1lUYxz/n/H7v/bK9u71uzeU2sywSL3hBESGLhCxCkW5GEYZEYZYKXUAsMIJCDMK0hKjAggVJYIEiVKakhjPS1XI6N9fc3nfzvd9+7+93Tn+o0+WmpV84/xzOeT6H53nO9xFcR33D9o7uuHPncFq1WLYO2w5BKbFMSS7oFQMNEdm5vi3xzN4NE3LjxRBjbfZfsD86ftZedCGrpnIDGZLibRF9eGNbbMkPm5qLNwQc7bL2dJ63l6CR/A95XCq+YIrR1lDte3FcwP7fS0f6E85sblKm1Lm5LeqrlobQqst7I6889Fdp760EB7CVCBw+I1YMJYtbRwF6h+yPT5937kfDrS7bkeEDHfmlowDHz5bv0yCEUDpgpkbOu8yb42TKntbfTsW+BJDnhu0dyZxuBZjq2S8W1ezBZ+TwuaHOk7iYW2HhkYVL1/+LBF2DYi4gzO5B504NCDR1/iTDiTJSOGQt8GMxJ/ID9f5BDKlRWpIuh7lQqiRpVTFYaiDvBMdE5Mqexs7TfcvNoYzTggafmSMScNC6gin0oFSW5mAPHre4quE0ta4Utf4U0ENvNs6h2IIxARrhShXNJ2WxTBjAEDZCQE2Fwuv00+jtvir4GAE0hOlFivI4WZKkcuUqaTsEEFoXHR+OkhiiTF0owXCyjBoj5VZZE0+UOTdQpGTZhIzUuMUulQmYAmylMWZXHyJn+/G4cnjdgoqQSTxhEa1yozVk8w6prE3fmztJ7/2V8MxWZn/9BuTMUbXXjkN4zWLcHUfQ8xc3S9Mkq7UQATNPle+KZ1WGTJQDA0MWvQNFSpaiJuJixvvP4o5Wkj52mjPbvidTDtLg6xl5tW/Xe7g7jmDXNOB9a8sJGfKIAYB4seaadESr3QT8Bo1RLzURF163xFMZ4N4tq9BA99ZvmB7/lHr/4EXj6zyG77N3UEBmwzYmTp5wRtZHZBfA34WJaD26qFJCyG9g/Mv2qhfeQ9NzD4DtEHv9XQaSEXSpQGjzc0jHprj8JfSs+Zm7mqMvimnrBwLrlobijhI+j2EXGkOxfEvwVKjWO+gW1/lYTrHMLw9tIneqn8Ky1Wgl8O/ejj1pKhe2/Ui0onh06YKmeQLg4IlEe2fMPf2KKyplSq0bgzG7KdBtRr39hhTqGkjqRA+Hl76NUupSZ0oS239GNU8uLJlm7qyPVq256EUd/Qsj3pI94u2GLefVnzeE9nhcrkbjj/QCTibn0pu7A1u7RgD53jgohfAGwOtHKIXsO0VTZfFgfbRq7ah5MBBLfXHgL/VU2rr4u6bVDjGrPoXWGikljlIkCyZ9aS+WY2Omz9H/yAp0IkH65a2AJvzBK+hQpX08k7x9I/QDmJcBE+oqni5khoqp4Z4n4nl3wCUccpYg6BGXRqOkOqCoDuRRSvHTa1vQiQTW9EUUHl6FckoE932ekx3tgemwBXh81MAB8IVqnq+a0PTJ5Gqra0ptiaBn7AK3f3mQ7PffobwBEus+RKp8obUqt6+3o30GkBXw2G5YPu7QB8gM9+2SVnKOS9qTpBSGFAKlBUdOOgw9uhiRHMZ+dbPyr1j557TW8P76ushaQH0LLwjYBsQKcI+4kbMnhwZXaTv/oFMu1Grl+E+uXN2cOXCoxnf31PPLOv5oTUMJGGmxTSBmwj4BizW0/QOH7FN/HtcW2gAAAABJRU5ErkJggg==');
	foreach my $key ( keys %data ) {
		$icon{$key} = do {
			my $loader = Gtk3::Gdk::PixbufLoader->new();
			$loader->write([unpack 'C*', $data{$key}]);
			$loader->close();
			$loader->get_pixbuf();
		};
	}
	if ((!$FONT_PATH)&&(-e "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf")) {
		$FONT_PATH ="/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf";
	}
	if ($IGNORE_BOXES) {
		$IGNORE_BOXES =~ s/,/\|/g;
	}
	findUserDIR();
	setTBW();
}

sub findUserDIR {
	my ($dirh,$fn);

	if ($DIR) {
		return 0;
	}
	opendir($dirh,"$ENV{'HOME'}/.thunderbird") or die "ERROR opendir\n";;
	while ($fn = readdir($dirh)) {
		if ((-d "$ENV{'HOME'}/.thunderbird/$fn")&&($fn =~ /\.default$|\.\w+/)) {
			$DIR = "$ENV{'HOME'}/.thunderbird/$fn";
			last;
		}
	}
	closedir($dirh);
	if (!$DIR) {
		die "Could not find user DIR";
	}
}

sub setTBW {
	my ($x,$exit);
	my $r = 0;

	# Start Up script, wait 10 seconds or until found and start minimized to tray
	$exit = 0;
	while (!$TBW) {
		$x = `xdotool search --name 'Mozilla Thunderbird'`;
		chop $x;
		if ($x) {
			$TBW = $x;
			select(undef, undef, undef, 0.25);
			system "xdotool windowunmap --sync $TBW";
			system "xdotool windowsize $TBW 100% 100%";
			last;
		} elsif ($exit>40) {
			# Could not be found, aborted
			last;
		}
		$exit++;
		select(undef, undef, undef, 0.25);
	}
	# Wait for thunderbird to load email, before scanning
	return 0;
}
