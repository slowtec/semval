use semval::prelude::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Email(pub String);

impl Email {
    pub const fn min_len() -> usize {
        // a@b.c = 5 chars
        5
    }
}

impl Validate<()> for Email {
    fn validate(&self) -> ValidationResult<()> {
        let mut errors = Self::start_validation();
        if self.0.len() < Self::min_len() {
            errors.add_error((), Validity::too_short(Self::min_len()));
        }
        if self.0.chars().filter(|c| *c == '@').count() != 1 {
            errors.add_error((), Validity::Invalid);
        }
        errors.into_result()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Phone(pub String);

impl Phone {
    pub const fn min_len() -> usize {
        6
    }
}

impl Validate<()> for Phone {
    fn validate(&self) -> ValidationResult<()> {
        if self.0.chars().filter(|c| !c.is_whitespace()).count() < Self::min_len() {
            ValidationErrors::error((), Validity::too_short(Self::min_len())).into_result()
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Customer {
    pub name: String,
    pub email: Option<Email>,
    pub phone: Option<Phone>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CustomerValidation {
    Name,
    Phone,
    Email,
    ContactData,
}

impl Validate<CustomerValidation> for Customer {
    fn validate(&self) -> ValidationResult<CustomerValidation> {
        let mut errors = Self::start_validation();
        if self.name.is_empty() {
            errors.add_error(CustomerValidation::Name, Validity::Empty);
        }
        if let Some(ref email) = self.email {
            errors.map_and_merge_result(email.validate(), |()| CustomerValidation::Email)
        }
        if let Some(ref phone) = self.phone {
            errors.map_and_merge_result(phone.validate(), |()| CustomerValidation::Phone)
        }
        // Either email or phone must be present
        if self.email.is_none() && self.phone.is_none() {
            errors.add_error(CustomerValidation::ContactData, Validity::Missing);
        }
        errors.into_result()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
struct Quantity(pub usize);

impl Validate<()> for Quantity {
    fn validate(&self) -> ValidationResult<()> {
        if self.0 >= 1 {
            Ok(())
        } else {
            ValidationErrors::error((), Validity::too_few(1)).into_result()
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Reservation {
    pub customer: Customer,
    pub quantity: Quantity,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReservationValidation {
    Customer(CustomerValidation),
    Quantity,
}

impl Validate<ReservationValidation> for Reservation {
    fn validate(&self) -> ValidationResult<ReservationValidation> {
        let mut errors = Self::start_validation();
        errors.map_and_merge_result(self.customer.validate(), ReservationValidation::Customer);
        errors.map_and_merge_result(self.quantity.validate(), |()| ReservationValidation::Quantity);
        errors.into_result()
    }
}

fn main() {
    let default_reservation = Reservation::default();
    println!("Validate {:?}: {:?}", default_reservation, default_reservation.validate());
}