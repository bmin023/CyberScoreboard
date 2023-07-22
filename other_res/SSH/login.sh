#!/bin/bash

#Check if 2 arg was passed
if [ $# -ne 2 ]; then
    echo "Usage: $0 <host> <username>:<password>"
    exit 1
fi

#split the username and password
USER=$(echo $2 | cut -d ':' -f 1)
PASS=$(echo $2 | cut -d ':' -f 2)

#Check if sshpass is installed
if ! command -v sshpass &> /dev/null; then
    echo "This module requires sshpass to be installed."
    exit 1
fi

#Attempt connection to host with password.
sshpass -p $PASS ssh -o ConnectTimeout=1 -o StrictHostKeyChecking=no $USER@$1 echo "Hello!" &> /dev/null

# $? checks the exit status of the last command.
RES=$?
if [ $RES -eq 255 ]; then
    echo "No SSH connection found."
    exit 1
fi

if [ $RES -ne 0 ]; then
    echo "SSH found, unable to login."
    exit 1
fi

exit 0
