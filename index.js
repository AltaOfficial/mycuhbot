const { connectToDb, getDb } = require("./db");
const { getClient, loginClientToDiscord } = require("./utilities/serverClient");
const {
  checkMyccount,
  getMycuhBucks,
} = require("./utilities/myccount/myccountUtils");
const {
  handleButtonResponse,
} = require("./utilities/handlers/handleButtonResponse");
const { handleModalSubmit } = require("./utilities/handlers/handleModalSubmit");
const {
  removeMycuhProduct,
} = require("./utilities/products/mycuhProductsUtil");
const {
  checkPendingRequest,
  newRequest,
  REQUEST_STATE,
  clearAllPendingRequests,
  setServerDetails,
} = require("./utilities/utils");
const { embedMycuhPrices } = require("./utilities/embedMycuhPrices");
const {
  ActivityType,
  EmbedBuilder,
  ButtonBuilder,
  ButtonStyle,
  ActionRowBuilder,
  ModalBuilder,
  TextInputBuilder,
  TextInputStyle,
} = require("discord.js");
const cron = require("node-cron");

cron.schedule("0 0 * * *", () => {
  embedMycuhPrices();
});

let client = getClient();

client.on("ready", () => {
  console.log(`✅ ${client.user.tag} is now online.`);

  // Connect to database
  connectToDb(async (err) => {
    if (!err) {
      console.log("Successfully connected to database");
    } else {
      console.log("Failed to connect to database, error: " + err);
    }
    dbConnection = getDb();
    await setServerDetails();
    clearAllPendingRequests();
    embedMycuhPrices();
  });

  client.user.setActivity({
    name: " For Commands",
    type: ActivityType.Watching,
  });
});

loginClientToDiscord();

// handle commands
client.on("interactionCreate", async (interaction) => {
  if (interaction.isChatInputCommand()) {
    let transferChannel = client.channels.cache.get(
      process.env.TRANSFERS_CHANNEL_ID
    );
    let mycuhRole = interaction.guild.roles.cache.find(
      (roles) => roles.name == "Mycuhhhhh"
    );

    try {
      switch (interaction.commandName) {
        // Displays amount of mycuh bucks user has
        case "balance":
          if (interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID)) {
            interaction.reply({
              content: "You're mycuh you have unlimited mycuh bucks",
              ephemeral: true,
            });
          } else {
            let balance = await getMycuhBucks(interaction);
            if (balance == null) {
              interaction.reply({
                content: "Connection with database is slow, try again",
              });
            } else {
              interaction.reply({
                content: `You have **${balance}** mycuh bucks 💵`,
              });
            }
          }
          break;

        case "addmycuhproduct":
          // TODO: Make it so only mycuh role can use this command
          if (interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID)) {
            let addMycuhProductModal = new ModalBuilder()
              .setCustomId("addMycuhProductModal")
              .setTitle("Create a new Mycuh Product");

            let firstActionRow = new ActionRowBuilder().addComponents(
              new TextInputBuilder()
                .setCustomId("mycuhProductName")
                .setLabel("Product Name")
                .setPlaceholder("Enter the product name")
                .setStyle(TextInputStyle.Short)
                .setRequired(true)
            );
            let secondActionRow = new ActionRowBuilder().addComponents(
              new TextInputBuilder()
                .setCustomId("mycuhProductPrice")
                .setLabel("Price")
                .setPlaceholder("Enter the price")
                .setStyle(TextInputStyle.Short)
                .setRequired(true)
            );
            let thirdActionRow = new ActionRowBuilder().addComponents(
              new TextInputBuilder()
                .setCustomId("mycuhProductCostScales")
                .setLabel("Does the cost scale?")
                .setPlaceholder("Type Yes or no")
                .setStyle(TextInputStyle.Short)
                .setRequired(true)
            );
            let fourthActionRow = new ActionRowBuilder().addComponents(
              new TextInputBuilder()
                .setCustomId("mycuhProductDescription")
                .setLabel("Description")
                .setPlaceholder("Enter the description")
                .setStyle(TextInputStyle.Paragraph)
                .setRequired(true)
            );

            addMycuhProductModal.addComponents(
              firstActionRow,
              secondActionRow,
              thirdActionRow,
              fourthActionRow
            );

            interaction.showModal(addMycuhProductModal);
          } else {
            interaction.reply({
              content: "Only mycuhhhh can use this command",
              ephemeral: true,
            });
          }
          break;

        case "removemycuhproduct":
          if (interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID)) {
            let productName = interaction.options.get(
              "remove-product-name"
            ).value;
            let removedProduct = await removeMycuhProduct(productName);
            if (removedProduct.success) {
              interaction.reply({
                content: "Product removed successfully",
                ephemeral: true,
              });
            } else {
              interaction.reply({
                content: "Failed to remove product: " + removedProduct.error,
                ephemeral: true,
              });
            }
          } else {
            interaction.reply({
              content: "Only mycuhhhh can use this command",
              ephemeral: true,
            });
          }
          break;

        // Request mycuh bucks from mycuhhhh himself
        case "request":
          let requestAmount = interaction.options.get("request-amount").value;
          let requestReason = interaction.options.get("request-reason").value;
          let acceptButton = new ButtonBuilder()
            .setStyle(ButtonStyle.Success)
            .setCustomId("mycuh_accept_button")
            .setLabel("Accept")
            .setDisabled(false);
          let denyButton = new ButtonBuilder()
            .setStyle(ButtonStyle.Danger)
            .setCustomId("mycuh_deny_button")
            .setLabel("Deny")
            .setDisabled(false);
          let actionRow = new ActionRowBuilder().addComponents([
            acceptButton,
            denyButton,
          ]);
          let embed = new EmbedBuilder()
            .setTitle(`Requesting Mycuh Bucks`)
            .setDescription(
              `<@${
                interaction.user.id
              }> is requesting to add **${requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${requestReason}\nMycuh Response: **Waiting...** 🕑`
            )
            .setColor("Green");

          if (interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID)) {
            interaction.reply({
              content: "Ur funny mycuhhhh",
              ephemeral: true,
            });
            break;
          } else if (
            (await checkPendingRequest(REQUEST_STATE.INCOMING, interaction)) ==
            true
          ) {
            interaction.reply({
              content: `❌ <@${member.user.id}> already has a pending request`,
              ephemeral: true,
            });
            break;
          } else if (
            (await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction)) ==
            true
          ) {
            interaction.reply({
              content:
                "You already have a pending request, wait for it to be accepted or expire 🕑",
              ephemeral: true,
            });
            break;
          } else if (requestAmount <= 0) {
            interaction.reply({
              content: "You cant ask for that amount retard 😡",
              ephemeral: true,
            });
            break;
          }

          let requestTransactionMessage = await transferChannel.send({
            content: `${mycuhRole ? `${mycuhRole}` : ""}`,
            embeds: [embed],
            components: [actionRow],
          });
          newRequest(
            requestTransactionMessage,
            interaction,
            null,
            { requestAmount: requestAmount, requestReason: requestReason },
            "request"
          );
          interaction.reply({
            content: "✅ Request created, expires in 5 mins",
          });
          break;

        // Transfer mycuh bucks from one user to another
        case "transfer":
          let member = interaction.options.get("transfer-member");
          let transferAmount = interaction.options.get("transfer-amount").value;
          let transferReason = interaction.options.get("transfer-reason").value;

          if (
            member.member.roles.cache.has(process.env.MYCUH_ROLE_ID) == true
          ) {
            interaction.reply({
              content:
                "❔ use the request command to ask for mycuh bucks from mycuh",
              ephemeral: true,
            });
            break;
          } else if (
            (await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction)) ==
            true
          ) {
            interaction.reply({
              content:
                "You already have a pending request, wait for it to be accepted or expire 🕑",
              ephemeral: true,
            });
            break;
          } else if (transferAmount <= 0) {
            interaction.reply({
              content: "You cant ask for that amount retard 😡",
              ephemeral: true,
            });
            break;
          } else if (interaction.user.id == member.user.id) {
            interaction.reply({
              content: "You cant ask yourself for mycuh bucks meat head 😡",
              ephemeral: true,
            });
            break;
          }

          if ((await checkMyccount(member)) == "Created") {
            interaction.reply({
              content: "🤒 This monkey is dead broke",
              ephemeral: true,
            });
            break;
          } else if (
            (await checkPendingRequest(REQUEST_STATE.INCOMING, member)) == true
          ) {
            interaction.reply({
              content: `❌ <@${member.user.id}> already has a pending request`,
              ephemeral: true,
            });
            break;
          } else if ((await getMycuhBucks(member)) < transferAmount) {
            interaction.reply({
              content: `❌ You're asking for more mycuh bucks than <@${member.user.id}> has`,
              ephemeral: true,
            });
            break;
          }

          let transferAcceptButton = new ButtonBuilder()
            .setStyle(ButtonStyle.Success)
            .setCustomId("user_accept_button")
            .setLabel("Accept")
            .setDisabled(false);
          let transferDenyButton = new ButtonBuilder()
            .setStyle(ButtonStyle.Danger)
            .setCustomId("user_deny_button")
            .setLabel("Deny")
            .setDisabled(false);
          let transferActionRow = new ActionRowBuilder().addComponents([
            transferAcceptButton,
            transferDenyButton,
          ]);
          let transferEmbed = new EmbedBuilder()
            .setTitle("Mycuh Bucks Transfer Request")
            .setDescription(
              `<@${
                interaction.user.id
              }> is requesting **${transferAmount.toString()}** mycuh buck(s) from <@${
                member.user.id
              }> to their myccount\nReason: ${transferReason}\nUser Response: **Waiting...** 🕑`
            )
            .setColor("Green");

          let transferTransactionMessage = await transferChannel.send({
            embeds: [transferEmbed],
            components: [transferActionRow],
          });
          newRequest(
            transferTransactionMessage,
            interaction,
            member,
            {
              requestAmount: transferAmount,
              requestReason: transferReason,
            },
            "transfer"
          );
          interaction.reply({
            content: "✅ Transfer request created, expires in 5 mins",
          });
          break;
      }
    } catch (err) {
      console.log(`Error Occured: ${err} index.js`);
    }
  } else if (interaction.isButton()) {
    await handleButtonResponse(interaction);
  } else if (interaction.isModalSubmit()) {
    await handleModalSubmit(interaction);
  }
});
