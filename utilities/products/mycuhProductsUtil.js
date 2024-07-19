const { embedMycuhPrices } = require("../embedMycuhPrices");
const { ensureType } = require("../utils");

let mycuhProductsList = [];

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

exports.mycuhProductsList = mycuhProductsList;
exports.addMycuhProduct = addMycuhProduct;
exports.removeMycuhProduct = removeMycuhProduct;
