use semval::prelude::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
            .invalidate_if(self.0.len() < Self::min_len(), EmailAddressInvalidity::MinLength)
            .invalidate_if(
                self.0.chars().filter(|c| *c == '@').count() != 1,
                EmailAddressInvalidity::Format,
            )
            .into()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
            .invalidate_if(self.len() < Self::min_len(), PhoneNumberInvalidity::MinLength)
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
            .validate_and_map(&self.email, ContactDataInvalidity::Email)
            .validate_and_map(&self.phone, ContactDataInvalidity::Phone)
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

impl Validate for Customer {
    type Invalidity = CustomerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.name.is_empty(), CustomerInvalidity::NameEmpty)
            .validate_and_map(&self.contact_data, CustomerInvalidity::ContactData)
            .into()
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
            .validate_and_map(&self.customer, ReservationInvalidity::Customer)
            .validate_and_map(&self.quantity, ReservationInvalidity::Quantity)
            .into()
    }
}

fn main() {
    let mut reservation = Reservation::default();
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.contact_data.email = Some(EmailAddress("a@b@c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.name = "Mr X".to_string();
    reservation.customer.contact_data.phone = Some(PhoneNumber("0 123".to_string()));
    reservation.customer.contact_data.email = None;
    reservation.quantity = Quantity(4);
    println!("{:?}: {:?}", reservation, reservation.validate());

    reservation.customer.contact_data.phone = None;
    reservation.customer.contact_data.email = Some(EmailAddress("a@b.c".to_string()));
    println!("{:?}: {:?}", reservation, reservation.validate());
}
