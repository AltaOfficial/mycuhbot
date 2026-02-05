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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let market = StockMarket::new();
        let users = market.get_users();
        let max_stocks = market.get_max_stocks();

        assert_eq!(users.len(), 0);
        assert_eq!(max_stocks, 100);
    }

    #[test]
    fn test_add_user() {
        let mut market = StockMarket::new();
        
        market.add_user("1111");
        let users = market.get_users();
        
        assert_eq!(users.len(), 1);

        let user = users.get("1111");
        assert!(user.is_some());
    }

    #[test]
    fn test_buy_5() {
        let id = ("1111", "2222");
        let mut market = setup(&[id.0, id.1]);

        let user1 = market.get_users().get("1111").unwrap();
        let user2 = market.get_users().get("2222").unwrap();
        let amount = 5;

        let user1_owned_og = user1.company_stock_owned.get(id.1).unwrap();
        let user2_ = user2.
        market.buy_stock(id.0, id.1, amount);
        assert!(user1.defined_stock_amount_price.0, user1 stock go up);
        assert!(Some(market.get_user_info("2222")));

        market.buy_stock(id.1, id.0, 5);

    }
}

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
