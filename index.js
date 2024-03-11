const { connectToDb, getDb } = require("./db");
const { checkMyccount, getMycuhBucks, checkPendingRequest, newRequest, REQUEST_STATE, handleButtonResponse, clearAllPendingRequests } = require("./utils");
const { Client, GatewayIntentBits, Partials, InteractionType, ActivityType, Message, EmbedBuilder, ButtonBuilder, ButtonStyle, ActionRowBuilder} = require("discord.js");

const client = new Client({
    intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent,
        GatewayIntentBits.GuildMembers
    ],
    partials: [Partials.Message, Partials.Channel]
});

// Connect to database
let dbConnection;
connectToDb((err) => {
    if(!err) {
        console.log("Successfully connected to database");
    }else {
        Console.log("Failed to connect to database, error: " + err);
    }
    dbConnection = getDb();
    clearAllPendingRequests();
});

client.on("ready", () => {
    console.log(`✅ ${client.user.tag} is now online.`);

    client.user.setActivity({
        name: " For Commands",
        type: ActivityType.Watching
    });
});

client.login(process.env.DISCORD_TOKEN);

// handle commands
client.on("interactionCreate", async (interaction) => {
    if(interaction.isChatInputCommand()){
        let transferChannel = client.channels.cache.get(process.env.TRANSFERS_CHANNEL_ID);
        let mycuhRole = interaction.guild.roles.cache.find(roles => roles.name == "Mycuhhhhh");
    
        try {
            switch(interaction.commandName){
                // Displays amount of mycuh bucks user has
                case "balance":
                    let balance = await getMycuhBucks(interaction);
                    interaction.reply({content: `You have **${balance}** mycuh bucks 💵`, ephemeral: false});
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
                    let actionRow = new ActionRowBuilder().addComponents([acceptButton, denyButton]);
                    let embed = new EmbedBuilder()
                        .setTitle(`Requesting Mycuh Bucks`)
                        .setDescription(`<@${interaction.user.id}> is requesting to add **${requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${requestReason}\nMycuh Response: **Waiting...** 🕑`)
                        .setColor("Green");

                    if(interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID)){
                        interaction.reply({content: "Ur funny mycuhhhh", ephemeral: true});
                        break;
                    }else if(await checkPendingRequest(REQUEST_STATE.INCOMING, interaction) == true){
                        interaction.reply({content: `❌ <@${member.user.id}> already has a pending request`, ephemeral: true});
                        break;
                    }else if(await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction) == true){
                        interaction.reply({content: "You already have a pending request, wait for it to be accepted or expire 🕑", ephemeral: true});
                        break;
                    }else if(requestAmount <= 0){
                        interaction.reply({content: "You cant ask for that amount retard 😡", ephemeral: true});
                        break;
                    }

                    let requestTransactionMessage = await transferChannel.send({content: `${mycuhRole ? `${mycuhRole}` : ""}`, embeds: [embed], components: [actionRow]});
                    newRequest(requestTransactionMessage, interaction, null, {requestAmount: requestAmount, requestReason: requestReason}, true)
                    interaction.reply({content: "✅ Request created, expires in 5 mins"});
                break;
        
                // Redeem mycuh bucks
                case "redeem":
                    interaction.reply({content: "coming soon", ephemeral: false});
                break;
        
                // Transfer mycuh bucks from one user to another
                case "transfer":
                    let member = interaction.options.get("transfer-member");
                    let transferAmount = interaction.options.get("transfer-amount").value;
                    let transferReason = interaction.options.get("transfer-reason").value;

                    if(member.member.roles.cache.has(process.env.MYCUH_ROLE_ID) == true){
                        interaction.reply({content: "❔ use the request command to ask for mycuh bucks from mycuh", ephemeral: true});
                        break;
                    }else if(await checkPendingRequest(REQUEST_STATE.OUTGOING, interaction) == true){
                        interaction.reply({content: "You already have a pending request, wait for it to be accepted or expire 🕑", ephemeral: true});
                        break;
                    }else if(transferAmount <= 0){
                        interaction.reply({content: "You cant ask for that amount retard 😡", ephemeral: true});
                        break;
                    }else if(interaction.user.id == member.user.id){
                        interaction.reply({content: "You cant ask yourself for mycuh bucks meat head 😡", ephemeral: true});
                        break;
                    }

                    if(await checkMyccount(member) == "Created"){
                        interaction.reply({content: "🤒 This monkey is dead broke", ephemeral: true});
                        break;
                    }else if(await checkPendingRequest(REQUEST_STATE.INCOMING, member) == true){
                        interaction.reply({content: `❌ <@${member.user.id}> already has a pending request`, ephemeral: true});
                        break;
                    } else if(await getMycuhBucks(member) < transferAmount){
                        interaction.reply({content: `❌ You're asking for more mycuh bucks than <@${member.user.id}> has`, ephemeral: true});
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
                    let transferActionRow = new ActionRowBuilder().addComponents([transferAcceptButton, transferDenyButton]);
                    let transferEmbed = new EmbedBuilder()
                        .setTitle("Mycuh Bucks Transfer Request")
                        .setDescription(`<@${interaction.user.id}> is requesting **${transferAmount.toString()}** mycuh buck(s) from <@${member.user.id}> to their myccount\nReason: ${transferReason}\nUser Response: **Waiting...** 🕑`)
                        .setColor("Green");
                    
                    let transferTransactionMessage = await transferChannel.send({embeds: [transferEmbed], components: [transferActionRow]});
                    newRequest(transferTransactionMessage, interaction, member, {requestAmount: transferAmount, requestReason: transferReason});
                    interaction.reply({content: "✅ Transfer request created, expires in 5 mins"});
                break;
            }
        }catch(err) {
            console.log(`Error Occured: ${err}`);
        }
    }else if(interaction.isButton()){
        await handleButtonResponse(interaction);
    }
    
});