import
  slacklib, asynchttpserver, asyncdispatch, autonomic


const
  msgTurns = slackMsg("#general", "autonomic", "fallback", "pretext", "0x000000", "title", "It freakin worked!")
  slackPort = Port(65533)

  OAUTHtoken = "xoxp-227550644838-226658542578-228075433093-dfd35cc7e3e6719477278bdfefb58931"


#proc roll(user: string): int =



## Using slacklib to handle requests from slack
proc slackServerRun*(slackReq: asynchttpserver.Request) {.async.} =

  echo slackReq.body

  # Case the command
  case slackEvent(slackReq, "command"):
  of "/turns":
    # If you need to run a proc with the arguments sent, access the 'text' field:
    var
      args = slackEvent(slackReq, "text")
      responseMsg = turns(args)
    echo args
    echo responseMsg
    await slackRespond(slackReq, responseMsg)

  of "/proposal":
    echo "proposal"
    var
      user = slackEvent(slackReq, "user")
      proposalStr = slackEvent(slackReq, "text")


  of "/vote":
    echo "Voting!"
    await slackRespond(slackReq, slackMsg("#general", "autonomic", "", "", "", "", "Vote worked!"))

  #of "/roll":
  #  echo "rolling"
  #  newRoll = slackEvent(slackReq, "user").roll()
  #  await slackRespond(slackReq, rollMsg(newRoll))

  else:
    echo "something else!"
    await slackRespond(slackReq, slackMsg("#general", "autonomic", "ERROR", "", "danget", "", "wrongo commando"))


## Run server loop
waitFor slackServer.serve(slackPort, slackServerRun)
