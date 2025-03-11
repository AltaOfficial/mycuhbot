mod stock_market;
mod server;

fn main() {
    // mongodb server call
    let server = server::main();
    //println!("{:?}", server.unwrap());


    /*
    let mut market = stock_market::StockMarket::new();

    // Adding users
    market.add_user(1111);
    market.add_user(2222);
    market.add_user(3333);
    market.add_user(4444);

    println!("-----Added Users!-----");
    let mut user1 = market.get_user_info(1111);
    let mut user2 = market.get_user_info(2222);
    println!("{}\n", user1);
    println!("{}\n\n", user2);

    // Simulating stock transactions
    market.buy_stock(1111, 2222, 5);
    market.buy_stock(2222, 1111, 5);

    println!("-----User made transactions!-----");
    user1 = market.get_user_info(1111);
    user2 = market.get_user_info(2222);
    println!("{}\n", user1);
    println!("{}\n\n", user2);

    market.sell_stock(1111, 2222, 5);
    market.buy_stock(2222, 1111, 9);

    user1 = market.get_user_info(1111);
    user2 = market.get_user_info(2222);
    println!("{}\n", user1);
    println!("{}\n\n", user2);


    println!("-----Random Market Event!-----");
    // Trigger a random market event (inflation/deflation)
    market.random_market_event();

    user1 = market.get_user_info(1111);
    user2 = market.get_user_info(2222);
    let mut user3 = market.get_user_info(3333);
    let mut user4 = market.get_user_info(4444);
    println!("{}\n", user1);
    println!("{}\n", user2);
    println!("{}\n", user3);
    println!("{}\n\n", user4);

    println!("----User 2 Files Bankruptcy!----");
    // Simulate bankruptcy
    market.bankruptcy(2222);

    user1 = market.get_user_info(1111);
    user3 = market.get_user_info(3333);
    user4 = market.get_user_info(4444);
    println!("{}\n", user1);
    println!("{}\n", user3);
    println!("{}\n\n", user4);
    */
}

