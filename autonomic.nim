# database storage of state
# functions to interact with state
# convert time from UTC to ...
# slash commands
#   turns
#     returns current person and time, next person and earliest time
#     in players local time
#     button to share with channel
#     shares in UTC and all other timezones (randomised order)
#   /settimezone
#     possible timezones
#     timezone
#   voting
#     '/vote' returns argument template
#     proposal number
#     Y/N
#     check proposal
#   proposal
#     template
#     number
#     content
#       transmute/amend/enact/repeal
#         rule number
#         text
#       or just text
#     check turn
#   roll
#     template
#     print with updated score
#   messages reference rule number that dictates their workings
#   automatically update wiki?

# make code concise but readable and clearly structured
# nomic module
# scoreboardinit.sql to initiliase scoreboard from wiki

import
  slacklib, nomicdb


proc turns*(args: string): string =
  ## returns the name of current President,
  ##  the next President and the ealiest time the current turn can end.
  ##  (According to rule xxx)
  let
    (currentPresident, turnStartTime) = nomicdb.getCurrentTurn()
    nextPresident = nomicdb.getNextPresident()
  return currentPresident & " " & turnStartTime & " " & nextPresident

#echo turns("thingz")
