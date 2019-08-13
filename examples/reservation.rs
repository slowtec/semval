use semval::prelude::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Email(String);

impl Email {
    const fn min_len() -> usize {
        // a@b.c = 5 chars
        5
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum EmailValidation {
    MinLen(usize),
    InvalidFormat,
}

impl Validate for Email {
    type Validation = EmailValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if self.0.len() < Self::min_len() {
            context.add_violation(EmailValidation::MinLen(Self::min_len()));
        }
        if self.0.chars().filter(|c| *c == '@').count() != 1 {
            context.add_violation(EmailValidation::InvalidFormat);
        }
        context.into_result()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Phone(String);

impl Phone {
    const fn min_len() -> usize {
        6
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PhoneValidation {
    MinLen(usize),
}

impl Validate for Phone {
    type Validation = PhoneValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if self.0.chars().filter(|c| !c.is_whitespace()).count() < Self::min_len() {
            context.add_violation(PhoneValidation::MinLen(Self::min_len()));
        }
        context.into_result()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ContactData {
    email: Option<Email>,
    phone: Option<Phone>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ContactDataValidation {
    Phone(PhoneValidation),
    Email(EmailValidation),
    Incomplete,
}

impl Validate for ContactData {
    type Validation = ContactDataValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if let Some(ref email) = self.email {
            context.map_and_merge_result(email.validate(), ContactDataValidation::Email)
        }
        if let Some(ref phone) = self.phone {
            context.map_and_merge_result(phone.validate(), ContactDataValidation::Phone)
        }
        // Either email or phone must be present
        if self.email.is_none() && self.phone.is_none() {
            context.add_violation(ContactDataValidation::Incomplete);
        }
        context.into_result()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Customer {
    name: String,
    contact_data: ContactData,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CustomerValidation {
    NameEmpty,
    ContactData(ContactDataValidation),
}

impl Validate for Customer {
    type Validation = CustomerValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if self.name.is_empty() {
            context.add_violation(CustomerValidation::NameEmpty);
        }
        context.map_and_merge_result(self.contact_data.validate(), CustomerValidation::ContactData);
        context.into_result()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
struct Quantity(usize);

impl Quantity {
    const fn min() -> Self {
        Self(1)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum QuantityValidation {
    Min(Quantity),
}

impl Validate for Quantity {
    type Validation = QuantityValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if *self < Quantity::min() {
            context.add_violation(QuantityValidation::Min(Self::min()));
        }
        context.into_result()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Reservation {
    customer: Customer,
    quantity: Quantity,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ReservationValidation {
    Customer(CustomerValidation),
    Quantity(QuantityValidation),
}

impl Validate for Reservation {
    type Validation = ReservationValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.customer.validate(), ReservationValidation::Customer);
        context.map_and_merge_result(self.quantity.validate(), ReservationValidation::Quantity);
        context.into_result()
    }
}

fn main() {
    let mut reservation = Reservation::default();
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.contact_data.email = Some(Email("a@b@c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.name = "Mr X".to_string();
    reservation.customer.contact_data.email = Some(Email("a@b.c".to_string()));
    reservation.quantity = Quantity(4);
    println!("{:?}: {:?}", reservation, reservation.validate());
}
