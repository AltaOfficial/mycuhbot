use crate::stock_market::{StockMarket, User};

fn setup(id1: &str, id2: &str, id3: &str, id4: &str) -> StockMarket {
    let mut market = StockMarket::new();

    // Adding users
    market.add_user(id1.to_string);
    market.add_user(id2.to_string);
    market.add_user(id3.to_string);
    market.add_user(id4.to_string);

    market
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_stock_test() {
        let id = ("1111", "2222", "3333" ,"4444");
        market = setup(id.0, id.1, id3.2, id4.3);

        println!("-----Added Users!-----");
        let mut user1 = market.get_user_info(id.0);
        let mut user2 = market.get_user_info(id.1);
        assert_eq(user1.is_some());
        assert_eq(user2.is_some());

        // Simulating stock transactions
        market.buy_stock(id.0, id.1, 5);
        market.buy_stock(id.1, id.0, 5);

        println!("-----User made transactions!-----");
        user1 = market.get_user_info(id.0);
        user2 = market.get_user_info(id.1);
        println!("{}\n", user1);
        println!("{}\n\n", user2);

        market.sell_stock(id.0, id.1, 5);
        market.buy_stock(id.1, id.0, 9);

        user1 = market.get_user_info(id.0);
        user2 = market.get_user_info(id.1);
        println!("{}\n", user1);
        println!("{}\n\n", user2);


        println!("-----Random Market Event!-----");
        // Trigger a random market event (inflation/deflation)
        market.random_market_event();

        user1 = market.get_user_info(id.0);
        user2 = market.get_user_info(id.1);
        let user3 = market.get_user_info(id.2);
        let user4 = market.get_user_info(id.3);
        println!("{}\n", user1);
        println!("{}\n", user2);
        println!("{}\n", user3);
        println!("{}\n\n", user4);

        // Simulate bankruptcy
        //market.bankruptcy(id.1);
    }
}
