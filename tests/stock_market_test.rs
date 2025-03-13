mod common;

use mycuhbot::stock_market::*;


fn setup(users: &[&str]) -> StockMarket {
    let mut market = StockMarket::new();

    // adding users
    for user in users {
        market.add_user(user);
    }

    market
}


#[test]
fn test_new() {
    let market = StockMarket::new();
    let users = market.get_users();
    let max_stocks = market.get_max_stocks();

    assert_eq!(users.len(), 0);
}

#[test]
fn test_add_user() {
    let mut market = StockMarket::new();
    market.add_user("1111");

    assert!(market.get_user_info("1111"));
}

/*
#[test]
fn test_buy_5() {
    let id = ("1111", "2222");
    let mut market = setup(&[id.0, id.1]);

    let user1 = market.get_user_info("1111");
    let user2 = market.get_user_info("2222");
    market.buy_stock(id.0, id.1, 5);
    assert_ne!(market.get_user_info("1111"), user1);
    assert!(Some(market.get_user_info("2222")));

    market.buy_stock(id.1, id.0, 5);

}
*/

/*
    #[test]
    fn general_stock_test() {
        let id = ("1111", "2222", "3333" ,"4444");
        market = setup(id.0, id.1, id3.2, id4.3);

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
        let mut user3;
        let mut user4;

        for _ in 1..10 {
            // Trigger a random market event (inflation/deflation)
            market.random_market_event();

            user1 = market.get_user_info(id.0);
            user2 = market.get_user_info(id.1);
            user3 = market.get_user_info(id.2);
            user4 = market.get_user_info(id.3);
            println!("{}\n{}\n{}\n{}\n", user1, user2, user3, user4);
        }

        // Simulate bankruptcy
        println!("-----User 2 Filed Bankruptcy!-----");
        market.bankruptcy(id.1);

        user1 = market.get_user_info(id.0);
        user2 = market.get_user_info(id.1);
        user3 = market.get_user_info(id.2);
        user4 = market.get_user_info(id.3);
        println!("{}\n{}\n{}\n{}\n", user1, user2, user3, user4);
        */
    //}
//}
