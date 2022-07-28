use std::fmt::{self, Display, Formatter};

use rust_decimal::Decimal;

use crate::errors::AccountOperationError;

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

    fn is_locked(&self) -> Result<(), AccountOperationError> {
        match self.locked {
            false => Ok(()),
            true => Err(AccountOperationError::AccountLocked(self.id)),
        }
    }

    pub fn get_id(&self) -> AccountId {
        self.id
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<(), AccountOperationError> {
        self.is_locked()?;

        if Decimal::ZERO >= amount {
            return Err(AccountOperationError::NonPositiveAmount);
        }

        self.wallet.amount = self
            .wallet
            .amount
            .checked_add(amount)
            .ok_or(AccountOperationError::OverflowInWallet)?;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), AccountOperationError> {
        self.is_locked()?;

        if Decimal::ZERO >= amount {
            return Err(AccountOperationError::NonPositiveAmount);
        }

        if amount > self.wallet.available_funds() {
            return Err(AccountOperationError::InsufficientFunds);
        }

        self.wallet.amount -= amount;
        Ok(())
    }

    pub fn hold(&mut self, amount: Decimal) -> Result<(), AccountOperationError> {
        self.is_locked()?;

        if Decimal::ZERO >= amount {
            return Err(AccountOperationError::NonPositiveAmount);
        }

        // assuming that you can't hold what you don't have
        if amount > self.wallet.available_funds() {
            return Err(AccountOperationError::InsufficientFunds);
        }
        self.wallet.held += amount;
        Ok(())
    }

    pub fn unhold(&mut self, amount: Decimal) -> Result<(), AccountOperationError> {
        self.is_locked()?;

        if Decimal::ZERO >= amount {
            return Err(AccountOperationError::NonPositiveAmount);
        }

        if amount > self.wallet.held {
            return Err(AccountOperationError::InsufficientFunds);
        }

        self.wallet.held -= amount;

        Ok(())
    }
}

/// We use this Display impl to output an Account to a csv record.
impl Display for Account {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "{},{},{},{},{}",
            self.get_id(),
            self.wallet.available_funds(),
            self.wallet.held,
            self.wallet.amount,
            self.locked
        )
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;
    use crate::errors::AccountOperationError::*;

    #[test]
    fn account_can_deposit_funds() {
        let mut acc = Account::new(0);
        let _ = acc.deposit(dec!(1.773));
        assert_eq!(acc.wallet.amount, dec!(1.773));
        let _ = acc.deposit(dec!(1.664));
        assert_eq!(acc.wallet.amount, dec!(3.437));
    }

    #[test]
    fn account_cannot_deposit_negative_funds() {
        let mut acc = Account::new(0);

        let expected_error = Err(NonPositiveAmount);
        let result = acc.deposit(dec!(-1));
        assert_eq!(result, expected_error);
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
        let _ = acc.withdraw(dec!(1.773));
        assert_eq!(acc.wallet.amount, dec!(1662.227));
        let _ = acc.withdraw(dec!(1.664));
        assert_eq!(acc.wallet.amount, dec!(1660.563));
    }

    #[test]
    fn account_cannot_withdraw_funds_with_empty_wallet() {
        let mut acc = Account::new(0);
        let expected_error = Err(InsufficientFunds);
        let result = acc.withdraw(dec!(1.773));
        assert_eq!(result, expected_error);
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
        let _ = acc.hold(dec!(10));
        assert_eq!(acc.wallet.held, dec!(10));
    }

    #[test]
    fn account_cannot_hold_funds_with_an_empty_wallet() {
        let mut acc = Account::new(0);
        let expected_error = Err(InsufficientFunds);
        let result = acc.withdraw(dec!(1.773));
        assert_eq!(result, expected_error);
    }

    #[test]
    fn account_can_unhold_funds() {
        let mut acc = Account::new_with_wallet(
            0,
            Wallet {
                amount: dec!(1664),
                held: dec!(10),
            },
        );
        let _ = acc.unhold(dec!(10));
        assert_eq!(acc.wallet.held, dec!(0));
    }
}
