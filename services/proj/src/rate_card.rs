//! FR-PROJ-005 — per-engagement rate-card versioning.

use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingRole {
    Engineer,
    Designer,
    Pm,
    Qa,
    Analyst,
    Exec,
}

impl BillingRole {
    pub const ALL: [Self; 6] = [
        Self::Engineer,
        Self::Designer,
        Self::Pm,
        Self::Qa,
        Self::Analyst,
        Self::Exec,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Currency {
    VND,
    USD,
    SGD,
    EUR,
    JPY,
}

impl Currency {
    pub const ALL: [Self; 5] = [Self::VND, Self::USD, Self::SGD, Self::EUR, Self::JPY];

    pub const fn decimals(self) -> u8 {
        match self {
            Self::VND | Self::JPY => 0,
            Self::USD | Self::SGD | Self::EUR => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateCard {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub engagement_id: Uuid,
    pub role: BillingRole,
    pub currency: Currency,
    pub hourly_rate_minor: i64,
    pub billable_default: bool,
    pub effective_from: NaiveDate,
    /// Exclusive end date. `None` means active/current.
    pub effective_to: Option<NaiveDate>,
    pub archived: bool,
    pub created_by_subject_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RateCardError {
    #[error("negative hourly rate: {0}")]
    NegativeAmount(i64),
    #[error("overlapping rate card interval")]
    OverlapConflict,
    #[error("effective_from too far in future")]
    EffectiveFromTooFar,
    #[error("rate card not found")]
    NotFound,
}

pub fn create_rate_card(
    tenant_id: Uuid,
    engagement_id: Uuid,
    role: BillingRole,
    currency: Currency,
    hourly_rate_minor: i64,
    billable_default: bool,
    effective_from: NaiveDate,
    created_by_subject_id: Uuid,
) -> Result<RateCard, RateCardError> {
    validate_rate(hourly_rate_minor, effective_from)?;
    Ok(RateCard {
        id: Uuid::new_v4(),
        tenant_id,
        engagement_id,
        role,
        currency,
        hourly_rate_minor,
        billable_default,
        effective_from,
        effective_to: None,
        archived: false,
        created_by_subject_id,
    })
}

pub fn supersede_active(
    cards: &mut Vec<RateCard>,
    new_card: RateCard,
) -> Result<Option<Uuid>, RateCardError> {
    ensure_no_overlap(cards, &new_card)?;
    let mut superseded = None;
    for card in cards.iter_mut().filter(|c| {
        c.engagement_id == new_card.engagement_id
            && c.role == new_card.role
            && c.currency == new_card.currency
            && c.effective_to.is_none()
            && !c.archived
    }) {
        card.effective_to = Some(new_card.effective_from);
        superseded = Some(card.id);
    }
    cards.push(new_card);
    Ok(superseded)
}

pub fn lookup_at(
    cards: &[RateCard],
    engagement_id: Uuid,
    role: BillingRole,
    currency: Currency,
    at: NaiveDate,
) -> Result<&RateCard, RateCardError> {
    cards
        .iter()
        .filter(|c| {
            c.engagement_id == engagement_id
                && c.role == role
                && c.currency == currency
                && !c.archived
                && c.effective_from <= at
                && c.effective_to.map(|end| at < end).unwrap_or(true)
        })
        .max_by_key(|c| c.effective_from)
        .ok_or(RateCardError::NotFound)
}

pub fn preview_supersession(
    cards: &[RateCard],
    new_card: &RateCard,
) -> Result<Option<Uuid>, RateCardError> {
    ensure_no_overlap(cards, new_card)?;
    Ok(cards
        .iter()
        .find(|c| {
            c.engagement_id == new_card.engagement_id
                && c.role == new_card.role
                && c.currency == new_card.currency
                && c.effective_to.is_none()
                && !c.archived
        })
        .map(|c| c.id))
}

fn validate_rate(hourly_rate_minor: i64, effective_from: NaiveDate) -> Result<(), RateCardError> {
    if hourly_rate_minor < 0 {
        return Err(RateCardError::NegativeAmount(hourly_rate_minor));
    }
    let max = Utc::now().date_naive() + Duration::days(365);
    if effective_from > max {
        return Err(RateCardError::EffectiveFromTooFar);
    }
    Ok(())
}

fn ensure_no_overlap(cards: &[RateCard], new_card: &RateCard) -> Result<(), RateCardError> {
    for card in cards.iter().filter(|c| {
        c.engagement_id == new_card.engagement_id
            && c.role == new_card.role
            && c.currency == new_card.currency
    }) {
        let old_start = card.effective_from;
        let old_end = card.effective_to.unwrap_or(NaiveDate::MAX);
        let new_start = new_card.effective_from;
        let new_end = new_card.effective_to.unwrap_or(NaiveDate::MAX);
        if new_start < old_end && old_start < new_end && card.effective_to.is_some() {
            return Err(RateCardError::OverlapConflict);
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct RateCardAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub engagement_id: Uuid,
    pub role: BillingRole,
    pub currency: Currency,
    pub old_card_id: Option<Uuid>,
    pub new_card_id: Uuid,
    pub new_rate_minor: i64,
}

pub fn audit_created(card: &RateCard) -> RateCardAuditRow {
    RateCardAuditRow {
        kind: "proj.rate_card_created",
        tenant_id: card.tenant_id,
        engagement_id: card.engagement_id,
        role: card.role,
        currency: card.currency,
        old_card_id: None,
        new_card_id: card.id,
        new_rate_minor: card.hourly_rate_minor,
    }
}

pub fn audit_superseded(old_card_id: Uuid, card: &RateCard) -> RateCardAuditRow {
    RateCardAuditRow {
        kind: "proj.rate_card_superseded",
        old_card_id: Some(old_card_id),
        ..audit_created(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        Utc::now().date_naive()
    }

    fn card(rate: i64, effective_from: NaiveDate) -> RateCard {
        create_rate_card(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BillingRole::Engineer,
            Currency::USD,
            rate,
            true,
            effective_from,
            Uuid::new_v4(),
        )
        .unwrap()
    }

    #[test]
    fn roles_and_currencies_are_closed() {
        assert_eq!(BillingRole::ALL.len(), 6);
        assert_eq!(Currency::ALL.len(), 5);
        assert_eq!(Currency::VND.decimals(), 0);
        assert_eq!(Currency::USD.decimals(), 2);
    }

    #[test]
    fn lookup_uses_effective_date_versions() {
        let first = card(10_000, today());
        let engagement = first.engagement_id;
        let tenant = first.tenant_id;
        let role = first.role;
        let currency = first.currency;
        let mut cards = vec![first];
        let second = create_rate_card(
            tenant,
            engagement,
            role,
            currency,
            20_000,
            true,
            today() + Duration::days(10),
            Uuid::new_v4(),
        )
        .unwrap();
        let old = preview_supersession(&cards, &second).unwrap().unwrap();
        assert_eq!(supersede_active(&mut cards, second).unwrap(), Some(old));
        assert_eq!(
            lookup_at(&cards, engagement, role, currency, today())
                .unwrap()
                .hourly_rate_minor,
            10_000
        );
        assert_eq!(
            lookup_at(
                &cards,
                engagement,
                role,
                currency,
                today() + Duration::days(10)
            )
            .unwrap()
            .hourly_rate_minor,
            20_000
        );
    }

    #[test]
    fn negative_and_far_future_rates_rejected() {
        let err = create_rate_card(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BillingRole::Qa,
            Currency::VND,
            -1,
            true,
            today(),
            Uuid::new_v4(),
        )
        .unwrap_err();
        assert_eq!(err, RateCardError::NegativeAmount(-1));

        let err = create_rate_card(
            Uuid::new_v4(),
            Uuid::new_v4(),
            BillingRole::Qa,
            Currency::VND,
            1,
            true,
            today() + Duration::days(366),
            Uuid::new_v4(),
        )
        .unwrap_err();
        assert_eq!(err, RateCardError::EffectiveFromTooFar);
    }
}
