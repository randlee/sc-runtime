#!/bin/sh
# The leader exits at once. The grandchild inherits stdout and stderr and
# holds both open, so a drain with no host deadline stalls until it dies.
dir=$(dirname "$0")
printf '%s\n' "$$" > "$dir/leader.pid"
sleep 3600 &
printf '%s\n' "$!" > "$dir/child.pid"
exit 0
