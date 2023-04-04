# Resources
This is the folder for all the different checkers and configs for the judge. In [services.yaml](services.yaml), you write the name of your service and a bash command for it to execute. The command will be executed and the results will be saved in the server.
## Why Bash Commands?
The beauty of using bash commands over a single script that has various checkers is that this system is entirely modular. You can add new checkers without having to understand how any other checkers or the judge works.
It also means that a lot of functionality is already built into the judge without having to write a lot of code.
### Password Files
Another thing that comes with bash is the ability to resolve commands. This can be used to randomly pick a username and password to check for.
Any checker that takes in a username and password in the form `<username>:<password>` you can instead replace with `$(shuf -n 1 <password_file>)`. This will randomly pick a password from that file.
(I would suggest putting all password files in the same folder, [PW](PW) with the file ending in `.pw`) in case I can get password change requests working in the future.
### Enviornment Variables
Your bash commands can read (and write) enviornment variables. If you want to switch a checker from http to https halfway through the scenario, simply set the checker to `WEB/curlfind.sh $WEBSITE_URL "My Title"`. Enviornment variables are set in the form `<variable_name>="<value>"` You can do that in a console by typing `export <variable_name>="<value>"` or typing them into a file named .env in the [resources folder](../resources/.env). It is better practice to write them in the .env file. So that I can write something on the website to change them. (For now, stick with only strings in enviornment variables. That way it is easier to parse)
## Writing Checkers
You can write these checkers in any language you want, as long as it can be executed by bash. The judge checks to see what exit code the checker returns. If it returns 0, the checker passes. If it returns 1, the checker fails.
There is also a 2 second timeout implemented in the judge. If the checker takes longer than 2 seconds to execute, judge assumes it failed. If your checker is taking longer than 2 seconds to execute, something is probably wrong anyways.
# Checkers
- Miscellaneous
  - [x] Check if port is open on host `./port.sh <host> <port>`
  - [x] Returns bad `"true"`
  - [x] Returns good `"false"`
  - [x] Returns randomly `./rand.py <1/n chance of bad>`
- SSH
  - [x] No Login `SSH/nologin.sh <host>`
  - [x] Login `SSH/login.sh <host> <username>:<password>`
- Minecraft
  - [x] Ping Server for Description and Match `MC/matchdesc.py <host> <Description>`
- Web
  - [x] Access Webpage and check if word is in it `WEB/curlfind.sh <host> <word>`
  - [ ] Get Webpage and shasum it against a file
- FTP
  - [x] No Login `FTP/nologin.sh <host>` **NOT WORKING IN DOCKER RIGHT NOW LOOK INTO THAT**
  - [ ] Login, Check for data
- Databases (MySQL, PostgreSQL)
  - [ ] No Login
  - [ ] Login, Check for data
- AD
  - [x] Query AD `AD/nologin.sh <host>`
  - [x] Login to AD `AD/login.sh <host> <domain> <username>:<password> (user directory)`
- DNS
  - [x] nslookup `nslookup <host>`
- Mail (idk for any of these honestly)
  - [ ] SMPT
  - [ ] POP3
  - [ ] IMAP