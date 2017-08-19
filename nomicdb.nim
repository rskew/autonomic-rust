import
  db_sqlite, strutils


var
  db = db_sqlite.open("nomicdb.db", nil,nil,nil)


## Initialise db if empty
when true:
  db.exec(sql"Drop table if exists turns")
  db.exec(sql("""CREATE TABLE turns (
        id        INTEGER PRIMARY KEY,
        playerid  INTEGER,
        starttime VARCHAR(50))"""))
  db.exec(sql"INSERT INTO turns (id,playerid,starttime) VALUES (?,?,?)",1,1,50)
  db.exec(sql"INSERT INTO turns (id,playerid,starttime) VALUES (?,?,?)",2,2,70)

when true:
  db.exec(sql"Drop table if exists players")
  db.exec(sql("""CREATE TABLE players (
        id        INTEGER PRIMARY KEY,
        handle    VARCHAR(50),
        timezone  VARCHAR(50),
        score     INTEGER)"""))
  db.exec(sql"INSERT INTO players (id,handle,timezone,score) VALUES (?,?,?,?)",1,"faceperson","MUMBAI",50)
  db.exec(sql"INSERT INTO players (id,handle,timezone,score) VALUES (?,?,?,?)",2,"otherplayer","SWIZZ",10)
  db.exec(sql"INSERT INTO players (id,handle,timezone,score) VALUES (?,?,?,?)",3,"newface","AAA",-10)


proc getCurrentTurn*(): (string, string) =
  let
    thisTurn = db.getRow(sql"SELECT playerid, starttime FROM turns ORDER BY ID DESC LIMIT 1")
    player = db.getRow(sql"SELECT handle FROM players WHERE id=?", thisTurn[0])[0]
    starttime = thisTurn[1]
  return (player, starttime)

proc getNextPresident*(): string =
  let
    thisPresidentId = db.getRow(sql"SELECT playerid FROM turns ORDER BY ID DESC LIMIT 1")[0].parseInt()
    nPlayers = db.getRow(sql"SELECT COUNT(*) FROM players")[0].parseInt()
    nextPresidentId = thisPresidentId + 1 mod nPlayers
    nextPresident = db.getRow(sql"SELECT handle FROM players WHERE id=?", nextPresidentId)[0]
  return nextPresident


## table players [playerid, handle, timezone, score]
## table rules [ruleid, votes-for, votes-against, timestamp]
## table turns [turnid, playerid, starttime]
## table votes [voteid, playerid, timestamp, Y/N]
## table proposals [proposalid, playerid, timestamp, transmute-or-not, content]
#
#let theDb = db.open("mytest.db", nil, nil, nil)
#
##theDb.exec(sql"Drop table if exists myTestTbl")
##theDb.exec(sql("""create table myTestTbl (
##     Id    INTEGER PRIMARY KEY,
##     Name  VARCHAR(50) NOT NULL,
##     i     INT(11),
##     f     DECIMAL(18,10))"""))
#
##theDb.exec(sql"BEGIN")
##for i in 1..1000:
##  theDb.exec(sql"INSERT INTO myTestTbl (name,i,f) VALUES (?,?,?)",
##        "Item#" & $i, i, sqrt(i.float))
##theDb.exec(sql"COMMIT")
#
#for x in theDb.fastRows(sql"select * from myTestTbl"):
#  echo x
#
##let id = theDb.tryInsertId(sql"INSERT INTO myTestTbl (name,i,f) VALUES (?,?,?)",
##      "Item#1002", 1002, sqrt(1002.0))
##echo "Inserted item: ", theDb.getValue(sql"SELECT name FROM myTestTbl WHERE id=?", id)
#
#theDb.close()
