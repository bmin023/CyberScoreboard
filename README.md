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
You should now have the scoreboard up and running at http://localhost:8000
# Customizing your services.yaml
The services.yaml file is where you declare all the services you want scored. Each line contains a name followed by the command to check that service. An example is given below.
```yaml
# example services.yaml
ssh: SSH/nologin.sh 192.168.7.21
website: WEB/curlfind.sh cool.com "This is so cool"
```
The above services.yaml creates two services. The first is a service called ssh which calls the script resources/SSH/nologin.sh with the argument '192.168.7.21'. nologin.sh will attempt logging into the ssh server at the domain it was provided and make sure that the server properly handles that. In the background, the scoreboard is running the command `/bin/bash -c SSH/nologin.sh 192.168.7.21` with the cwd set to the resources folder every tick.
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
This creates two teams with different environments. It is easier to understand why we set it up this way when you see the service.yaml that could compliment this.
```yaml
ssh: SSH/nologin.sh $SSH_SERVER
website: WEB/curlfind.sh $WEBSITE "This is so cool"
```
In the background, the scoreboard runs each checker multiple times, once for each team. It swaps out the environment it uses depending on the team so that SSH_SERVER or whatever you declare will be replaced with the correct string for that team.

# Customizing your injects.yaml
The injects.yaml file is the only not required file of the main configs. It is also the most involved. It is formatted in the same manner where
you have a name followed by its values, but there are many more values.
- start: How many minutes into the game the inject should start. By default, it starts at the beginning of the game.
- duration: How many minutes the inject should last. If not present, the inject will be marked as sticky and will not end.
- file_types: A list of file extensions that the inject will accept as submissions. If not present, the inject will accept any file type.
- no_submit: A boolean. If true, the inject will not accept submissions. False by default.
- markdown: A string of markdown that will be rendered as the inject's description. It as accepts team environment variables in the form {{ VARIABLE_NAME }}.
- side_effects: A special list of commands that will activate when the inject ends.

An example is given below.
```yaml
# example injects.yaml

# Normal inject accepting text files
Inject_1:
    start: 30
    duration: 60
    file_types:
        - .txt
        - .docx
        - .pdf
    markdown: |
        # Your First Inject
        You will submit a text file containing a flag you can find in
        {{ TEAM_FLAG_SERVER }}.
        It's funny writing markdown in yaml in markdown.
        **Good Luck!**
# A Sticky Notification
# You can create links as well, which can be useful for downloading files.
Inject_2:
    no_submit: true
    markdown: |
        # Welcome to the Game
        You're passwords are stored here:
        [Passwords](https://{{ SB_URL }}//downloads/passwords/{{ TEAM_NAME }}.csv)
# A Side Effect Inject
# This inject will:
#  1. Delete the service named SSH
#  2. Create a new service called HSS
#  3. Edit the Website service to be called "Cool Website"
Inject_3:
    start: 20
    duration: 10
    markdown: |
        # Get Ready for the Scoreboard to Change!
        You're website better be cool by the time this timer
        reaches zero.
    no_submit: true
    # The difference between deleting a service then adding it versus
    # editing it is that deleting it removes the history and the score
    # (from the scoreboard's perspective. The team keeps the points)
    # while editing it keeps the history and stats from the old service.
    side_effects:
        - !DeleteService SSH
        - !AddService
            name: HSS
            command: SSH/nologin.sh $SSH_SERVER
            multiplier: 1
        - !EditService
            - Website
            - name: Cool Website
              command: WEB/curlfind.sh $WEBSITE "This website is cool now"
              multiplier: 1
```
The second example has a download link it it for extra files. This works because
eveything in the resources/downloads folder is served at the url /downloads.

# Passwords
Credentials were a bit of a challenge to implement because the checks that use these credentials are entirely separate from the scoreboard.
What I eventually decided on was that credentials would be stored in files in the resources/PW/TEAM_NAME file.
You can declare multiple groups by creating multiple files. For instance SSH.pw would hold the SSH credential for that team.

Each file is a list of username password pairs in the form of `USERNAME:PASSWORD` separated by newlines. If password groups are found in these locations,
teams will be ables to edit their passwords (but not see which are set) on their team page.

It is useful to you how you use these in your checkers. The best way I've found is to take in a username password pair as an argument. Then I can declare
a service like so:
```yaml
# shuf is a coreutils command that randomly selects a line from a file.
ssh: SSH/login.sh $SSH_SERVER $(shuf -n 1 PW/$TEAM_NAME/SSH.pw)
```

# Environment Variables
There are a few environment variables that will affect how the scoreboard runs. 
Mostly just where it will look for different files.
- SB_RESOURCE_DIR: The directory where all resources are stored. This will by the working directory for the scoreboard and where all checkers are run from. Defaults to a folder named 'resources' in your current working directory.
- SB_PORT: What port the scoreboard will listen on. 8000 by default.
- SB_TEAMS: The name of the teams config. Defaults to teams.yaml
- SB_SERVICES: The name of the services config. Defaults to services.yaml
- SB_INJECTS: The name of the injects config. Defaults to injects.yaml
- SB_APP_DIR: Where the React SPA is located. By default it is the public folder in your current working directory.
- SB_ADMIN_PASSWORD: The password for the admin account on the scoreboard. Not set by default.

# Scoreboard Passwords
Separate from scoreboard passwords, you can set passwords for different teams in the
game along with the admin account. The admin account is set through the environment variable `SB_ADMIN_PASSWORD`. The rest are set in the team config using the `TEAM_PASSWORD` environment variable. Below is an example of what that could look like.

```yaml
TEAM1:
  ENV1: foo
  TEAM_PASSWORD: securePassword90!
TEAM2:
  ENV1: bar
  TEAM_PASSWORD: superSecret87!
```

# Admin Page
There is an admin page at the /admin. Nearly everything that can be configured in the files can also be configured
in the admin page at runtime. Services, Teams, Injects, Passwords, and Saving/Loading functionality can all be found there.
It is still recommended that you configure in the config files and leave the admin page for quick changes during the game.
That being said, the admin page is very useful for testing and debugging and is a fully functional alternative to the config files.

Just make sure you use the save button.

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
