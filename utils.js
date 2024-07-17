const { ObjectId } = require("mongodb");
const { getDb } = require("./db");
const {
  EmbedBuilder,
  ActionRowBuilder,
  ButtonBuilder,
  ButtonStyle,
  ModalBuilder,
  TextInputBuilder,
  TextInputStyle,
} = require("discord.js");

const REQUEST_STATE = {
  INCOMING: 0,
  OUTGOING: 1,
};

let mycuhProductsList = [];

// this file has gotten so fat man...

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
  await new Promise((resolve, reject) => {
    dbConnection
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
/**
 * Sets database connection and discord server
 * client global variables.
 * @param {*} databaseConnection
 * @param {*} serverClient
 */
async function setServerDetails(databaseConnection, serverClient) {
  dbConnection = databaseConnection;
  client = serverClient;
}

async function embedMycuhPrices() {
  let mycuhPricesChannel = await client.channels.cache.get(
    process.env.MYCUH_PRICES_CHANNEL_ID
  );
  mycuhProductsList = [];

  let mycuhProducts = await dbConnection.collection("products").find();

  // TODO: see in collection "mycuhProducts" in the products document whether there is a sale and for what amount off
  let mycuhPricesEmbed = new EmbedBuilder();
  if (await mycuhProducts.hasNext()) {
    let num = 0;
    while (await mycuhProducts.hasNext()) {
      num++;
      let product = await mycuhProducts.next();
      mycuhProductsList[num] = product.productName;
      mycuhPricesEmbed
        .setTitle("Mycuh Products Store")
        .setThumbnail(
          "https://media.discordapp.net/attachments/1022643214278209558/1262638722969763950/IMG_20191116_231523_168.jpg?ex=66975372&is=669601f2&hm=f64f8fcade1df3c6077df84cd6bed4e5aa41385ca22e66bcb853866c2978b3e7&=&format=webp"
        )
        .addFields({
          name: `#${num} ${product.productName}`,
          value: `Price: **ℳ︁ ${product.productPrice}**\nCost Scales: **${
            product.productCostScales ? "Yes" : "No"
          }**\nDescription: **${product.productDescription}**`,
          inline: true,
        })
        .setFooter({
          text: "Last Updated",
        })
        .setTimestamp();
    }
    let buyButton = new ButtonBuilder()
      .setStyle(ButtonStyle.Success)
      .setCustomId("user_product_buy_button")
      .setLabel("Redeem Mycuh Bucks");
    let actionRow = new ActionRowBuilder().addComponents([buyButton]);
    await deleteAllMessagesInChannel(mycuhPricesChannel);

    mycuhPricesChannel.send({
      content: "",
      embeds: [mycuhPricesEmbed],
      components: [actionRow],
    });
  } else {
    await deleteAllMessagesInChannel(mycuhPricesChannel);
    mycuhPricesEmbed
      .setDescription(
        "Hmm seems like theres no mycuh products, tell mycuhhh to stop being selfish and add some."
      )
      .setFooter({
        text: "Last Updated",
      })
      .setTimestamp();

    mycuhPricesChannel.send({
      content: "",
      embeds: [mycuhPricesEmbed],
    });
  }
}

async function addMycuhProduct(interaction) {
  let mycuhProductExists = await dbConnection.collection("products").findOne({
    productName: interaction.fields.getTextInputValue("mycuhProductName"),
  });
  if (mycuhProductExists != undefined || mycuhProductExists != null)
    return interaction.reply({
      content: "This product already exists",
      ephemeral: true,
    });
  productName = ensureType(
    "string",
    interaction.fields.getTextInputValue("mycuhProductName")
  );
  productPrice = ensureType(
    "integer",
    interaction.fields.getTextInputValue("mycuhProductPrice")
  );
  productDescription = ensureType(
    "string",
    interaction.fields.getTextInputValue("mycuhProductDescription")
  );
  productCostScales = ensureType(
    "boolean",
    interaction.fields.getTextInputValue("mycuhProductCostScales")
  );
  if (!productName.success)
    return interaction.reply({
      content: "Product name invalid",
      ephemeral: true,
    });
  else if (!productPrice.success)
    return interaction.reply({
      content: "Product price invalid",
      ephemeral: true,
    });
  else if (!productDescription.success)
    return interaction.reply({
      content: "Product description invalid",
      ephemeral: true,
    });
  else if (!productCostScales.success)
    return interaction.reply({
      content: "Product cost scales input invalid",
      ephemeral: true,
    });

  await dbConnection.collection("products").insertOne({
    productName: productName.value,
    productPrice: productPrice.value,
    productDescription: productDescription.value,
    productCostScales: productCostScales.value,
    percentageOff: 0,
  });
  interaction.reply({
    content: `${productName.value} added to products list`,
    ephemeral: true,
  });

  embedMycuhPrices();
}

/**
 * Removes a mycuh product by name, from the database.
 * @param {*} productName
 * @returns success status and error (if one occured)
 */
async function removeMycuhProduct(productName) {
  try {
    let deletedProduct = await dbConnection
      .collection("products")
      .deleteOne({ productName: productName });
    if (!deletedProduct.deletedCount) {
      throw new Error("Product does not exist");
    }
  } catch (error) {
    return {
      success: false,
      error: error,
    };
  }
  embedMycuhPrices();
  return { success: true };
}

async function setMycuhProductsSale(isRemovingSale, percentageOff = 0) {
  if (isSettingSale) {
    // TODO: go into collection "mycuhProducts" in document "products" and change isOnSale to true and change percentage amount off to specified
  }
}

async function changeMycuhBucks(messageId, requestAmount, isRemoving) {
  if (isRemoving) {
    await new Promise((resolve, reject) => {
      // removes bucks
      dbConnection
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
    await new Promise((resolve, reject) => {
      // adds bucks
      dbConnection
        .collection("users")
        .findOne({ "pendingOutgoingRequest.messageId": messageId })
        .then((document) => {
          if (document) {
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
 * Gets myccount of provided user.
 * @param {*} discordUser
 * @returns myccount document from database
 */
async function getMyccount(discordUser) {
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

async function deleteAllMessagesInChannel(channel) {
  let fetchedMessagesData;
  do {
    fetchedMessagesData = await channel.messages.fetch({ limit: 100 });
    channel.bulkDelete(fetchedMessagesData, true);
  } while (fetchedMessagesData > 1);
}

// Handle transaction requests
async function newRequest(
  transactionMessage,
  sendingUser,
  receivingUser,
  requestDetails,
  mycuhBuckRequest = false
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
          },
        },
      },
      { upsert: true }
    );
    if (mycuhBuckRequest == false) {
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
        dbConnection.collection("users").updateOne(
          { discordId: sendingUser.user.id },
          {
            $set: {
              pendingOutgoingRequest: {
                messageId: "",
                requestReason: "",
                requestAmount: 0,
              },
            },
          },
          { upsert: true }
        );
        if (mycuhBuckRequest == false) {
          dbConnection.collection("users").updateOne(
            { discordId: receivingUser.user.id },
            {
              $set: {
                pendingIncomingRequest: {
                  messageId: "",
                  requestReason: "",
                  requestAmount: 0,
                },
              },
            },
            { upsert: true }
          );
        }
        resolve("Success");
      });
      if (mycuhBuckRequest == false) {
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
      } else {
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

async function handleModalSubmit(interaction) {
  switch (interaction.customId) {
    case "addMycuhProductModal":
      addMycuhProduct(interaction);
      break;
    case "userProductBuyNumberModal":
      if (await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction)) {
        return interaction.reply({
          content:
            "You already have a pending outgoing request, wait for it to expire",
        });
      }
      let productNumber = ensureType(
        "integer",
        interaction.fields.getTextInputValue("userProductNumber")
      );
      if (!productNumber.success) {
        return interaction.reply({
          content: "Input is not a number",
          ephemeral: true,
        });
      } else if (mycuhProductsList[productNumber.value] == undefined) {
        return interaction.reply({
          content:
            "A product with that number doesnt exist (make sure not to include the '#')",
          ephemeral: true,
        });
      }

      let productQuantity;
      let product = await dbConnection
        .collection("products")
        .findOne({ productName: mycuhProductsList[productNumber.value] });
      let confirmPurchaseEmbed = new EmbedBuilder();

      if (product.productCostScales) {
        productQuantity = ensureType(
          "integer",
          interaction.fields.getTextInputValue("userProductQuantity")
        );
        if (!productQuantity.success) {
          return interaction.reply({
            content: "Error: The product cost scales input valid quantity",
          });
        } else if (productQuantity.value <= 0) {
          return interaction.reply({
            content:
              "You cant just buy nothing sir... (Error: input quantity 0 or less)",
          });
        }
        confirmPurchaseEmbed.addFields([
          {
            name: "Quantity to redeem",
            inline: true,
            value: `Quanity: ${productQuantity.value}`,
          },
        ]);
      }

      confirmPurchaseEmbed
        .setTimestamp()
        .setTitle("Confirm Redeem")
        .setColor("Green")
        .addFields(
          {
            name: "Total Price",
            inline: true,
            value: `Cost: **ℳ︁ ${
              product.productCostScales
                ? product.productPrice * productQuantity.value
                : product.productPrice
            }**`,
          },
          {
            name: "Description",
            inline: false,
            value: product.productDescription,
          }
        );

      let confirmButton = new ButtonBuilder()
        .setCustomId("user_confirm_redeem_button")
        .setStyle(ButtonStyle.Success)
        .setLabel("Request Redemption");

      let actionRow = new ActionRowBuilder().addComponents([confirmButton]);
      interaction.reply({
        content: "",
        embeds: [confirmPurchaseEmbed],
        ephemeral: true,
        components: [actionRow],
      });

      break;
  }
}

// Handles when a user clicks the accept or deny button under a request
async function handleButtonResponse(interaction) {
  let response;
  let userConfirmRedeemRequest =
    interaction.customId == "user_confirm_redeem_button" ? true : false;
  let userBuyRequest =
    interaction.customId == "user_product_buy_button" ? true : false;
  let userAcceptedRequest =
    interaction.customId == "user_accept_button" ? true : false;
  let mycuhAcceptedRequest =
    interaction.customId == "mycuh_accept_button" ? true : false;
  let isMycuhRequest =
    interaction.customId == "mycuh_accept_button" ||
    interaction.customId == "mycuh_deny_button"
      ? true
      : false;
  let isMycuh = interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID);
  let transferEmbed = new EmbedBuilder();
  let receivingUserId;
  let requestDetails;
  let sendingUser;

  await new Promise((resolve, reject) => {
    dbConnection
      .collection("users")
      .findOne({"pendingOutgoingRequest.messageId": interaction.message.id })
      .then((document) => {
        if (document) {
          resolve(document);
        } else {
          resolve("Message id not attached to a needed user");
        }
      });
  }).then((result) => {
    if (result == "Message id not attached to a needed user") {
      response = result;
    } else {
      sendingUser = result;
      requestDetails = {
        requestAmount: sendingUser.pendingOutgoingRequest.requestAmount,
        requestReason: sendingUser.pendingOutgoingRequest.requestReason,
      };
    }
  });
  // TODO: check that expired transfer buttons dont crash server

  if (
    isMycuhRequest &&
    isMycuh &&
    response != "Message id not attached to a needed user"
  ) {
    if (mycuhAcceptedRequest == false) {
      await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
      transferEmbed
        .setTitle("Requesting Mycuh Bucks")
        .setColor("Red")
        .setDescription(
          `<@${
            sendingUser.discordId
          }> is requesting to add **${requestDetails.requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${
            requestDetails.requestReason
          }\nMycuh Response: **Denied** ❌`
        );
      interaction.message.edit({
        content: "",
        embeds: [transferEmbed],
        components: [],
      });
      return;
    } else {
      await changeMycuhBucks(
        interaction.message.id,
        requestDetails.requestAmount,
        false
      );
      await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
      transferEmbed
        .setTitle("Requesting Mycuh Bucks")
        .setColor("Green")
        .setDescription(
          `<@${
            sendingUser.discordId
          }> is requesting to add **${requestDetails.requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${
            requestDetails.requestReason
          }\nMycuh Response: **Accepted** ✅`
        );
      return interaction.message.edit({
        content: "",
        embeds: [transferEmbed],
        components: [],
      });
    }
  } else if (userConfirmRedeemRequest) {
    // TODO: create request

    return interaction.reply({
      content: "✅ Request created, expires in 5 minutes",
      ephemeral: true,
    });
  } else if (userBuyRequest && isMycuh) {
    return interaction.reply({
      content: "Noooo dood, only users can do this",
      ephemeral: true,
    });
  } else if (isMycuhRequest && !isMycuh) {
    interaction.reply({ content: 'You\'re not "him" 🦸', ephemeral: true });
    return;
  } else if (userBuyRequest) {
    let productNumberInput = new TextInputBuilder()
      .setCustomId("userProductNumber")
      .setStyle(TextInputStyle.Short)
      .setLabel("Enter product number (DONT INCLUDE THE '#')");
    let productQuantityInput = new TextInputBuilder()
      .setCustomId("userProductQuantity")
      .setStyle(TextInputStyle.Short)
      .setRequired(false)
      .setLabel("Input quantity");
    let productNumberActionRow = new ActionRowBuilder().addComponents([
      productNumberInput,
    ]);
    let productQuantityActionRow = new ActionRowBuilder().addComponents([
      productQuantityInput,
    ]);

    let userBuyRequestModal = new ModalBuilder()
      .setCustomId("userProductBuyNumberModal")
      .setTitle("Enter mycuh product number and quantity")
      .addComponents([productNumberActionRow, productQuantityActionRow]);

    interaction.showModal(userBuyRequestModal);
  } else {
    await new Promise((resolve, reject) => {
      dbConnection
        .collection("users")
        .findOne({ discordId: interaction.user.id })
        .then((document) => {
          if (document) {
            if (
              document.pendingIncomingRequest.messageId ==
              interaction.message.id
            ) {
              userAcceptedRequest == true
                ? resolve("Accepted")
                : resolve("Rejected");
            }
          }
          resolve("Not allowed");
        });
    }).then((result) => {
      response = result;
    });

    await new Promise((resolve, reject) => {
      dbConnection
        .collection("users")
        .findOne({ "pendingIncomingRequest.messageId": interaction.message.id })
        .then((document) => {
          if (document) {
            resolve(document);
          } else {
            resolve("Message id not attached to a needed user");
          }
        });
    }).then((result) => {
      if (result == "Message id not attached to a needed user") {
        response = "Message id not attached to a needed user";
      } else {
        receivingUserId = result.discordId;
      }
    });
  }

  if (response == "Not allowed") {
    interaction.reply({
      content: "Nice try, but you're not allowed to accept or deny this",
      ephemeral: true,
    });
  } else if (response == "Rejected") {
    transferEmbed
      .setTitle("Mycuh Bucks Transfer Request")
      .setColor("Red")
      .setDescription(
        `<@${
          sendingUser.discordId
        }> is requesting **${requestDetails.requestAmount.toString()}** mycuh buck(s) from <@${receivingUserId}> to their myccount\nReason: ${
          requestDetails.requestReason
        }\nUser Response: **Denied** ❌`
      );
    await clearPendingRequest(receivingUserId, REQUEST_STATE.INCOMING); // clears pending request from receiving user
    await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
    interaction.message.edit({ embeds: [transferEmbed], components: [] });
  } else if (response == "Accepted") {
    transferEmbed
      .setTitle("Mycuh Bucks Transfer Request")
      .setColor("Green")
      .setDescription(
        `<@${
          sendingUser.discordId
        }> is requesting **${requestDetails.requestAmount.toString()}** mycuh buck(s) from <@${receivingUserId}> to their myccount\nReason: ${
          requestDetails.requestReason
        }\nUser Response: **Accepted** ✅`
      );
    interaction.message.edit({ embeds: [transferEmbed], components: [] });
    // remove bucks from receiving user and add to sending user
    await changeMycuhBucks(
      interaction.message.id,
      requestDetails.requestAmount,
      false
    ); // adds mycuh bucks
    await changeMycuhBucks(
      interaction.message.id,
      requestDetails.requestAmount,
      true
    ); // removes mycuh bucks
    await clearPendingRequest(receivingUserId, REQUEST_STATE.INCOMING); // clears pending request from receiving user
    await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
  } else if (
    response == "Message id not attached to a needed user" &&
    !userBuyRequest
  ) {
    interaction.reply({
      content: "This request has already expired",
      ephemeral: true,
    });
    interaction.message.delete();
  }
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
        if (parseInt(text).toString() == "NaN")
          throw new Error("error changing text to number");
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

exports.checkMyccount = checkMyccount;
exports.addMycuhProduct = addMycuhProduct;
exports.removeMycuhProduct = removeMycuhProduct;
exports.getMycuhBucks = getMycuhBucks;
exports.ensureType = ensureType;
exports.setServerDetails = setServerDetails;
exports.handleModalSubmit = handleModalSubmit;
exports.embedMycuhPrices = embedMycuhPrices;
exports.checkPendingRequest = checkPendingRequest;
exports.clearAllPendingRequests = clearAllPendingRequests;
exports.newRequest = newRequest;
exports.handleButtonResponse = handleButtonResponse;
exports.REQUEST_STATE = REQUEST_STATE;
