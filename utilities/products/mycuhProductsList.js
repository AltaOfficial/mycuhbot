const { getDb } = require("../../db");
let mycuhProductsList = [];

async function setMycuhProductsList() {
  const dbConnection = getDb();
  mycuhProductsList = [];
  let mycuhProducts = await dbConnection.collection("products").find();

  if (await mycuhProducts.hasNext()) {
    let num = 0;
    while (await mycuhProducts.hasNext()) {
      num++;
      let product = await mycuhProducts.next();
      mycuhProductsList[num] = product;
    }
  }
}

async function getMycuhProductsList() {
  return mycuhProductsList;
}

exports.setMycuhProductsList = setMycuhProductsList;
exports.getMycuhProductsList = getMycuhProductsList;
