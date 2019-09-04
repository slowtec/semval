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

// This conversion allows to use ValidationContext::validate()
// instead of ValidationContext::validate_and_map().
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
            .validate_and_map(&self.customer, ReservationInvalidity::Customer)
            .validate_and_map(&self.quantity, ReservationInvalidity::Quantity)
            .into()
    }
}

struct NewReservation(Reservation);

impl ValidatedFrom<NewReservation> for Reservation {
    fn validated_from(from: NewReservation) -> ValidatedResult<Reservation> {
        let into = from.0;
        if let Err(context) = into.validate() {
            Err((into, context))
        } else {
            Ok(into)
        }
    }
}

fn new_reservation_with_quantity(quantity: Quantity) -> NewReservation {
    NewReservation(Reservation {
        customer: Customer {
            name: "Mr X".to_string(),
            contact_data: ContactData {
                email: Some(EmailAddress("mr_x@example.com".to_string())),
                ..Default::default()
            },
        },
        quantity,
    })
}

fn main() {
    let mut reservation = Reservation {
        customer: Default::default(),
        quantity: Quantity::new(0),
    };
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

    // Type-safe conversion and validation of input data
    for quantity in &[Quantity::new(1), Quantity::new(0)] {
        let new_reservation = new_reservation_with_quantity(*quantity);
        match Reservation::validated_from(new_reservation) {
            Ok(valid_reservation) => {
                println!("Received a valid reservation {:?}", valid_reservation);
            }
            Err((invalid_reservation, context)) => {
                println!(
                    "Received an invalid reservation {:?}: {:?}",
                    invalid_reservation, context
                );
            }
        }
    }
}
