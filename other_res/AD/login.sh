#/bin/bash

#Argument 1: host, 2: domain, 3: username:password, 4 (optional): user directory

#Check if ldapsearch is installed
if ! [ -x "$(command -v ldapsearch)" ]; then
  echo 'Error: ldap-utils is not installed.' >&2
  exit 1
fi

#Split username and password
USER=$(echo $3 | cut -d: -f1)
PASS=$(echo $3 | cut -d: -f2)

#Default user directory is Users at least for Baylor
if [ -z "$4" ]; then
    UDir="Users"
else
    UDir="$4"
fi

#Parse the domain name. myad.yay.com -> dc=myad,dc=yay,dc=com
Fdom=$(echo "dc=$2" | sed -r 's/[.]+/,dc=/g')

#Do the search. Very very finicky.
ldapsearch -x -H ldap://$1 -b $Fdom -D 'cn=$USER,cn=$UDir,$Fdom' -w $PASS > /dev/null

Res=$?

if [ $Res -eq 49 ]; then
    echo "Found AD, cannot login or bad domain"
    exit 1
fi

if [ $Res -ne 0 ]; then
    echo "Cannot find AD"
    exit 1
fi

echo 0