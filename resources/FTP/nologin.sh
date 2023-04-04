#!/bin/bash

#Make sure only 1 arg was passed
if [ $# -ne 1 ]; then
    echo "Usage: $0 <host>"
    exit 1
fi

#Get the URL from arguments
URL=$1
#Open ftp, open the url then quit. If it can't open, timeout after 1 second.
timeout 1 ftp -v -n >/tmp/ftpc.log <<EOF
        open $URL
        quit
EOF
#Record if ftp timed out.
TRES=$?
#Open the log, see if the open was an invalid command.
tail -n1 /tmp/ftpc.log | grep "Invalid" #2>&1
RES=$?
#If invalid command or timed out, exit bad.
if [ $RES -eq 0 ] || [ $TRES -eq 124 ]
then
        exit 1
fi
#else exit good
exit 0
