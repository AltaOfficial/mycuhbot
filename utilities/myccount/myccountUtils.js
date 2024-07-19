const { ObjectId } = require("mongodb");
const { getDb } = require("../../db");
/**
 * Gets myccount of provided user.
 * @param {*} discordUser
 * @returns myccount document from database
 */
async function getMyccount(discordUser) {
  let dbConnection = getDb();
  await checkMyccount(discordUser);
  let response;

  if (discordUser == undefined) {
    throw new Error("Discord id was not defined");
  }

  await new Promise((resolve, reject) => {
    dbConnection
      .collection("users")
      .findOne({ discordId: discordUser.user.id })
      .then((document) => {
        resolve(document);
      });
  }).then((value) => {
    response = value;
  });

  return response;
}

/**
 * Returns amount of mycuh bucks the provided user has.
 * @param {*} discordUser
 * @returns
 */
async function getMycuhBucks(discordUser) {
  let document = await getMyccount(discordUser);
  if (document == null) {
    return null;
  }
  return document.bucks;
}

/**
 * Checks to see if users discord id is connected to a myccount
 * or creates one if one doesnt exist.
 * @param {*} discordUser
 * @returns "Success" or "Created" status
 */
async function checkMyccount(discordUser) {
  let response;

  if (discordUser == undefined) {
    throw new Error("Discord id was not defined");
  }
  await new Promise(async (resolve, reject) => {
    await dbConnection
      .collection("users")
      .findOne({
        discordId: discordUser.user.id,
      })
      .then((document) => {
        if (document == null) {
          let objId = new ObjectId();
          let user = {
            _id: objId,
            username: discordUser.user.username,
            discordId: discordUser.user.id,
            bucks: 0,
            pendingIncomingRequest: {
              messageId: "",
              requestReason: "",
              requestAmount: 0,
            },
            pendingOutgoingRequest: {
              messageId: "",
              requestReason: "",
              requestAmount: 0,
              requestQuantity: 0,
              requestProductName: "",
              requestTotalCost: 0,
            },
          };

          dbConnection.collection("users").insertOne(user);
          resolve("Created");
        }
        resolve("Success");
      });
  }).then((value) => {
    response = value;
  });
  return response;
}

async function changeMycuhBucks(messageId, requestAmount, isRemoving) {
  if (isRemoving) {
    await new Promise(async (resolve, reject) => {
      // removes bucks
      await dbConnection
        .collection("users")
        .findOne({ "pendingIncomingRequest.messageId": messageId })
        .then((document) => {
          if (document) {
            dbConnection.collection("users").updateOne(
              { "pendingIncomingRequest.messageId": messageId },
              {
                $set: {
                  bucks: document.bucks - requestAmount,
                  "pendingIncomingRequest.messageId": "",
                },
              }
            );
          } else {
            console.log("user not found");
          }
        });
      resolve();
    });
  } else {
    await new Promise(async (resolve, reject) => {
      // adds bucks
      await dbConnection
        .collection("users")
        .findOne({ "pendingOutgoingRequest.messageId": messageId })
        .then((document) => {
          if (document) {
            console.log(document);
            dbConnection.collection("users").updateOne(
              { "pendingOutgoingRequest.messageId": messageId },
              {
                $set: {
                  bucks: document.bucks + requestAmount,
                  "pendingOutgoingRequest.messageId": "",
                },
              }
            );
          } else {
            console.log("user not found");
          }
        });
      resolve();
    });
  }
}

exports.getMycuhBucks = getMycuhBucks;
exports.changeMycuhBucks = changeMycuhBucks;
exports.checkMyccount = checkMyccount;
