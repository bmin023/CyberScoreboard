#!/bin/bash

#Check if 2 args were passed
if [ $# -ne 2 ]; then
    echo "Usage: $0 <website> <search query>"
    exit 1
fi

#Check if curl is installed
if ! command -v curl &> /dev/null; then
    echo "This module requires curl to be installed."
    exit 1
fi

#Curl the website
WEB=$(curl $1 2>&1)

if [ $? -ne 0 ]; then
    echo "No website found at $1"
    exit 1
fi

#grep to find the search query
echo "$WEB" | grep -i "$2" &> /dev/null

if [ $? -ne 0 ]; then
    echo "Website up, but can't find $2"
    exit 1
fi

exit 0