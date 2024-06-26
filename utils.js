const { ObjectId } = require("mongodb");
const { getDb } = require("./db");
const { EmbedBuilder } = require("discord.js")

const REQUEST_STATE = {
    INCOMING: 0,
    OUTGOING: 1
}

// Checking to see if users discord id is connected to a myccount 
// or creates one if one doesnt exist
async function checkMyccount(discordUser){
    let dbConnection = getDb();
    let response;

    if(discordUser == undefined) {
        throw new Error("Discord id was not defined");
    }
    await new Promise((resolve, reject) => {
        dbConnection.collection("users").findOne({
            discordId: discordUser.user.id
        }).then((document) => {
            if(document == null){
                let objId = new ObjectId();
                let user = {
                    _id: objId,
                    username: discordUser.user.username,
                    discordId: discordUser.user.id,
                    bucks: 0,
                    pendingIncomingRequest: {messageId: "", requestReason: "", requestAmount: 0},
                    pendingOutgoingRequest: {messageId: "", requestReason: "", requestAmount: 0},
                }

                dbConnection.collection("users").insertOne(user);
                resolve("Created")
            }
            resolve("Success");
        });
    }).then((value) => {
        response = value;
    });
    return response;
}

async function embdedMycuhPrices(){
    let dbConnection = getDb();
    await new Promise((resolve, reject) => {
        dbConnection.collection("mycuhProducts").findOne({})
    });
    // TODO: delete everything in mycuh prices channel then post embed of mycuh prices from database
    // TODO: see in collection "mycuhProducts" in the products document whether there is a sale and for what amount off

    let mycuhPricesEmbed = new EmbedBuilder()
    .setCustomId()
}

async function addMycuhProduct(productName, productPrice, productDescription){
    let dbConnection = getDb();
    // TODO: adds or updates mycuh products to database with price and description and percentage off for possible future use
    // if mycuhProduct name already exists update
    embdedMycuhPrices();
}

async function removeMycuhProduct(){
    let dbConnection = getDb();
    // TODO: remove mycuh product from database
}

async function setMycuhProductsSale(isRemovingSale, percentageOff=0){
    if(isSettingSale){
        // TODO: go into collection "mycuhProducts" in document "products" and change isOnSale to true and change percentage amount off to specified
    }
}

async function changeMycuhBucks(messageId, requestAmount, isRemoving){
    let dbConnection = getDb();
    if(isRemoving){
        await new Promise((resolve, reject) => { // removes bucks
            dbConnection.collection("users").findOne({"pendingIncomingRequest.messageId": messageId}).then((document) => {
                if(document){
                    dbConnection.collection("users").updateOne(
                        {"pendingIncomingRequest.messageId": messageId},
                        {
                            $set: {
                                bucks: document.bucks - requestAmount,
                                "pendingIncomingRequest.messageId": ""
                            }
                        })
                }else{
                    console.log("user not found");
                }
            });
            resolve();
        });
    }else{
        await new Promise((resolve, reject) => { // adds bucks
            dbConnection.collection("users").findOne({"pendingOutgoingRequest.messageId": messageId}).then((document) => {
                if(document){
                    dbConnection.collection("users").updateOne(
                        {"pendingOutgoingRequest.messageId": messageId},
                        {
                            $set: {
                                bucks: document.bucks + requestAmount,
                                "pendingOutgoingRequest.messageId": ""
                            }
                        })
                }else{
                    console.log("user not found");
                }
            });
            resolve();
        });
    }
}

// Returns mycuh bucks
async function getMycuhBucks(discordUser){
    let document = await getMyccount(discordUser);
    if(document == null){
        return null;
    }
    return document.bucks;
}

// Returns a myccount
async function getMyccount(discordUser){
    let dbConnection = getDb();
    await checkMyccount(discordUser);
    let response;

    if(discordUser == undefined) {
        throw new Error("Discord id was not defined");
    }

    await new Promise((resolve, reject) => {
        dbConnection.collection("users").findOne({discordId: discordUser.user.id})
        .then((document) => {
            resolve(document);
        });
    }).then((value) => {
        response = value;
    });

    return response;
}

// Handle transaction requests
async function newRequest(transactionMessage, sendingUser, receivingUser, requestDetails, mycuhBuckRequest=false){
    let dbConnection = getDb();
    const EXPIRE_TIME = 300; // 5 minutes

    await new Promise((resolve, reject) => {
        dbConnection.collection("users").updateOne({discordId: sendingUser.user.id}, {$set: {pendingOutgoingRequest: {messageId: transactionMessage.id, requestReason: requestDetails.requestReason, requestAmount: requestDetails.requestAmount}}}, {upsert: true});
        if(mycuhBuckRequest == false){
            dbConnection.collection("users").updateOne({discordId: receivingUser.user.id}, {$set: {pendingIncomingRequest: {messageId: transactionMessage.id, requestReason: requestDetails.requestReason, requestAmount: requestDetails.requestAmount}}}, {upsert: true});
        }
        resolve("Success");
    });
    
    let messageTimer = setTimeout(async function() {
        let transferEmbed = new EmbedBuilder();
        let noConnectedUser = false;
            await new Promise((resolve, reject) => {
                dbConnection.collection("users").findOne({"pendingOutgoingRequest.messageId": transactionMessage.id})
                .then((document) => {
                    if(!document){
                        resolve("Not found");
                    }else{
                        resolve("Found");
                    }
                });
            }).then((result) => {
                if(result == "Not found"){
                    noConnectedUser = true;
                }
            });
            if(!noConnectedUser){
                await new Promise((resolve, reject) => {
                    dbConnection.collection("users").updateOne({discordId: sendingUser.user.id}, {$set: {pendingOutgoingRequest: {messageId: "", requestReason: "", requestAmount: 0}}}, {upsert: true});
                    if(mycuhBuckRequest == false){
                        dbConnection.collection("users").updateOne({discordId: receivingUser.user.id}, {$set: {pendingIncomingRequest: {messageId: "", requestReason: "", requestAmount: 0}}}, {upsert: true});
                    }
                    resolve("Success");
                });
                if(mycuhBuckRequest == false){
                    transferEmbed.setTitle("Mycuh Bucks Transfer Request")
                    .setDescription(`<@${sendingUser.user.id}> is requesting **${requestDetails.requestAmount.toString()}** mycuh buck(s) from <@${receivingUser.user.id}> to their myccount\nReason: ${requestDetails.requestReason}\nUser Response: **Expired** 🕑`);
                    transactionMessage.edit({embeds: [transferEmbed], components: []});
                }else{
                    transferEmbed.setTitle("Requesting Mycuh Bucks")
                    .setDescription(`<@${sendingUser.user.id}> is requesting to add **${requestDetails.requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${requestDetails.requestReason}\nMycuh Response: **Expired** 🕑`);
                    transactionMessage.edit({content: "", embeds: [transferEmbed], components: []});
                }
            }
    }, EXPIRE_TIME * 1000);

    return messageTimer;
}

async function checkPendingRequest(incomingOrOutgoing, discordUser){
    let dbConnection = getDb();
    let response;
    await checkMyccount(discordUser);

    await new Promise((resolve, reject) => {
        dbConnection.collection("users").findOne({discordId: discordUser.user.id})
        .then((document) => {
            if(incomingOrOutgoing == REQUEST_STATE.INCOMING){
                if(document.pendingIncomingRequest.messageId == ""){
                    resolve(false);
                }else{
                    resolve(true);
                }
            }else if(incomingOrOutgoing == REQUEST_STATE.OUTGOING){
                if(document.pendingOutgoingRequest.messageId == ""){
                    resolve(false);
                }else{
                    resolve(true);
                }
            }
        })
    }).then((value) => {
        response = value;
    });

    return response;
}

// Handles when a user clicks the accept or deny button under a request
async function handleButtonResponse(interaction) {
    let response;
    let dbConnection = getDb();
    let userAcceptedRequest = (interaction.customId == "user_accept_button") ? true : false;
    let mycuhAcceptedRequest = (interaction.customId == "mycuh_accept_button") ? true : false;
    let isMycuhRequest = (interaction.customId == "mycuh_accept_button" || interaction.customId == "mycuh_deny_button") ? true : false;
    let isMycuh = interaction.member.roles.cache.has(process.env.MYCUH_ROLE_ID);
    let transferEmbed = new EmbedBuilder();
    let receivingUserId;
    let requestDetails;
    let sendingUser;

    await new Promise((resolve, reject) => {
        dbConnection.collection("users").findOne({"pendingOutgoingRequest.messageId": interaction.message.id})
        .then((document) => {
            if(document){
                resolve(document);
            }else{
                resolve("Message id not attached to a needed user");
            }
        });
    }).then((result) => {
        if(result == "Message id not attached to a needed user"){
            response = result;
        }else{
            sendingUser = result;
            requestDetails = {requestAmount: sendingUser.pendingOutgoingRequest.requestAmount, requestReason: sendingUser.pendingIncomingRequest.requestReason};
        }
    });
    // TODO: check for expired transfer buttons dont crash server
    

    if(isMycuhRequest && isMycuh && response != "Message id not attached to a needed user"){
        if(mycuhAcceptedRequest == false){
            await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
            transferEmbed.setTitle("Requesting Mycuh Bucks")
            .setColor("Red")
            .setDescription(`<@${sendingUser.discordId}> is requesting to add **${requestDetails.requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${requestDetails.requestReason}\nMycuh Response: **Denied** ❌`);
            interaction.message.edit({content: "", embeds: [transferEmbed], components: []});
            return;
        }else{
            await changeMycuhBucks(interaction.message.id, requestDetails.requestAmount, false);
            await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
            transferEmbed.setTitle("Requesting Mycuh Bucks")
            .setColor("Green")
            .setDescription(`<@${sendingUser.discordId}> is requesting to add **${requestDetails.requestAmount.toString()}** mycuh buck(s) to their myccount\nReason: ${requestDetails.requestReason}\nMycuh Response: **Accepted** ✅`);
            interaction.message.edit({content: "", embeds: [transferEmbed], components: []});
            return;
        }
    }else if(isMycuhRequest && !isMycuh){
        interaction.reply({content: "You're not \"him\" 🦸", ephemeral: true});
        return;
    }else{
        await new Promise((resolve, reject) => {
            dbConnection.collection("users").findOne({discordId: interaction.user.id})
            .then((document) => {
                if(document){
                    if(document.pendingIncomingRequest.messageId == interaction.message.id){
                        (userAcceptedRequest == true) ? resolve("Accepted") : resolve("Rejected");
                    }
                }
                resolve("Not allowed");
            });
        }).then((result) => {
            response = result;
        });

        await new Promise((resolve, reject) => {
            dbConnection.collection("users").findOne({"pendingIncomingRequest.messageId": interaction.message.id})
            .then((document) => {
                if(document){
                    resolve(document);
                }else{
                    resolve("Message id not attached to a needed user");
                }
            });
        }).then((result) => {
            if(result == "Message id not attached to a needed user"){
                response = "Message id not attached to a needed user";
            }else{
                receivingUserId = result.discordId;
            }
        });
    }

    if(response == "Not allowed"){
        interaction.reply({content: "Nice try, but you're not allowed to accept or deny this", ephemeral: true });
    }else if(response == "Rejected"){
        transferEmbed.setTitle("Mycuh Bucks Transfer Request")
        .setColor("Red")
        .setDescription(`<@${sendingUser.discordId}> is requesting **${requestDetails.requestAmount.toString()}** mycuh buck(s) from <@${receivingUserId}> to their myccount\nReason: ${requestDetails.requestReason}\nUser Response: **Denied** ❌`);
        await clearPendingRequest(receivingUserId, REQUEST_STATE.INCOMING); // clears pending request from receiving user
        await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
        interaction.message.edit({embeds: [transferEmbed], components: []});
    }else if(response == "Accepted"){
        transferEmbed.setTitle("Mycuh Bucks Transfer Request")
        .setColor("Green")
        .setDescription(`<@${sendingUser.discordId}> is requesting **${requestDetails.requestAmount.toString()}** mycuh buck(s) from <@${receivingUserId}> to their myccount\nReason: ${requestDetails.requestReason}\nUser Response: **Accepted** ✅`);
        interaction.message.edit({embeds: [transferEmbed], components: []});
        // remove bucks from receiving user and add to sending user
        await changeMycuhBucks(interaction.message.id, requestDetails.requestAmount, false); // adds mycuh bucks
        await changeMycuhBucks(interaction.message.id, requestDetails.requestAmount, true); // removes mycuh bucks
        await clearPendingRequest(receivingUserId, REQUEST_STATE.INCOMING); // clears pending request from receiving user
        await clearPendingRequest(sendingUser.discordId, REQUEST_STATE.OUTGOING); // clears pending request from sending user
    }else if("Message id not attached to a needed user"){
        interaction.reply({content: "This request has already expired", ephemeral: true});
        interaction.message.delete();
    }
}

async function clearPendingRequest(discordUserId, incomingOrOutgoing){
    let dbConnection = getDb();
    if(incomingOrOutgoing == REQUEST_STATE.INCOMING){
        await new Promise((resolve, reject) => {
            dbConnection.collection("users").updateOne({discordId: discordUserId}, 
            {
                $set: {
                    "pendingIncomingRequest.messageId": ""
                }
            },
            {upsert: true});
            resolve();
        });
    }else{
        await new Promise((resolve, reject) => {
            dbConnection.collection("users").updateOne({discordId: discordUserId}, 
            {
                $set: {
                    "pendingOutgoingRequest.messageId": ""
                }
            },
            {upsert: true});
            resolve();
        });
    }
}

async function clearAllPendingRequests() {
    let dbConnection = getDb();
    // Expire past pending requests
    await new Promise((resolve, reject) => {
        dbConnection.collection("users").updateMany({}, {$set: {pendingIncomingRequest: {messageId: "", requestReason: "", requestAmount: 0}, pendingOutgoingRequest: {messageId: "", requestReason: "", requestAmount: 0}}})
        resolve("Success");
    });
}

exports.checkMyccount = checkMyccount;
exports.getMycuhBucks = getMycuhBucks;
exports.checkPendingRequest = checkPendingRequest;
exports.clearAllPendingRequests = clearAllPendingRequests;
exports.newRequest = newRequest;
exports.handleButtonResponse = handleButtonResponse;
exports.REQUEST_STATE = REQUEST_STATE;