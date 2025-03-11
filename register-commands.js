require("dotenv").config();
const { REST, Routes, ApplicationCommandOptionType } = require("discord.js");

const commands = [
  {
    name: "balance",
    description: "Shows mycuh bank balance",
  },
  {
    name: "addmycuhproduct",
    description: "Create a product in the mycuh prices channel",
  },
  {
    name: "removemycuhproduct",
    description: "Create a new product in the mycuh prices channel",
    options: [
      {
        name: "remove-product-name",
        description: "Name of the product you want to remove",
        type: ApplicationCommandOptionType.String,
      },
    ],
  },
  {
    name: "request",
    description: "Ask mycuh to receive mycuh buck(s)",
    options: [
      {
        name: "request-amount",
        description: "Amount of mycuh bucks to ask for",
        type: ApplicationCommandOptionType.Number,
        required: true,
      },
      {
        name: "request-reason",
        description: "The reason for asking for mycuh bucks",
        type: ApplicationCommandOptionType.String,
        required: true,
      },
    ],
  },
  {
    name: "transfer",
    description: "Ask another member to transfer mycuh buck(s)",
    options: [
      {
        name: "transfer-member",
        description: "Member to ask for mycuh bucks from",
        type: ApplicationCommandOptionType.User,
        required: true,
      },
      {
        name: "transfer-amount",
        description: "Amount of mycuh bucks to ask for",
        type: ApplicationCommandOptionType.Number,
        required: true,
      },
      {
        name: "transfer-reason",
        description: "Reason for asking for mycuh bucks",
        type: ApplicationCommandOptionType.String,
        required: true,
      },
    ],
  },


  {
    name: "start-market",
    description: "Starts the stock market (admin)",
  },
  {
    name: "file-company",
    description: "Join the stock market and make investments",
  },
  {
    name: "file-for-bankruptcy",
    description: "Leave the stock market and lose all assests",
  },

  {
    name: "buy-stock",
    description: "Buy y stock from x company",
    options: [
      {
        name: "company",
        description: "Company from stock market",
        type: ApplicationCommandOptionType.User,
        required: true,
      },
      {
        name: "amount",
        description: "Amount of stock from company",
        type: ApplicationCommandOptionType.Number,
        required: true,
      },
  },
  {
    name: "sell-stock",
    description: "Sell y stock from x company",
    options: [
      {
        name: "company",
        description: "Company from stock market",
        type: ApplicationCommandOptionType.User,
        required: true,
      },
      {
        name: "amount",
        description: "Amount of stock from company",
        type: ApplicationCommandOptionType.Number,
        required: true,
      },
  },

  /*
  {
    name: "random-event",
    description: "Cause a random event to happen that effects stock prices in a number of companys",
  },
  */

];

const rest = new REST({ version: "10" }).setToken(process.env.DISCORD_TOKEN);

(async () => {
  console.log("Registering slash commands...");

  try {
    await rest.put(
      Routes.applicationGuildCommands(
        process.env.APPLICATION_ID,
        process.env.GUILD_ID
      ),
      { body: commands }
    );

    console.log("Slash commands were registered successfully!");
  } catch (err) {
    console.log("Error occured: " + err);
  }
})();
