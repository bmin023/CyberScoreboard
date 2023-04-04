#/bin/bash

#Argument 1: host

#Check if ldapsearch is installed
if ! [ -x "$(command -v ldapsearch)" ]; then
  echo 'Error: ldap-utils is not installed.' >&2
  exit 1
fi


#Do the search. Very very finicky.
ldapsearch -x -H ldap://$1

Res=$?

#0 is success, 1 means server found, but couldn't bind
#which is fine if no login is needed
if [ $Res -ne 0 ] || [ $Res -ne 1 ]; then
    echo "No AD found"
    exit 1
fi

echo 0