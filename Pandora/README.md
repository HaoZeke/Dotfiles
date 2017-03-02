#What?

These scripts are to be used to set-up Pandora for use with pianobar. (with a proxy for non-US residents)

## Pre-Requisites
You'll need to install the following and set them up as mentioned.
I'm assuming you're using pulseaudio.
ArchLinux is assumed but should work elsewhere too.

### Tor
Installing tor itself is trivial.
There's no need for torify or the rest but that's up to you.
You *will* however, need to add the following to your torrc

```bash
vim $HOME/.torrc  # or sudo vim /etc/tor/torrc
ExitNodes {US}
```

To run tor on Archlinux

```bash
sudo -u tor /usr/bin/tor
# OR
systemctl start tor
```

### Polipo
Tor runs as a SOCK5 proxy, but pianobar doesn't use SOCK5 proxies.... So say hello to *polipo*.

Use the polipo_pan file as the config file and run polipo

```bash
polipo -c $HOME/Github/Dotfiles/Pandora/polipo_pan
```

### Pianobar
This is the main program tying it all together. 
Basically just use the config file mentioned here. *(change the user details)*

```bash
cp -r ~/Github/Dotfiles/Pandora/pianobar ~/.config/
```

### Pulseaudio Fix (libao.conf)
So even after all that it's not gonna work.
Cuz libao.conf isn't set up for pulseaudio.

```bash
touch ~/.libao # OR sudo vim /etc/libao.conf
# Change to this  (comment out dev=default)
default_driver=pulse
```

## Profit