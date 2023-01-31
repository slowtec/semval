<!-- SPDX-FileCopyrightText: slowtec GmbH -->
<!-- SPDX-License-Identifier: MPL-2.0 -->

# semval

[![Crates.io](https://img.shields.io/crates/v/semval.svg)](https://crates.io/crates/semval)
[![Docs.rs](https://docs.rs/semval/badge.svg)](https://docs.rs/semval)
[![Deps.rs](https://deps.rs/repo/github/slowtec/semval/status.svg)](https://deps.rs/repo/github/slowtec/semval)
[![Security audit](https://github.com/slowtec/semval/actions/workflows/security-audit.yaml/badge.svg)](https://github.com/slowtec/semval/actions/workflows/security-audit.yaml)
[![Continuous integration](https://github.com/slowtec/semval/actions/workflows/continuous-integration.yaml/badge.svg)](https://github.com/slowtec/semval/actions/workflows/continuous-integration.yaml)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)

A lightweight and unopinionated library with minimal dependencies for semantic validation in Rust.

Without any macro magic, at least not now.

TL;DR If you need to validate complex data structures at runtime then this crate
may empower you to enrich your domain model with semantic validation.

## Motivation

How do you recursively validate complex data structures, collect all violations
along the way, and finally report or evaluate those findings? Validating external
data at runtime before feeding it into further processing stages is crucial to
avoid inconsistencies and even more to prevent physical damage.

## Example

### Use case

Assume you are creating a web service for managing _reservations_ in a restaurant.
Customers can place reservations for a certain start time and a number of guests.
As _contact data_ they need to leave their _phone number_ or _e-mail address_,
at least one of both.

The JSON request body for creating a new reservation may look like in this example:

```json
{
  "start": "2019-07-30T18:00:00Z",
  "number_of_guests": 4,
  "customer": {
    "name": "slowtec GmbH",
    "contact_data": {
      "phone": "+49 711 500 716 72",
      "email": "post@slowtec.de"
    }
  }
}
```

### Domain model

Let's focus on the contact data. The corresponding type-safe data model in Rust might
look like this:

```rust
struct PhoneNumber(String);

struct EmailAddress(String);

struct ContactData {
  pub phone: Option<PhoneNumber>,
  pub email: Option<EmailAddress>,
}
```

In this example, both phone number and e-mail address are still represented by strings,
but wrapped into
[tuple structs](https://doc.rust-lang.org/1.9.0/book/structs.html#tuple-structs)
with a single member. This commonly used
[_newtype_ pattern](https://doc.rust-lang.org/book/ch19-04-advanced-types.html?highlight=newtype#using-the-newtype-pattern-for-type-safety-and-abstraction)
establishes type safety at compile time and enables us to add _behavior_ to these
types.

### Business Rules

Our reservation business requires that contact data entities are only accepted if all
of the following conditions are satisfied:

- The e-mail address is valid
- The phone number is valid
- Either e-mail address, or phone number, or both are present

## Validation

Let's develop a software design for the _reservation_ example use case. It should empower
us to validate domain entities according to our business requirements.

We will solely focus on the _contact data_ entity for simplicity. This is sufficient to
deduce the basic principles. The complete code can be found in the file
[reservation.rs](https://github.com/slowtec/semval/blob/main/examples/reservation.rs)
that is provided as an example in the repository.

### Invalidity

What are the possible outcomes of a validation? If the validation succeeds we are done
and processing continues as if nothing happened, i.e. validation is typically an
[_idempotent_](https://en.wikipedia.org/wiki/Idempotence)
operation. If the validation fails we somehow want to understand why it failed to
resolve conflicts or to fix inconsistencies. Finally, we may need to report any
unresolved findings back to the caller.

Reasons for a failed validation are expressed in terms of _invalidity_. An invalidity
is basically the inverse of some validation condition.

The invalidity variants for contact data are:

- The e-mail address is invalid
- The phone number is invalid
- Both e-mail address and phone number are missing

Please note that different invalidity variants may apply at the same time, e.g. both e-mail address
and phone number might be invalid for the same entity.

### Results

We already realized that the successful result of a validation is essentially _nothing_.
In Rust this nothing is represented by the unit type `()`.

Any invalidity will cause the validation to fail. Does this mean we should fail early and
abort the validation when detecting the first invalidity? Not necessarily. Consider the
use case of form validation with direct user interaction. If the user submits a form with
multiple invalid or missing fields we should report all of them to reduce the number of
unsuccessful retries and round trips.

This leads us to a preliminary definition for validation results:

```rust
type NaiveValidationResult = Result<(), Vec<Invalidity>>
```

We will refine it in a moment.

### Context

Validation is a recursive operation that needs to traverse deeply nested data structures.
The current state during such a traversal defines a _context_ for the validation with
a certain _level of abstraction_.

At the `ContactData` level we need to recursively validate both phone number and e-mail
address if present. Those subordinate validations are performed on a lower level of
abstraction, unaware of the upper-level context.

Additionally, we check if both members are missing and then reject the `ContactData` as
_incomplete_. This is the only validation that is actually implemented on the current
level without recursion.

Let's encode all possible variants in Rust by using _sum types_:

```rust
enum PhoneNumberInvalidity {
  ...lower abstraction level...
}

enum EmailAddressInvalidity {
  ...lower abstraction level...
}

enum ContactDataInvalidity {
  Phone(PhoneNumberInvalidity),
  Email(EmailAddressInvalidity),
  Incomplete,
}
```

Please note that each validation result refers to only a single `Invalidity` type. The
recursive nesting of validation results from lower-level contexts is achieved by wrapping
their `Invalidity` types into subordinate variants. The names of those variants typically
resemble the role names within the current context.

### Results ...continued

With the preliminary considerations, we are now able to finalize our definition of a
generic validation result:

```rust
struct ValidationContext<V: Invalidity> {
  ...implementation details...
}

type ValidationResult<V: Invalidity> = Result<(), ValidationContext<V>>
```

The `ValidationContext` is responsible for collecting validation results in the form
of multiple variants of the associated `Invalidity` type. Each item represents a
violation of some validation condition, i.e. a single invalidity that has been
detected. The concrete implementation of how invalidities are collected is hidden.

### Behavior

We enhance our domain entities by implementing the generic `Validate` trait:

```rust
pub trait Validate {
    type Invalidity: Invalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity>;
}
```

The associated type `Invalidity` is typically defined as a companion type
of the corresponding domain entity, as we have seen above. Don't get confused
by the trait bound of the same name that is just an alias for `Any + Debug`.

Provided that all components of our composite entity `ContactData` already
implement this trait the implementation becomes straightforward:

```rust
impl Validate for ContactData {
    type Invalidity = ContactDataInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.email, ContactDataInvalidity::EmailAddress)
            .validate_with(&self.phone, ContactDataInvalidity::PhoneNumber)
            .invalidate_if(
                // Either email or phone must be present
                self.email.is_none() && self.phone.is_none(),
                ContactDataInvalidity::Incomplete,
            )
            .into()
    }
}
```

The validation function starts by creating a new, empty context. Then it
continues by recursively collecting results from subordinate validations as well
as executing own validations rules. Finally, it transforms the context into a
result for passing it back to the caller.

The
[_fluent interface_](https://martinfowler.com/bliki/FluentInterface.html)
has proven to be useful and readable for the majority of use cases, even if more
complex validations may require to break the control flow at certain points.

## Corollary

We have translated the validation rules for our business requirements into a few
lines of comprehensive code. This code is associated with the corresponding
domain entity and only needs to consider a single level of abstraction.
Recursive composition enables us to validate complex data structures and to
trace back the cause of failed validations.

The validation code is independent of infrastructure components and an ideal
candidate for including it in the
[_functional core_](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell)
of a system. With simple unit tests we can verify that the validation
works as expected and reliably protects us from accepting invalid data.

## What not

We didn't cover

- how to enhance `Invalidity` types with additional, context-sensitive data
  by defining them as _tagged variants_ and
- how to route and interpret validation results.

The answers to both questions depend on each other, require use case-specific
solutions, and are not restricted by this library in any way.

## License

Licensed under the Mozilla Public License 2.0 (MPL-2.0) (see [MPL-2.0.txt](LICENSES/MPL-2.0.txt) or <https://www.mozilla.org/MPL/2.0/>).

Permissions of this copyleft license are conditioned on making available source code of licensed files and modifications of those files under the same license (or in certain cases, one of the GNU licenses). Copyright and license notices must be preserved. Contributors provide an express grant of patent rights. However, a larger work using the licensed work may be distributed under different terms and without source code for files added in the larger work.

### Contribution

Any contribution intentionally submitted for inclusion in the work by you shall be licensed under the Mozilla Public License 2.0 (MPL-2.0).

It is required to add the following header with the corresponding [SPDX short identifier](https://spdx.dev/ids/) to the top of each file:

```rust
// SPDX-License-Identifier: MPL-2.0
```
