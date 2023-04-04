#!/usr/bin/python3
import sys

if __name__ == '__main__':
    import random
    #get arguments from command line
    args = sys.argv
    #check if there are enough arguments
    if len(args) != 2:
        print("Usage: ./rand.py <number>")
        sys.exit(1)
    #check if the argument is a number
    try:
        number = int(args[1])
    except ValueError:
        print("Error: argument is not a number")
        sys.exit(1)
    number = abs(number)
    rand = random.randint(1, number)
    print(rand)
    if(rand == 1):
        exit(1)
    else:
        exit(0)
