use uuid::Uuid;
use crate::currency::Currency;
use crate::error::PaySwitchError;
use crate::ledger::{Amount, Ledger, Transfer};

pub struct CrossBorderRoute {
    pub source_currency: Currency,
    pub target_currency: Currency,
    /// Exchange rate represented as atomic ratio (e.g. 1 USD = 120.50 BDT -> 12050 / 100).
    pub rate_numerator: u128,
    pub rate_denominator: u128,
    /// Platform switching fee basis points (e.g. 80 bps = 0.80%).
    pub fee_bps: u128,
}

impl CrossBorderRoute {
    pub fn new(
        source_currency: Currency,
        target_currency: Currency,
        rate_numerator: u128,
        rate_denominator: u128,
        fee_bps: u128,
    ) -> Self {
        Self {
            source_currency,
            target_currency,
            rate_numerator,
            rate_denominator,
            fee_bps,
        }
    }

    /// Calculates net target atomic amount and platform processing fee.
    pub fn convert(&self, source_amount: Amount) -> (Amount, Amount) {
        // Fee = source_amount * fee_bps / 10000
        let fee = (source_amount.0 * self.fee_bps) / 10000;
        let net_source = source_amount.0.saturating_sub(fee);

        // Target = net_source * rate_numerator / rate_denominator
        let target_amount = (net_source * self.rate_numerator) / self.rate_denominator;

        (Amount(target_amount), Amount(fee))
    }
}

pub struct PaymentRouter<'a> {
    ledger: &'a Ledger,
}

impl<'a> PaymentRouter<'a> {
    pub fn new(ledger: &'a Ledger) -> Self {
        Self { ledger }
    }

    /// Dispatches an atomic cross-border settlement from Inbound Nostro to Outbound Vostro/Merchant.
    pub fn route_cross_border_settlement(
        &self,
        inbound_nostro_id: Uuid,
        platform_fee_id: Uuid,
        merchant_account_id: Uuid,
        inbound_amount: Amount,
        route: &CrossBorderRoute,
    ) -> Result<(Transfer, Amount), PaySwitchError> {
        let (converted_amount, fee_amount) = route.convert(inbound_amount);

        // 1. Collect platform routing fee
        if fee_amount.0 > 0 {
            self.ledger.execute_transfer(
                inbound_nostro_id,
                platform_fee_id,
                fee_amount,
                route.source_currency,
            )?;
        }

        // 2. Settlement delivery to merchant
        let transfer = self.ledger.execute_transfer(
            inbound_nostro_id,
            merchant_account_id,
            Amount(inbound_amount.0 - fee_amount.0),
            route.source_currency,
        )?;

        Ok((transfer, converted_amount))
    }
}
