#!/bin/bash

#Check if 1 arg was passed
if [ $# -ne 1 ]; then
    echo "Usage: $0 <host>"
    exit 1
fi

#Check if sshpass is installed
if ! command -v sshpass &> /dev/null; then
    echo "This module requires sshpass to be installed."
    exit 1
fi

#Attempt connection to host with password "nopassword" because it doesn't matter.
sshpass -p nopassword ssh -o ConnectTimeout=1 $1 echo "Hello!" &> /dev/null

# $? checks the exit status of the last command. 255 means no service found.
if [ $? -eq 255 ]; then
    echo "No ssh connection found."
    exit 1
fi

exit 0