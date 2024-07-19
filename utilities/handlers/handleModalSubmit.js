const { getDb } = require("../../db");
const { getMycuhBucks } = require("../myccount/myccountUtils");
const { getMycuhProductsList } = require("../products/mycuhProductsList");
const { addMycuhProduct } = require("../products/mycuhProductsUtil");
const {
  checkPendingRequest,
  ensureType,
  REQUEST_STATE,
  setRequestData,
} = require("../utils");
const {
  EmbedBuilder,
  ActionRowBuilder,
  ButtonBuilder,
  ButtonStyle,
} = require("discord.js");

async function handleModalSubmit(interaction) {
  let dbConnection = getDb();
  let mycuhProductsList = await getMycuhProductsList();

  switch (interaction.customId) {
    case "addMycuhProductModal":
      addMycuhProduct(interaction);
      break;
    case "userProductBuyNumberModal":
      if (await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction)) {
        return interaction.reply({
          content:
            "You already have a pending outgoing request, wait for it to expire",
          ephemeral: true,
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
      let product = await dbConnection.collection("products").findOne({
        productName: mycuhProductsList[productNumber.value],
      });
      let confirmPurchaseEmbed = new EmbedBuilder();
      let totalCost;

      if (product.productCostScales) {
        productQuantity = ensureType(
          "integer",
          interaction.fields.getTextInputValue("userProductQuantity")
        );
        totalCost = product.productPrice * productQuantity.value;
        if (!productQuantity.success) {
          return interaction.reply({
            content: "Error: The product cost scales input valid quantity",
            ephemeral: true,
          });
        } else if (productQuantity.value <= 0) {
          return interaction.reply({
            content:
              "You cant just buy nothing sir... (Error: input quantity 0 or less)",
            ephemeral: true,
          });
        }
      } else {
        totalCost = product.productPrice;
      }

      if ((await getMycuhBucks(interaction)) < totalCost) {
        return interaction.reply({
          content:
            "You dont have enough mycuh bucks for this product and/or quantity",
          ephemeral: true,
        });
      }

      // go by user discord id and set their request data
      setRequestData({
        productName: mycuhProductsList[productNumber.value],
        productTotalCost: totalCost,
        productQuantity: productQuantity.value,
      });

      confirmPurchaseEmbed
        .setTimestamp()
        .setTitle("Confirm Redeem")
        .setColor("Green")
        .addFields(
          {
            name: "Total Price",
            inline: true,
            value: `Cost: **ℳ︁ ${totalCost}**`,
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

exports.handleModalSubmit = handleModalSubmit;
