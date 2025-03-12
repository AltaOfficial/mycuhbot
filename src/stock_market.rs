use core::f32;
use std::collections::HashMap;
use std::fmt;
use rand::Rng;

#[derive(Debug)]
pub struct User {
    user_id: String, // may not need, already exist in 'users' hashmap
    bal: f32,
    defined_stock_amount_price: (u32, f32), // (amount, price) 
    company_stock_owned: HashMap<String, u32>, // <ID, amount> Other companies they own stock in
}

impl fmt::Display for User {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "User ID: {}\nBalance: ${}\nDefined Stock: {} at ${}\nOwned Stocks: {:?}",
            self.user_id,
            self.bal,
            self.defined_stock_amount_price.0,
            self.defined_stock_amount_price.1,
            self.company_stock_owned
        )
    }
}


pub struct StockMarket {
    users: HashMap<String, User>, // Mapping of user_id to User object
    max_stocks: u32,
}


impl StockMarket {
    pub fn new() -> StockMarket {
        StockMarket {
            users: HashMap::new(),
            max_stocks: 100,
        }
    }

    // Add a user to the market with a defined stock
    pub fn add_user(&mut self, id: &str) {
        // adds new user to all other users list
        let id_str = id.to_string();
        for (_, user) in self.users.iter_mut() {
            user.company_stock_owned.insert(id_str.clone(), 0);
        }

        let mut new_user = User {
            user_id: id_str.clone(),
            bal: 10.0,
            defined_stock_amount_price: (self.max_stocks, 1.0), // Default stock: 100 units at price 10.0
            company_stock_owned: HashMap::new(), // No other stock owned initially
        };
        // adds all other users to new user stock list
        for company_id in self.users.keys() {
            new_user.company_stock_owned.insert(company_id.clone(), 0);
        }

        // add new user to whole list
        self.users.insert(id_str, new_user);
    }

    pub fn get_user_info(&self, id: &str) -> &User {
        self.users.get(id).unwrap()
    }

    pub fn buy_stock(&mut self, buyer_id: &str, seller_id: &str, amount: u32) {
        self.trade_stock(buyer_id, seller_id, amount, true);
    }

    pub fn sell_stock(&mut self, seller_id: &str, company_id: &str, amount: u32) {
        self.trade_stock(company_id, seller_id, amount, false); // Selling is just the reverse of buying in logic
    }

    fn trade_stock(&mut self, buyer_id: &str, seller_id: &str, amount: u32, is_buying: bool) {
        // names buyer and seller make it easier to read, logic changes based on higher order
        // functions
        if let (Some(mut buyer), Some(mut seller)) = (
            self.users.remove(buyer_id), 
            self.users.remove(seller_id)
        ){
            // Ensure buyer has enough balance and seller has enough stock to sell
            // buy shares of company
            if is_buying {
                let cost = seller.defined_stock_amount_price.1 * amount as f32;
                if buyer.bal >= cost && seller.defined_stock_amount_price.0 >= amount {
                    buyer.bal -= cost;
                    seller.bal += cost;

                    // Update stock ownership
                    buyer.company_stock_owned.entry(seller_id.to_string()).and_modify(|stocks| *stocks += amount);
                    seller.company_stock_owned.entry(seller_id.to_string()).and_modify(|stocks| *stocks -= amount);
                    seller.defined_stock_amount_price.0 -= amount;
                } else {
                    println!("Was unable to buy/sell {} stocks", amount);
                }

            // sell shares of company
            } else { 
                let cost = buyer.defined_stock_amount_price.1 * amount as f32;
                if buyer.bal >= cost && *seller.company_stock_owned.get(buyer_id).unwrap() >= amount {
                    buyer.bal -= cost;
                    seller.bal += cost;

                    // Update stock ownership
                    buyer.company_stock_owned.entry(buyer_id.to_string()).and_modify(|stocks| *stocks += amount);
                    seller.company_stock_owned.entry(buyer_id.to_string()).and_modify(|stocks| *stocks -= amount);
                    buyer.defined_stock_amount_price.0 += amount;
                } else {
                    println!("Was unable to buy/sell {} stocks", amount);
                }
            }

            // adjust stock price based on what was sold or bought
            if is_buying {
                //seller = self.adjust_stock_price(seller, amount, is_buying);
                adjust_stock_price(&mut seller, amount, is_buying);
            } else {
                //buyer = self.adjust_stock_price(buyer, amount, is_buying);
                adjust_stock_price(&mut buyer, amount, is_buying);
            }
            self.users.insert(buyer_id.to_string(), buyer);
            self.users.insert(seller_id.to_string(), seller);
        }
    }



    pub fn random_market_event(&mut self) {
        let mut rng = rand::rng();
        
        let type_of_event = rng.random_range(1.0..=100.0);
        match type_of_event {
            1.0..=20.0 => {
                (|| { 
                    let is_inflation = rng.random_bool(0.8);
                    let change_percent = exp_weighted_change(1.5);
                    let mut stock_value_change;

                    for user in self.users.values_mut() {
                        stock_value_change = (user.defined_stock_amount_price.1 * change_percent).round() / 100.0;
                        if is_inflation {
                            user.defined_stock_amount_price.1 -= stock_value_change;
                            println!("Inflation: stock price decreased for {} by {:.2}%", user.user_id, stock_value_change);
                        } else {
                            user.defined_stock_amount_price.1 += stock_value_change;
                            println!("Deflation: stock price increased for {} by {:.2}%", user.user_id, stock_value_change);
                        }
                    }
                })
                ();
            },

            20.0..=100.0 => { 
                (|| {
                    let rand_index = rng.random_range(1.0..=self.users.len() as f32);
                    if let Some(rand_user) = self.users.keys().nth(rand_index as usize).cloned() {
                        if let Some(user) = self.users.get_mut(&rand_user) {
                            let change_percent = exp_weighted_change(0.7);

                            let stock_value_change = (user.defined_stock_amount_price.1 * change_percent).round() / 100.0;

                            if rng.random_bool(0.5) {
                                user.defined_stock_amount_price.1 += stock_value_change;
                                println!("Company {} just increased stock price by {}%", rand_user, stock_value_change * 100.0);
                            } else {
                                user.defined_stock_amount_price.1 -= stock_value_change;
                                println!("Company {} just decreased stock price by {}%", rand_user, stock_value_change * 100.0);
                            }
                        }
                    }
                })
                (); 
            },

            _ => {
                println!("how the hecking heck did you get here!??!");
            }
        }
    }

    pub fn bankruptcy(&mut self, id: &str) {
        // return stocks back to company and adjust stock price for respective company
        if let Some(bankrupt_user) = self.users.get(id) {
            let bankrupt_stock = bankrupt_user.company_stock_owned.clone();

            for user in self.users.values_mut() {
                if let Some(amount) = bankrupt_stock.get(&user.user_id) {
                    user.defined_stock_amount_price.0 += amount;
                    adjust_stock_price(user, *amount, false);
                }
            }

            // Remove bankrupt user from users list
            self.users.remove(id);
        } else {
            return;
        }
        // removes listing of bankrupt user for other users
        for (_, user) in self.users.iter_mut() {
            user.company_stock_owned.remove(id);
        }
        println!("user {} has been removed!", id);
    }
}


// Adjust stock price based on market dynamics 
// should change based on how many buys of a share
// should also change based on number of things bought from user
fn adjust_stock_price(company: &mut User, amount: u32, is_buying: bool) /*-> User*/ {
    let mut price = company.defined_stock_amount_price.1;

    //let max_price = 100.0;
    //let midpoint: f32 = max_price / 2.0;
    let price_rate = 0.01;

    // Exponentially change the price
    if is_buying {
        //price = price + (amount as f32 * price_rate); // linear 
        //price = price * (1.0_f32 + price_rate).powf(amount as f32); // exponential
        price = price + ((1.0_f32 + amount as f32).log10() * price_rate); // logarithmic
        //price = max_price / (1.0_f32 + f32::consts::E.powf(-(amount as f32 - midpoint) * price_rate)) // Sigmoid (S-curve) doesn't work
    } else {
        //price = price - (amount as f32 * price_rate); // linear
        //price = price / (1.0_f32 + price_rate).powf(amount as f32); // exponential
        price = price - ((1.0_f32 + amount as f32).log10() * price_rate); // logarithmic
        //price = max_price / (1.0_f32 + f32::consts::E.powf(-(amount as f32 - midpoint) * price_rate)) // Sigmoid (S-curve) doesn't work
    }
    price = (price * 10000.0).round() / 10000.0; // round to sigfig 0.0001
    
    // Update the stock price for the company
    company.defined_stock_amount_price.1 = price;
    //company
}

fn exp_weighted_change(rate: f32) -> f32 {
    let exp_value = || {
        let mut rng = rand::rng();
        let uniform_random: f32 = rng.random();
        let exp_random = -uniform_random.ln() / rate;  // Inverse transform sampling
        exp_random
    };

    (exp_value() % 50.0) + 1.0
}

