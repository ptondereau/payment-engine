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

    #[cfg(test)]
    pub fn new_with_wallet(id: AccountId, wallet: Wallet) -> Self {
        Self {
            id,
            wallet,
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

    pub fn withdraw(&mut self, amount: Decimal) {
        if self.locked {
            panic!()
        }

        if amount <= Decimal::ZERO {
            panic!("Only can withdraw positive amount")
        }

        if amount > self.wallet.available_funds() {
            panic!("Insufficent funds in the wallet")
        }

        self.wallet.amount -= amount;
    }

    pub fn hold(&mut self, amount: Decimal) {
        if self.locked {
            panic!()
        }

        if amount <= Decimal::ZERO {
            panic!("Only can hold positive amount")
        }

        // assuming that you can't hold what you don't have
        if amount > self.wallet.available_funds() {
            panic!("Insufficent funds in the wallet")
        }
        self.wallet.held += amount;
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

    #[test]
    fn account_can_withdraw_funds() {
        let mut acc = Account::new_with_wallet(
            0,
            Wallet {
                amount: dec!(1664),
                held: dec!(0),
            },
        );
        acc.withdraw(dec!(1.773));
        assert_eq!(acc.wallet.amount, dec!(1662.227));
        acc.withdraw(dec!(1.664));
        assert_eq!(acc.wallet.amount, dec!(1660.563));
    }

    #[test]
    #[should_panic(expected = "Insufficent funds in the wallet")]
    fn account_cannot_withdraw_funds_with_negative_or_zero_in_its_wallet() {
        let mut acc = Account::new(0);
        acc.withdraw(dec!(1.773));
        assert_eq!(acc.wallet.amount, dec!(1662.227));
    }

    #[test]
    fn account_can_hold_funds() {
        let mut acc = Account::new_with_wallet(
            0,
            Wallet {
                amount: dec!(1664),
                held: dec!(0),
            },
        );
        acc.hold(dec!(10));
        assert_eq!(acc.wallet.held, dec!(10));
    }

    #[test]
    #[should_panic(expected = "Insufficent funds in the wallet")]
    fn account_cannot_hold_funds_with_an_empty_wallet() {
        let mut acc = Account::new(0);
        acc.withdraw(dec!(1.773));
    }
}
