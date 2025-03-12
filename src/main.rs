mod stock_market;
mod server;
mod cock;

use stock_market::StockMarket;

fn setup(id1: &str, id2: &str, id3: &str, id4: &str) -> StockMarket {
    let mut market = stock_market::StockMarket::new();

    // Adding users
    market.add_user(id1);
    market.add_user(id2);
    market.add_user(id3);
    market.add_user(id4);

    market
}


fn main() {
    // mongodb server call
    //let server = server::main();
    //println!("{:?}", server.unwrap());


    let id = ("1111", "2222", "3333" ,"4444");
    let mut market = setup(id.0, id.1, id.2, id.3);

    println!("-----Added Users!-----");
    let mut user1 = market.get_user_info(id.0);
    let mut user2 = market.get_user_info(id.1);
    println!("{}\n{}\n", user1, user2);

    // Simulating stock transactions
    market.buy_stock(id.0, id.1, 5);
    market.buy_stock(id.1, id.0, 5);

    println!("-----User made transactions!-----");
    user1 = market.get_user_info(id.0);
    user2 = market.get_user_info(id.1);
    println!("{}\n{}\n", user1, user2);

    market.sell_stock(id.0, id.1, 5);
    market.buy_stock(id.1, id.0, 9);

    user1 = market.get_user_info(id.0);
    user2 = market.get_user_info(id.1);
    println!("{}\n{}\n", user1, user2);


    println!("-----Random Market Event!-----");
    // Trigger a random market event (inflation/deflation)
    market.random_market_event();

    user1 = market.get_user_info(id.0);
    user2 = market.get_user_info(id.1);
    let user3 = market.get_user_info(id.2);
    let user4 = market.get_user_info(id.3);
    println!("{}\n{}\n{}\n{}\n", user1, user2, user3, user4);

    // Simulate bankruptcy
    //market.bankruptcy(id.1);
}

