# CyberScoreboard
Made for use in CCDC type games. It runs a series of checkers against different teams. The server is written in rust, the web app is written in typescript (react), and the checkers are written in whatever language you like (mostly bash and python)

The idea of this scoreboard is that it is a platform to build on with your own checkers and scripts. The scoreboard is just a runner and a pretty ui.

# Requirements
1. Be on a linux machine, it works on macos, but I don't see why you would do that.
2. Have rust installed and working (I'll make a releases page maybe but for now you'll have to build it yourself)
3. Have bash installed in /bin/bash.
4. Whatever checkers you are using, make sure you have their tools installed. For instance, the port scan checker relies on netcat, so if you want to use it, make sure you have netcat installed. The minecraft checker is written in python, so if you want to use it, have python installed.
# Running the Scoreboard
1. `git clone https://github.com/bmin023/CyberScoreboard.git`
2. `cd CyberScoreboard`
3. Make a services.yaml and a teams.yaml file in the resources directory. If you're just testing to see if it works, they can be blank, but they have to be present.
4. `cargo run -r`
You should now have a the scoreboard up and running at http://localhost:8000
# Customizing your services.yaml
The services.yaml file is where you declare all the services you want scored. Each line contains a name followed by the command to check that service. An example is given below.
```yaml
# example services.yaml
ssh: SSH/nologin.sh 192.168.7.21
website: WEB/curlfind.sh cool.com "This is so cool"
```
The above services.yaml creates two services. The first is a service called ssh which calls the script resourcs/SSH/nologin.sh with the argument '192.168.7.21'. nologin.sh was made to attempt logging into the ssh server at the domain it was provided and make sure that the server properly handles that. In the background, the scoreboard is running the command `/bin/bash -c SSH/nologin.sh 192.168.7.21` with the cwd set to the resources folder every tick.
The second service is named website and it searches for the string "This is so cool" in the website "cool.com".

These are just bash commands with associated names. Every tick, it runs those commands and checks their exit codes. If they are 0, the service is up that tick. If not, the service is down. You can look into the resources folder to see examples of scripts that work with the scoreboard. 
# Customizing your teams.yaml
The teams.yaml file is where you declare all the teams playing in the game. Each block is started with a team name and contains all the environment variables for that team. An example is given below.
```yaml
# example teams.yaml
team_1:
    SSH_SERVER: 192.168.1.22
    WEBSITE: team1.com
team_2:
    SSH_SERVER: 192.168.2.22
    WEBSITE: team2.com
```
This creates two teams with different environments. It is easier to understand why we set it up this way when you see the service.yaml that could compliment this setup.
```yaml
ssh: SSH/nologin.sh $SSH_SERVER
website: WEB/curlfind.sh $WEBSITE "This is so cool"
```
In the background, the scoreboard runs each checker multiple times, once for each team. It swaps out the environment it uses depending on the team so that SSH_SERVER or whatever you declare will be replaced with the correct string for that team.
# Writing your own Checkers
Go look in the resources folder for examples of how to write scipts that work with the scoreboard. But basically:

You must
- exit 0 if the checker succeeds
- exit anything else if it fails

Out of convention,
- error out if the correct tools weren't installed to use your checker
- error out if incorrect arguments were passed to your checker
- if you do error out, please put a reason why in either stdin or sterr. The scoreboard records these.
- if you take in a username and/or password as an argument, accept it in the form `username:password`, this way it will work with the scoreboard's password functionality.
- name the scripts in a way that it clues you in to its functionality. For instance curlfind.sh curls a websites and attempts to find a string. matchdesc.py checks if a servers description matches the one provided.
- put the scripts in the folder that matches their services. SSH scripts in the SSH folder. Web scripts in the WEB folder.
# Contributions
Please contribute. Especially checkers. If it seems like it can be used again, send it here.

If you know rust, there's still work to be done. Primarily documentation, the inject feature, and working on speed.
