use borsh::{BorshDeserialize, BorshSerialize};
use arch_program::pubkey::Pubkey;
use arch_program::program_error::ProgramError;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct DCAOrder {
    pub owner: Pubkey,
    pub token_pair: (Pubkey, Pubkey),
    pub total_amount: u64,         // Total amount to invest
    pub amount_per_period: u64,    // Amount to invest each period
    pub period_seconds: u64,       // Period length in seconds
    pub next_execution: u64,       // Timestamp for next execution
    pub remaining_periods: u32,    // Number of periods remaining
    pub status: DCAStatus,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum DCAStatus {
    Active,
    Completed,
    Cancelled,
}

impl LimitOrderBook {
    // Add new method to create DCA order
    pub fn create_dca_order(
        &mut self,
        owner: Pubkey,
        token_pair: (Pubkey, Pubkey),
        total_amount: u64,
        periods: u32,
        period_seconds: u64,
        current_time: u64,
    ) -> Result<(), ProgramError> {
        // Validate inputs
        if periods == 0 || total_amount == 0 {
            return Err(ProgramError::Custom(601)); // Invalid parameters
        }

        let amount_per_period = total_amount / periods as u64;
        if amount_per_period == 0 {
            return Err(ProgramError::Custom(602)); // Amount too small for given periods
        }

        let dca_order = DCAOrder {
            owner,
            token_pair,
            total_amount,
            amount_per_period,
            period_seconds,
            next_execution: current_time + period_seconds,
            remaining_periods: periods,
            status: DCAStatus::Active,
        };

        self.dca_orders.push(dca_order);
        Ok(())
    }

    pub fn process_dca_orders(
        &mut self,
        current_time: u64,
        current_market_prices: &[(Pubkey, Pubkey, u64)],
    ) -> Result<Vec<LimitOrder>, ProgramError> {
        let mut executed_orders = Vec::new();

        for dca_order in &mut self.dca_orders {
            if dca_order.status != DCAStatus::Active {
                continue;
            }

            if current_time >= dca_order.next_execution {
                let market_order = LimitOrder {
                    owner: dca_order.owner,
                    token_pair: dca_order.token_pair,
                    amount: dca_order.amount_per_period,
                    price: 0, // Market order, so we accept current price
                    order_type: OrderType::Buy,
                    status: OrderStatus::Open,
                };

                executed_orders.push(market_order);

                // Update DCA order
                dca_order.remaining_periods -= 1;
                dca_order.next_execution += dca_order.period_seconds;

                if dca_order.remaining_periods == 0 {
                    dca_order.status = DCAStatus::Completed;
                }
            }
        }

        Ok(executed_orders)
    }

    pub fn cancel_dca_order(&mut self, owner: &Pubkey, index: usize) -> Result<(), ProgramError> {
        if let Some(order) = self.dca_orders.get_mut(index) {
            if order.owner != *owner {
                return Err(ProgramError::Custom(603)); // Not the owner
            }
            if order.status != DCAStatus::Active {
                return Err(ProgramError::Custom(604)); // Order not active
            }
            order.status = DCAStatus::Cancelled;
            Ok(())
        } else {
            Err(ProgramError::Custom(605)) // Order not found
        }
    }

    pub fn get_dca_order_status(&self, index: usize) -> Result<DCAStatus, ProgramError> {
        self.dca_orders
            .get(index)
            .map(|order| order.status.clone())
            .ok_or(ProgramError::Custom(605))
    }
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct LimitOrderBook {
    orders: Vec<LimitOrder>,
    dca_orders: Vec<DCAOrder>,
}

impl LimitOrderBook {
    pub fn new() -> Self {
        LimitOrderBook {
            orders: Vec::new(),
            dca_orders: Vec::new(),
        }
    }
}

//test code starts from here

#[cfg(test)]
mod tests {
    use super::*;
    use arch_program::pubkey::Pubkey;
    
    // Helper function to create a dummy Pubkey for testing
    fn create_test_pubkey(value: u8) -> Pubkey {
        let mut bytes = [value; 32];
        Pubkey::new(&bytes)
    }

    // Helper function to create market prices for testing
    fn create_test_market_prices(price: u64) -> Vec<(Pubkey, Pubkey, u64)> {
        vec![(
            create_test_pubkey(1),
            create_test_pubkey(2),
            price,
        )]
    }

    #[test]
    fn test_create_dca_order() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let token_pair = (create_test_pubkey(2), create_test_pubkey(3));
        
        let result = order_book.create_dca_order(
            owner,
            token_pair,
            1000,    // total amount
            10,      // periods
            3600,    // 1 hour periods
            1000,    // current time
        );
        
        assert!(result.is_ok());
        assert_eq!(order_book.dca_orders.len(), 1);
        
        let order = &order_book.dca_orders[0];
        assert_eq!(order.owner, owner);
        assert_eq!(order.token_pair, token_pair);
        assert_eq!(order.total_amount, 1000);
        assert_eq!(order.amount_per_period, 100); // 1000/10
        assert_eq!(order.remaining_periods, 10);
        assert!(matches!(order.status, DCAStatus::Active));
    }

    #[test]
    fn test_create_dca_order_invalid_parameters() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let token_pair = (create_test_pubkey(2), create_test_pubkey(3));
        
        // Test with zero periods
        let result = order_book.create_dca_order(
            owner,
            token_pair,
            1000,
            0,      // zero periods should fail
            3600,
            1000,
        );
        assert!(matches!(result, Err(ProgramError::Custom(601))));
        
        // Test with zero total amount
        let result = order_book.create_dca_order(
            owner,
            token_pair,
            0,      // zero amount should fail
            10,
            3600,
            1000,
        );
        assert!(matches!(result, Err(ProgramError::Custom(601))));
    }

    #[test]
    fn test_process_dca_orders() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let token_pair = (create_test_pubkey(2), create_test_pubkey(3));
        let start_time = 1000;
        
        // Create DCA order
        order_book.create_dca_order(
            owner,
            token_pair,
            1000,
            5,          // 5 periods
            3600,       // 1 hour periods
            start_time,
        ).unwrap();
        
        // Test processing before execution time
        let result = order_book.process_dca_orders(
            start_time + 1800, // Half period
            &create_test_market_prices(100),
        ).unwrap();
        assert_eq!(result.len(), 0); // No orders should be executed
        
        // Test processing at execution time
        let result = order_book.process_dca_orders(
            start_time + 3600, // Full period
            &create_test_market_prices(100),
        ).unwrap();
        assert_eq!(result.len(), 1); // One order should be executed
        
        let executed_order = &result[0];
        assert_eq!(executed_order.amount, 200); // 1000/5
        assert!(matches!(executed_order.status, OrderStatus::Open));
        
        // Check DCA order was updated
        let dca_order = &order_book.dca_orders[0];
        assert_eq!(dca_order.remaining_periods, 4);
        assert_eq!(dca_order.next_execution, start_time + 7200); // Next period
    }

    #[test]
    fn test_dca_order_completion() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let token_pair = (create_test_pubkey(2), create_test_pubkey(3));
        let start_time = 1000;
        
        // Create DCA order with 2 periods
        order_book.create_dca_order(
            owner,
            token_pair,
            1000,
            2,          // 2 periods
            3600,       // 1 hour periods
            start_time,
        ).unwrap();
        
        // Process first period
        order_book.process_dca_orders(
            start_time + 3600,
            &create_test_market_prices(100),
        ).unwrap();
        
        // Process second (final) period
        order_book.process_dca_orders(
            start_time + 7200,
            &create_test_market_prices(100),
        ).unwrap();
        
        let dca_order = &order_book.dca_orders[0];
        assert_eq!(dca_order.remaining_periods, 0);
        assert!(matches!(dca_order.status, DCAStatus::Completed));
    }

    #[test]
    fn test_cancel_dca_order() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let other_owner = create_test_pubkey(2);
        let token_pair = (create_test_pubkey(3), create_test_pubkey(4));
        
        // Create DCA order
        order_book.create_dca_order(
            owner,
            token_pair,
            1000,
            5,
            3600,
            1000,
        ).unwrap();
        
        // Test cancellation by wrong owner
        let result = order_book.cancel_dca_order(&other_owner, 0);
        assert!(matches!(result, Err(ProgramError::Custom(603))));
        
        // Test successful cancellation
        let result = order_book.cancel_dca_order(&owner, 0);
        assert!(result.is_ok());
        
        let dca_order = &order_book.dca_orders[0];
        assert!(matches!(dca_order.status, DCAStatus::Cancelled));
        
        // Test cancelling already cancelled order
        let result = order_book.cancel_dca_order(&owner, 0);
        assert!(matches!(result, Err(ProgramError::Custom(604))));
    }

    #[test]
    fn test_get_dca_order_status() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let token_pair = (create_test_pubkey(2), create_test_pubkey(3));
        
        // Test getting status of non-existent order
        let result = order_book.get_dca_order_status(0);
        assert!(matches!(result, Err(ProgramError::Custom(605))));
        
        // Create order and test status
        order_book.create_dca_order(
            owner,
            token_pair,
            1000,
            5,
            3600,
            1000,
        ).unwrap();
        
        let status = order_book.get_dca_order_status(0).unwrap();
        assert!(matches!(status, DCAStatus::Active));
        
        // Cancel order and test status
        order_book.cancel_dca_order(&owner, 0).unwrap();
        let status = order_book.get_dca_order_status(0).unwrap();
        assert!(matches!(status, DCAStatus::Cancelled));
    }

    #[test]
    fn test_amount_distribution() {
        let mut order_book = LimitOrderBook::new();
        let owner = create_test_pubkey(1);
        let token_pair = (create_test_pubkey(2), create_test_pubkey(3));
        let total_amount = 1000;
        let periods = 3;
        
        order_book.create_dca_order(
            owner,
            token_pair,
            total_amount,
            periods,
            3600,
            1000,
        ).unwrap();
        
        let dca_order = &order_book.dca_orders[0];
        let expected_amount_per_period = total_amount / periods as u64;
        
        assert_eq!(dca_order.amount_per_period, expected_amount_per_period);
        assert_eq!(dca_order.amount_per_period * periods as u64, total_amount);
    }
}

