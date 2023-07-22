#!/bin/bash

# This script is used to simulate a slow script
# It will sleep for 4 seconds

sleep $1

# Generate a random number between 0 and 100
# If the number is less than 50, exit with a failure
# Otherwise, exit with a success

if [ $((RANDOM % 100)) -lt 50 ]; then
    exit 1
else
    exit 0
fi
