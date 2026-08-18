# For ssh-add
#
# Test the socket before scanning the process table. `pgrep` walks /proc and
# costs ~43 ms per shell here, which every terminal pane pays; `[ -S ]` is a
# builtin and costs nothing. It is also the more accurate question: an agent
# this shell cannot reach through SSH_AUTH_SOCK is of no use to it.
#
# Scoped to this user, because another user's agent satisfies `pgrep -x` while
# leaving this session without one. The UID comes from EUID rather than USER:
# both bash and zsh set it without a fork, and an empty USER makes pgrep reject
# the whole option and report no agent, which spawns one every single time.
# Two shells starting at the same instant can still both miss and both spawn,
# but the window is now the fallback path rather than every shell.
if [ ! -S "${SSH_AUTH_SOCK:-}" ]; then
  if ! pgrep -u "${EUID:-$(id -u)}" -x ssh-agent >/dev/null 2>&1; then
    eval "$(ssh-agent -s)"
    echo "Started ssh-agent with PID: $SSH_AGENT_PID"
  fi
fi

# Rclone Config
RCLONE_STATS=5s

# Common Configuration
######################

export SSH_ASKPASS=lxqt-openssh-askpass
export PAGER=less
# Less status line
export LESS='-R -f -X -i -P ?f%f:(stdin). ?lb%lb?L/%L.. [?eEOF:?pb%pb\%..]'
export LESSCHARSET='utf-8'

# LESS man page colors (makes Man pages more readable).
export LESS_TERMCAP_mb=$'\E[01;31m'
export LESS_TERMCAP_md=$'\E[01;31m'
export LESS_TERMCAP_me=$'\E[0m'
export LESS_TERMCAP_se=$'\E[0m'
export LESS_TERMCAP_so=$'\E[00;44;37m'
export LESS_TERMCAP_ue=$'\E[0m'
export LESS_TERMCAP_us=$'\E[01;32m'

# ls command colors
export LSCOLORS=exfxcxdxbxegedabagacad

# declare the environment variables
export CORRECT_IGNORE='_*'
export CORRECT_IGNORE_FILE='.*'

export WORDCHARS='*?_-.[]~=&;!#$%^(){}<>'
export WORDCHARS='*?.[]~&;!#$%^(){}<>'
