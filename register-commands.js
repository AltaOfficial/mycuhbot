require('dotenv').config();
const { REST, Routes, ApplicationCommandOptionType } = require("discord.js");

const commands = [
    {
        name: "balance",
        description: "Shows mycuh bank balance",
    },
    {
        name: "redeem",
        description: "Ask mycuh to use mycuh buck(s)",
        options: [{
            name: "ability",
            description: "Ability to redeem",
            type: ApplicationCommandOptionType.String,
            required: true
        }]
    },
    {
        name: "addmycuhproduct",
        description: "Create/Update a product in the mycuh prices channel"
    },
    {
        name: "removemycuhproduct",
        description: "Create a new product in the mycuh prices channel"
    },
    {
        name: "request",
        description: "Ask mycuh to receive mycuh buck(s)",
        options: [{
            name: "request-amount",
            description: "Amount of mycuh bucks to ask for",
            type: ApplicationCommandOptionType.Number,
            required: true
        },
        {
            name: "request-reason",
            description: "The reason for asking for mycuh bucks",
            type: ApplicationCommandOptionType.String,
            required: true
        }]
    },
    {
        name: "transfer",
        description: "Ask another member to transfer mycuh buck(s)",
        options: [{
            name: "transfer-member",
            description: "Member to ask for mycuh bucks from",
            type: ApplicationCommandOptionType.User,
            required: true
        },
        {
            name: "transfer-amount",
            description: "Amount of mycuh bucks to ask for",
            type: ApplicationCommandOptionType.Number,
            required: true
        },
        {
            name: "transfer-reason",
            description: "Reason for asking for mycuh bucks",
            type: ApplicationCommandOptionType.String,
            required: true
        }]
    }
];

const rest = new REST({ version: '10' }).setToken(process.env.DISCORD_TOKEN);

(async () => {
    console.log("Registering slash commands...");

    try{
        await rest.put(
            Routes.applicationGuildCommands(process.env.APPLICATION_ID, process.env.GUILD_ID),
            {body: commands}
        );

        console.log("Slash commands were registered successfully!")
    }
    catch(err) {
        console.log("Error occured: " + err);
    }
})();