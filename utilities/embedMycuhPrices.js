const {
  EmbedBuilder,
  ButtonBuilder,
  ButtonStyle,
  ActionRowBuilder,
} = require("discord.js");
const { deleteAllMessagesInChannel } = require("./utils");
const {
  setMycuhProductsList,
  getMycuhProductsList,
} = require("./products/mycuhProductsList");

async function embedMycuhPrices() {
  await setMycuhProductsList();
  let mycuhProductsList = await getMycuhProductsList();
  const client = require("./serverClient").getClient();
  let mycuhPricesChannel = client.channels.cache.get(
    process.env.MYCUH_PRICES_CHANNEL_ID
  );

  // TODO: see in collection "mycuhProducts" in the products document whether there is a sale and for what amount off
  let mycuhPricesEmbed = new EmbedBuilder();
  if (mycuhProductsList.length >= 1) {
    mycuhProductsList.forEach((product, index) => {
      mycuhProductsList[index] = product.productName;
      mycuhPricesEmbed
        .setTitle("Mycuh Products Store\n(Products Approved/Created by Mycuh)")
        .setThumbnail(
          "https://media.discordapp.net/attachments/1022643214278209558/1262638722969763950/IMG_20191116_231523_168.jpg?ex=66975372&is=669601f2&hm=f64f8fcade1df3c6077df84cd6bed4e5aa41385ca22e66bcb853866c2978b3e7&=&format=webp"
        )
        .addFields({
          name: `#${index} ${product.productName}`,
          value: `Price: **ℳ︁ ${product.productPrice}**\nCost Scales: **${
            product.productCostScales ? "Yes" : "No"
          }**\nDescription: **${product.productDescription}**`,
          inline: true,
        })
        .setFooter({
          text: "Last Updated",
        })
        .setTimestamp();
    });

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

exports.embedMycuhPrices = embedMycuhPrices;
