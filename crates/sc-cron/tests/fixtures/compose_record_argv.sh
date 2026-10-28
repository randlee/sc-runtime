#!/bin/sh
# Checked-in compose stand-in. Side files land beside the exec path (`$0`),
# which tests set to a per-run symlink, not beside this file.
printf '%s\n' "$@" > "$(dirname "$0")/argv.txt"
printf 'rendered'
