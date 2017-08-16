import
  slacklib, asynchttpserver, asyncdispatch

const msgTurns = slackMsg("#general", "autonomic", "", "", "", "", "It freakin worked!")

echo msgTurns

proc slackServerRun*(slackReq: asynchttpserver.Request) {.async.} =
  # To change the standard port:
  slackPort = Port(65533)

  # Case the command
  case slackEvent(slackReq, "command"):
  of "/turns":
    # If you need to run a proc with the arguments sent, access the 'text' field:
    echo slackEvent(slackReq, "text")
    await slackRespond(slackReq, msgTurns)

  of "/vote":
    echo "Voting!"
    await slackRespond(slackReq, slackMsg("#general", "autonomic", "", "", "", "", "Vote worked!"))

  else:
    echo "something else!"
    await slackRespond(slackReq, slackMsg("#general", "autonomic", "ERROR", "", "danget", "", "wrongo commando"))

waitFor slackServer.serve(slackPort, slackServerRun)
