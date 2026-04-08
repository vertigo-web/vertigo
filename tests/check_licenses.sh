#!/usr/bin/env bash

# Get the current year
CURRENT_YEAR=$(date +%Y)
EXIT_CODE=0

# Move to the project root if we are inside the tests folder
[[ -d ../crates ]] && cd ..

# Enable nullglob so the loop doesn't run if no files match
shopt -s nullglob

echo "Checking license years for $CURRENT_YEAR..."

for FILE in LICENSE-*; do
    if grep -q "$CURRENT_YEAR" "$FILE"; then
        echo "$FILE is up to date."
    else
        echo "$FILE is outdated! Could not find '$CURRENT_YEAR'." 1>&2
        EXIT_CODE=1
    fi
done

exit $EXIT_CODE
