use rust_decimal::Decimal;

pub type AccountId = u16;

/// State of what an account hold of money.
/// `held` is what is the amount of money held due to disputes.
#[derive(Debug, Default)]
pub struct Wallet {
    amount: Decimal,
    held: Decimal,
}

impl Wallet {
    /// Get the real amount of a wallet.
    pub fn available_funds(&self) -> Decimal {
        self.amount - self.held
    }
}

/// A customer account with its wallet.
/// It's just a business encapsulation.
#[derive(Debug)]
pub struct Account {
    id: AccountId,
    wallet: Wallet,
    pub locked: bool,
}

impl Account {
    pub fn new(id: AccountId) -> Self {
        Self {
            id,
            wallet: Wallet::default(),
            locked: false,
        }
    }

    pub fn get_id(&self) -> AccountId {
        self.id
    }

    pub fn deposit(&mut self, amount: Decimal) {
        if self.locked {
            panic!()
        }

        if amount <= Decimal::ZERO {
            panic!("Only can deposit positive amount")
        }

        self.wallet.amount = self.wallet.amount.checked_add(amount).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn account_can_deposit_funds() {
        let mut acc = Account::new(0);
        acc.deposit(dec!(1.773));
        assert_eq!(acc.wallet.amount, dec!(1.773));
        acc.deposit(dec!(1.664));
        assert_eq!(acc.wallet.amount, dec!(3.437));
    }

    #[test]
    #[should_panic(expected = "Only can deposit positive amount")]
    fn account_cannot_deposit_negative_funds() {
        let mut acc = Account::new(0);
        acc.deposit(dec!(-1));
    }
}
