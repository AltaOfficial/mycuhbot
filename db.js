const { MongoClient } = require("mongodb");
require("dotenv").config();


let dbConnection;

module.exports = {
    connectToDb: async (callback) => {
        await new Promise((resolve, reject) => {
            MongoClient.connect(process.env.DATABASE_URL)
            .then((client) => {
                dbConnection = client.db();
                resolve(callback());
            })
            .catch((err) => {
                reject(callback(err));
            });
        })
    },
    getDb: () => dbConnection
}