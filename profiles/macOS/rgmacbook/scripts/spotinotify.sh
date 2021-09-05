## Author: Rohit Goswami, 5/05/2021
## Email: rog32@hi.is
## Date: 5/05/2021
## License: MIT
## Description: Meant to be used in spotifyd settings
#!/usr/bin/env sh

# Needs:
# 1. spotifyd    -- https://github.com/Spotifyd/spotifyd
# 2. spotify-tui -- https://github.com/Rigellute/spotify-tui/
# 3. alerter     -- https://github.com/vjeantet/alerter

ANSWER="$(spt pb -s | alerter -actions alerter -actions Pause,Next,Previous,Like,Dislike -timeout 10)"
case $ANSWER in
    "@TIMEOUT") echo "Timeout man, sorry" ;;
    "@CLOSED") echo "You clicked on the default alert' close button" ;;
    "@CONTENTCLICKED") alacritty -e spt ;;
    "@ACTIONCLICKED") spt pb -t ;;
    "Pause") spt pb -t ;;
    "Next") spt pb -n ;;
    "Previous") spt pb -p ;;
    "Like") spt pb --like ;;
    "Dislike") spt pb --dislike ;;
    **) echo "? --> $ANSWER" ;;
esac
