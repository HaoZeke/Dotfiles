#######################
# Startup debugging   #
# tools for profiling #
#######################

# Start Profiler
# Test with:
# time  zsh -i -c exit
# To check without results try:
# for i in $(seq 1 10); do time zsh -i -c exit; done
if [[ "${ZSH_PROFILE}" == 1 ]]; then
    zmodload zsh/zprof
else
    ZSH_PROFILE=0
fi
