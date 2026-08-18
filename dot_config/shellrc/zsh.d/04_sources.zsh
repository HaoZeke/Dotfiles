# Common zinit stuff
######################
#
# Turbo. `zinit times` measured 166 ms of synchronous plugin loading, all of it
# ahead of the first prompt. Everything that does not have to be in place
# before the prompt renders now loads just after it, under `wait`.
#
# The suffix on each wait orders the queue, since zinit runs a bucket in
# registration order and buckets in lexicographic order:
#
#   0a  programs (04b_programs.zsh), so later plugins find their binaries
#   0b  ordinary plugins, aliases, snippets, autosuggestions
#   0c  fast-syntax-highlighting, which wraps ZLE widgets and has to see every
#       other widget already defined
#
# Left synchronous: zsh-completions, because it adds to fpath and compinit runs
# at the end of zshenv.zsh, before any turbo bucket fires.

# OMZ Plugins
zinit wait"0b" lucid for \
    OMZP::git/git.plugin.zsh \
    OMZP::github/github.plugin.zsh \
    OMZP::systemadmin \
    OMZP::sudo \
    OMZP::systemd \
    OMZP::rsync \
    OMZP::common-aliases/common-aliases.plugin.zsh

# Colors and Highlighting
zinit wait"0b" lucid for \
    zlsun/solarized-man

# Nix shell
# zinit light "chisui/zsh-nix-shell"

# Misc
#
# The ssh agent is handled in posix.d/06_prog_conf.sh, which bash and zsh both
# source. bobsoppe/zsh-ssh-agent used to do it again here for zsh alone, at
# 44 ms per shell -- the most expensive plugin in the list -- because it runs
# `ps x | grep ssh-agent | grep -q $SSH_AGENT_PID` every time. It also sourced
# ~/.ssh/environment-$HOST over the inherited environment, so a zsh session and
# a bash session on this host pointed at two different agents with two
# different sets of loaded keys, and a stale entry there spawned another agent
# rather than reusing the running one.
zinit wait"0b" lucid for \
    mollifier/cd-gitroot \
    urbainvaes/fzf-marks \
    changyuheng/zsh-interactive-cd

# Completions
zinit ice blockf
zinit light "zsh-users/zsh-completions"

# Spelled out rather than folded into the `for` list above: zinit's word parser
# reads the leading `as` of ascii-soup as the `as` ice and rejects the rest of
# the name as its value.
zinit ice wait"0b" lucid
zinit light "ascii-soup/zsh-url-highlighter"

zinit wait"0b" lucid for \
    molovo/tipz

# Suggestions
zinit ice wait"0b" lucid atload'_zsh_autosuggest_start'
zinit light zsh-users/zsh-autosuggestions

# Syntax highlighting last, after every other widget exists. `c` is as far as
# the suffix goes; zinit accepts a, b, c or none.
zinit ice wait"0c" lucid
zinit light "zdharma-continuum/fast-syntax-highlighting"

# Settings for plugins #

# enhancd
zinit ice wait'1' lucid pick'init.sh'
zinit light "b4b4r07/enhancd"
# export ENHANCD_FILTER="fzf --height 50% --reverse --ansi --preview 'ls -l {}' --preview-window down"
export ENHANCD_COMPLETION_BEHAVIOR=list
export ENHANCD_FILTER="fzf --height 50% --reverse --ansi  --info=inline --margin=1 --padding=1"
# export ENHANCD_DOT_SHOW_FULLPATH=1
TIPZ_TEXT='Alias:'


# Common Aliases (Plugin based)
################################

alias cdg=cd-gitroot
