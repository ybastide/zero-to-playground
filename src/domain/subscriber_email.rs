use validator::validate_email;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: &str) -> Result<SubscriberEmail, String> {
        if validate_email(s) {
            Ok(SubscriberEmail(s.to_string()))
        } else {
            Err(format!("invalid email {:?}", s))
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck_macros;
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn empty_string_is_rejected() {
        let email = "";
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com";
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com";
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_emails_are_parsed_successfully_once() {
        let email: String = SafeEmail().fake();
        claim::assert_ok!(SubscriberEmail::parse(&email));
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            // See https://github.com/LukeMathWalker/zero-to-production/issues/34#issuecomment-1124554482
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    impl AsRef<str> for ValidEmailFixture {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.as_ref()).is_ok()
    }
}
