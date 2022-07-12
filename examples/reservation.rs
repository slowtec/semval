// SPDX-FileCopyrightText: slowtec GmbH
// SPDX-License-Identifier: MPL-2.0

use semval::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct EmailAddress(String);

impl EmailAddress {
    const fn min_len() -> usize {
        // a@b.c = 5 chars
        5
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum EmailAddressInvalidity {
    MinLength,
    Format,
}

impl Validate for EmailAddress {
    type Invalidity = EmailAddressInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.0.len() < Self::min_len(),
                EmailAddressInvalidity::MinLength,
            )
            .invalidate_if(
                self.0.chars().filter(|c| *c == '@').count() != 1,
                EmailAddressInvalidity::Format,
            )
            .into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PhoneNumber(String);

impl PhoneNumber {
    const fn min_len() -> usize {
        6
    }
}

impl PhoneNumber {
    pub fn len(&self) -> usize {
        self.0.chars().filter(|c| !c.is_whitespace()).count()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PhoneNumberInvalidity {
    MinLength,
}

impl Validate for PhoneNumber {
    type Invalidity = PhoneNumberInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.len() < Self::min_len(),
                PhoneNumberInvalidity::MinLength,
            )
            .into()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ContactData {
    email: Option<EmailAddress>,
    phone: Option<PhoneNumber>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ContactDataInvalidity {
    Phone(PhoneNumberInvalidity),
    Email(EmailAddressInvalidity),
    Incomplete,
}

impl Validate for ContactData {
    type Invalidity = ContactDataInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.email, ContactDataInvalidity::Email)
            .validate_with(&self.phone, ContactDataInvalidity::Phone)
            .invalidate_if(
                // Either email or phone must be present
                self.email.is_none() && self.phone.is_none(),
                ContactDataInvalidity::Incomplete,
            )
            .into()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Customer {
    name: String,
    contact_data: ContactData,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CustomerInvalidity {
    NameEmpty,
    ContactData(ContactDataInvalidity),
}

// This conversion allows to use ValidationContext::validate()
// instead of ValidationContext::validate_with().
impl From<ContactDataInvalidity> for CustomerInvalidity {
    fn from(from: ContactDataInvalidity) -> Self {
        CustomerInvalidity::ContactData(from)
    }
}

impl Validate for Customer {
    type Invalidity = CustomerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.name.is_empty(), CustomerInvalidity::NameEmpty)
            .validate(&self.contact_data)
            .into()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Quantity(usize);

impl Quantity {
    const fn min() -> Self {
        Self(1)
    }

    fn new(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum QuantityInvalidity {
    MinValue,
}

impl Validate for Quantity {
    type Invalidity = QuantityInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::min(), QuantityInvalidity::MinValue)
            .into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Reservation {
    customer: Customer,
    quantity: Quantity,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ReservationInvalidity {
    Customer(CustomerInvalidity),
    Quantity(QuantityInvalidity),
}

impl Validate for Reservation {
    type Invalidity = ReservationInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.customer, ReservationInvalidity::Customer)
            .validate_with(&self.quantity, ReservationInvalidity::Quantity)
            .into()
    }
}

fn new_reservation_with_quantity(quantity: Quantity) -> Reservation {
    Reservation {
        customer: Customer {
            name: "Mr X".to_string(),
            contact_data: ContactData {
                email: Some(EmailAddress("mr_x@example.com".to_string())),
                ..Default::default()
            },
        },
        quantity,
    }
}

fn process_reservation(reservation: &Validated<Reservation>) {
    println!("Processing reservation: {:?}", &reservation);
}

fn main() {
    let mut reservation = Reservation {
        customer: Default::default(),
        quantity: Quantity::new(0),
    };
    println!("{:?}: {:?}", reservation, reservation.validate());
    debug_assert!(!reservation.is_valid());

    reservation.customer.contact_data.email = Some(EmailAddress("a@b@c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());
    debug_assert!(!reservation.is_valid());

    reservation.customer.name = "Mr X".to_string();
    reservation.customer.contact_data.phone = Some(PhoneNumber("0 123".to_string()));
    reservation.customer.contact_data.email = None;
    reservation.quantity = Quantity(4);
    println!("{:?}: {:?}", reservation, reservation.validate());
    debug_assert!(!reservation.is_valid());

    reservation.customer.contact_data.phone = None;
    reservation.customer.contact_data.email = Some(EmailAddress("a@b.c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());
    debug_assert!(reservation.is_valid());

    // Type-safe conversion and validation of input data
    for quantity in &[Quantity::new(1), Quantity::new(0)] {
        let new_reservation = new_reservation_with_quantity(*quantity);
        match Reservation::validated_from(new_reservation) {
            Ok(reservation) => {
                debug_assert!(reservation.is_valid());
                process_reservation(&reservation);
            }
            Err((reservation, context)) => {
                debug_assert!(!reservation.is_valid());
                println!(
                    "Received an invalid reservation {:?}: {:?}",
                    reservation, context
                );
            }
        }
    }
}
