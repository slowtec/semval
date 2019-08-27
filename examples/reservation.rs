use semval::prelude::*;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct UnexpectedValue<T> {
    pub expected: T,
    pub actual: T,
}

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
    MinLen(UnexpectedValue<usize>),
    InvalidFormat,
}

impl Validate for Email {
    type Validation = EmailValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::valid();
        context.add_violation_if(
            self.0.len() < Self::min_len(),
            EmailValidation::MinLen(UnexpectedValue {
                expected: Self::min_len(),
                actual: self.0.len(),
            }),
        );
        context.add_violation_if(
            self.0.chars().filter(|c| *c == '@').count() != 1,
            EmailValidation::InvalidFormat,
        );
        context.into()
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
    MinLen(UnexpectedValue<usize>),
}

impl Validate for Phone {
    type Validation = PhoneValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::valid();
        let len = self.0.chars().filter(|c| !c.is_whitespace()).count();
        context.add_violation_if(
            len < Self::min_len(),
            PhoneValidation::MinLen(UnexpectedValue {
                expected: Self::min_len(),
                actual: len,
            }),
        );
        context.into()
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
        let mut context = ValidationContext::valid();
        if let Some(ref email) = self.email {
            context.validate_and_map(email, ContactDataValidation::Email)
        }
        if let Some(ref phone) = self.phone {
            context.validate_and_map(phone, ContactDataValidation::Phone)
        }
        // Either email or phone must be present
        context.add_violation_if(
            self.email.is_none() && self.phone.is_none(),
            ContactDataValidation::Incomplete,
        );
        context.into()
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
        let mut context = ValidationContext::valid();
        context.add_violation_if(self.name.is_empty(), CustomerValidation::NameEmpty);
        context.validate_and_map(
            &self.contact_data,
            CustomerValidation::ContactData,
        );
        context.into()
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
    Min(UnexpectedValue<Quantity>),
}

impl Validate for Quantity {
    type Validation = QuantityValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::valid();
        context.add_violation_if(
            *self < Self::min(),
            QuantityValidation::Min(UnexpectedValue {
                expected: Self::min(),
                actual: *self,
            }),
        );
        context.into()
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
        let mut context = ValidationContext::valid();
        context.validate_and_map(&self.customer, ReservationValidation::Customer);
        context.validate_and_map(&self.quantity, ReservationValidation::Quantity);
        context.into()
    }
}

fn main() {
    let mut reservation = Reservation::default();
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.contact_data.email = Some(Email("a@b@c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.name = "Mr X".to_string();
    reservation.customer.contact_data.phone = Some(Phone("0 123".to_string()));
    reservation.customer.contact_data.email = None;
    reservation.quantity = Quantity(4);
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.contact_data.phone = None;
    reservation.customer.contact_data.email = Some(Email("a@b.c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());
}
