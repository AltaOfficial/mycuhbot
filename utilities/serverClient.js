const { Client, GatewayIntentBits, Partials } = require("discord.js");

let client = new Client({
  intents: [
    GatewayIntentBits.Guilds,
    GatewayIntentBits.GuildMessages,
    GatewayIntentBits.MessageContent,
    GatewayIntentBits.GuildMembers,
  ],
  partials: [Partials.Message, Partials.Channel],
});

module.exports = {
  loginClientToDiscord: async () => {
    await client.login(process.env.DISCORD_TOKEN);
  },
  getClient: () => client,
};
