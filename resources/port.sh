#!/bin/bash

#Check if netcat is installed
if ! [ -x "$(command -v nc)" ]; then
  echo 'Error: netcat is not installed.' >&2
  exit 1
fi

#Check there is 2 arguments
if [ $# -ne 2 ]; then
  echo 'usage: port.sh <host> <port>' >&2
  exit 1
fi

nc -zw1 $1 $2

exit $?