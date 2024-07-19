const {
  EmbedBuilder,
  ButtonBuilder,
  ButtonStyle,
  TextInputBuilder,
  TextInputStyle,
  ActionRowBuilder,
  ModalBuilder,
} = require("discord.js");
const { getClient } = require("../serverClient");
const { changeMycuhBucks } = require("../myccount/myccountUtils");
const {
  clearPendingRequest,
  newRequest,
  getRequestData,
  REQUEST_STATE,
  checkPendingRequest,
} = require("../utils");
const { getDb } = require("../../db");

const client = getClient();

// Handles when a user clicks the accept or deny button under a request
async function handleButtonResponse(interaction) {
  let dbConnection = getDb();
  let response;
  let userConfirmRedeemRequest =
    interaction.customId == "user_confirm_redeem_button" ? true : false;
  let userBuyRequest =
    interaction.customId == "user_product_buy_button" ? true : false;
  let userAcceptedRequest =
    interaction.customId == "user_accept_button" ? true : false;
  let mycuhAcceptedRequest =
    interaction.customId == "mycuh_accept_button" ||
    interaction.customId == "mycuh_redeem_accept_button"
      ? true
      : false;
  let isRedeemRequest =
    interaction.customId == "mycuh_redeem_accept_button" ||
    interaction.customId == "mycuh_redeem_deny_button";
  let isMycuhRequest =
    interaction.customId == "mycuh_accept_button" ||
    interaction.customId == "mycuh_deny_button" ||
    interaction.customId == "mycuh_redeem_accept_button" ||
    interaction.customId == "mycuh_redeem_deny_button"
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
      .findOne({ "pendingOutgoingRequest.messageId": interaction.message.id })
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
        requestProductName:
          sendingUser.pendingOutgoingRequest.requestProductName,
        requestQuantity: sendingUser.pendingOutgoingRequest.requestQuantity,
        requestTotalCost: sendingUser.pendingOutgoingRequest.requestTotalCost,
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

      if (isRedeemRequest) {
        transferEmbed
          .setTitle("Redeeming Mycuh Product")
          .setColor("Red")
          .setDescription(
            `<@${sendingUser.discordId}> is requesting to redeem **${requestDetails.requestProductName}** 
        Quantity: **${requestDetails.requestQuantity}**\nMycuh Response: **Denied** ❌`
          );
      } else {
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
      }
      interaction.message.edit({
        content: "",
        embeds: [transferEmbed],
        components: [],
      });
      return;
    } else if (isRedeemRequest) {
      console.log(requestDetails);
      await changeMycuhBucks(
        interaction.message.id,
        requestDetails.requestTotalCost,
        true
      );
      await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
      transferEmbed
        .setTitle("Redeeming Mycuh Product")
        .setColor("Green")
        .setDescription(
          `<@${sendingUser.discordId}> is requesting to redeem **${requestDetails.requestProductName}** 
        Quantity: **${requestDetails.requestQuantity}**\nMycuh Response: **Accepted** ✅`
        );
      return interaction.message.edit({
        content: "",
        embeds: [transferEmbed],
        components: [],
      });
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
    let requestData = await getRequestData();
    let transfersChannel = client.channels.cache.get(
      process.env.TRANSFERS_CHANNEL_ID
    );

    let redeemRequestActionRow = new ActionRowBuilder().addComponents([
      new ButtonBuilder()
        .setCustomId("mycuh_redeem_accept_button")
        .setStyle(ButtonStyle.Success)
        .setLabel("Accept"),
      new ButtonBuilder()
        .setCustomId("mycuh_redeem_deny_button")
        .setStyle(ButtonStyle.Danger)
        .setLabel("Deny"),
    ]);

    let redeemRequestModal = new EmbedBuilder()
      .setTitle("Redeeming Mycuh Product")
      .setColor("Green")
      .setDescription(
        `<@${interaction.user.id}> is requesting to redeem **${requestData.productName}** 
        Quantity: **${requestData.productQuantity}**\nMycuh Response: **Waiting...** 🕑`
      );

    let mycuhRole = interaction.guild.roles.cache.find(
      (roles) => roles.name == "Mycuhhhhh"
    );

    if (await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction)) {
      return interaction.reply({
        content:
          "You already have a pending request, wait for it to be accepted or expire 🕑",
        ephemeral: true,
      });
    }

    let redeemRequestMessage = await transfersChannel.send({
      content: `${mycuhRole ? `${mycuhRole}` : ""}`,
      embeds: [redeemRequestModal],
      components: [redeemRequestActionRow],
    });

    newRequest(
      redeemRequestMessage,
      interaction,
      "",
      {
        requestReason: "",
        requestAmount: 0,
        requestQuantity: requestData.productQuantity,
        requestProductName: requestData.productName,
        requestTotalCost: requestData.productTotalCost,
      },
      "redemption"
    );

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

exports.handleButtonResponse = handleButtonResponse;
