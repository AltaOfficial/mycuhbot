const { EmbedBuilder } = require("discord.js");
const { getDb } = require("../db");
const { checkMyccount } = require("./myccount/myccountUtils");

const REQUEST_STATE = {
  INCOMING: 0,
  OUTGOING: 1,
};

let requestData = {};

/**
 * Sets database connection
 * @param {*} databaseConnection
 * @param {*} serverClient
 */
async function setServerDetails() {
  dbConnection = getDb();
}

async function getRequestData() {
  return requestData;
}

async function setRequestData(object) {
  requestData = object;
}

async function deleteAllMessagesInChannel(channel) {
  let fetchedMessagesData;
  do {
    fetchedMessagesData = await channel.messages.fetch({ limit: 100 });
    channel.bulkDelete(fetchedMessagesData, true);
  } while (fetchedMessagesData > 1);
}

/**
 * Handle transaction requests
 * @param {*} transactionMessage message object of message sent to transfers channel
 * @param {*} sendingUser
 * @param {*} receivingUser
 * @param {*} requestDetails
 * @param {*} mycuhBuckRequest type of request (request, transfer, redemption)
 * @returns
 */
async function newRequest(
  transactionMessage,
  sendingUser,
  receivingUser,
  requestDetails,
  requestType
) {
  const EXPIRE_TIME = 300; // 5 minutes

  await new Promise((resolve, reject) => {
    dbConnection.collection("users").updateOne(
      { discordId: sendingUser.user.id },
      {
        $set: {
          pendingOutgoingRequest: {
            messageId: transactionMessage.id,
            requestReason: requestDetails.requestReason,
            requestAmount: requestDetails.requestAmount,
            requestQuantity: requestDetails.requestQuantity,
            requestProductName: requestDetails.requestProductName,
            requestTotalCost: requestDetails.requestTotalCost,
          },
        },
      },
      { upsert: true }
    );
    if (requestType == "transfer") {
      dbConnection.collection("users").updateOne(
        { discordId: receivingUser.user.id },
        {
          $set: {
            pendingIncomingRequest: {
              messageId: transactionMessage.id,
              requestReason: requestDetails.requestReason,
              requestAmount: requestDetails.requestAmount,
            },
          },
        },
        { upsert: true }
      );
    }
    resolve("Success");
  });

  // handling request expiration
  let messageTimer = setTimeout(async () => {
    let transferEmbed = new EmbedBuilder();
    let noConnectedUser = false;
    await new Promise((resolve, reject) => {
      dbConnection
        .collection("users")
        .findOne({ "pendingOutgoingRequest.messageId": transactionMessage.id })
        .then((document) => {
          if (!document) {
            resolve("Not found");
          } else {
            resolve("Found");
          }
        });
    }).then((result) => {
      if (result == "Not found") {
        noConnectedUser = true;
      }
    });

    if (!noConnectedUser) {
      await new Promise((resolve, reject) => {
        clearPendingRequest(sendingUser.user.id, REQUEST_STATE.OUTGOING);

        if (requestType == "transfer") {
          clearPendingRequest(sendingUser.user.id, REQUEST_STATE.INCOMING);
        }
        resolve("Success");
      });

      switch (requestType) {
        case "transfer":
          transferEmbed
            .setTitle("Mycuh Bucks Transfer Request")
            .setDescription(
              `<@${
                sendingUser.user.id
              }> is requesting **${requestDetails.requestAmount.toString()}** mycuh buck(s) from <@${
                receivingUser.user.id
              }> to their myccount\nReason: ${
                requestDetails.requestReason
              }\nUser Response: **Expired** 🕑`
            );
          transactionMessage.edit({ embeds: [transferEmbed], components: [] });
          break;

        case "request":
          transferEmbed
            .setTitle("Requesting Mycuh Bucks")
            .setDescription(
              `<@${
                sendingUser.user.id
              }> is requesting to add **${requestDetails.requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${
                requestDetails.requestReason
              }\nMycuh Response: **Expired** 🕑`
            );
          transactionMessage.edit({
            content: "",
            embeds: [transferEmbed],
            components: [],
          });
          break;
        case "redemption":
          console.log("expired");
          transferEmbed.setTitle("Redeeming Mycuh Product").setDescription(
            `<@${sendingUser.user.id}> is requesting to redeem **${requestDetails.requestProductName}** 
        Quantity: **${requestDetails.requestQuantity}**\nMycuh Response: **Expired** 🕑`
          );
          transactionMessage.edit({
            content: "",
            embeds: [transferEmbed],
            components: [],
          });
          break;
      }
    }
  }, EXPIRE_TIME * 1000);

  return messageTimer;
}

async function checkPendingRequest(incomingOrOutgoing, discordUser) {
  let response;
  await checkMyccount(discordUser);

  await new Promise((resolve, reject) => {
    dbConnection
      .collection("users")
      .findOne({ discordId: discordUser.user.id })
      .then((document) => {
        if (incomingOrOutgoing == REQUEST_STATE.INCOMING) {
          if (document.pendingIncomingRequest.messageId == "") {
            resolve(false);
          } else {
            resolve(true);
          }
        } else if (incomingOrOutgoing == REQUEST_STATE.OUTGOING) {
          if (document.pendingOutgoingRequest.messageId == "") {
            resolve(false);
          } else {
            resolve(true);
          }
        }
      });
  }).then((value) => {
    response = value;
  });

  return response;
}

async function clearPendingRequest(discordUserId, incomingOrOutgoing) {
  if (incomingOrOutgoing == REQUEST_STATE.INCOMING) {
    await new Promise((resolve, reject) => {
      dbConnection.collection("users").updateOne(
        { discordId: discordUserId },
        {
          $set: {
            "pendingIncomingRequest.messageId": "",
          },
        },
        { upsert: true }
      );
      resolve();
    });
  } else {
    await new Promise((resolve, reject) => {
      dbConnection.collection("users").updateOne(
        { discordId: discordUserId },
        {
          $set: {
            "pendingOutgoingRequest.messageId": "",
          },
        },
        { upsert: true }
      );
      resolve();
    });
  }
}

async function clearAllPendingRequests() {
  // Expire past pending requests
  await new Promise((resolve, reject) => {
    dbConnection.collection("users").updateMany(
      {},
      {
        $set: {
          pendingIncomingRequest: {
            messageId: "",
            requestReason: "",
            requestAmount: 0,
          },
          pendingOutgoingRequest: {
            messageId: "",
            requestReason: "",
            requestAmount: 0,
          },
        },
      }
    );
    resolve("Success");
  });
}

function ensureType(type, text) {
  switch (type) {
    case "integer":
      try {
        if (parseInt(text).toString() == "NaN") {
          throw new Error("error changing text to number");
        } else if (parseInt(text) < 1) {
          throw new Error("Number cannot be a negative");
        }
      } catch (err) {
        return { success: false };
      }
      return { success: true, value: parseInt(text) };

    case "string":
      try {
        text.toString();
      } catch (err) {
        return { success: false };
      }
      return { success: true, value: text };

    case "boolean":
      if (text.toLowerCase() == "true" || text.toLowerCase() == "yes") {
        return { success: true, value: true };
      } else if (text.toLowerCase() == "false" || text.toLowerCase() == "no") {
        return { success: true, value: false };
      }
      return { success: false };
    default:
      throw new Error("ensureType type is invalid");
  }
}

exports.ensureType = ensureType;
exports.deleteAllMessagesInChannel = deleteAllMessagesInChannel;
exports.setServerDetails = setServerDetails;
exports.clearPendingRequest = clearPendingRequest;
exports.checkPendingRequest = checkPendingRequest;
exports.clearAllPendingRequests = clearAllPendingRequests;
exports.newRequest = newRequest;
exports.getRequestData = getRequestData;
exports.setRequestData = setRequestData;
exports.REQUEST_STATE = REQUEST_STATE;
