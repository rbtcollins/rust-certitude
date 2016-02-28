use security_framework::certificate::SecCertificate;
use security_framework::policy::SecPolicy;
use security_framework::secure_transport::ProtocolSide;
use security_framework::trust::SecTrust;

pub fn validate_cert_chain(encoded_certs: Vec<&[u8]>, hostname: &str) -> bool {
    let mut certs = Vec::new();
    for encoded_cert in encoded_certs {
        let cert = SecCertificate::from_der(encoded_cert);
        match cert {
            Ok(cert) => certs.push(cert),
            Err(_) => return false,
        }
    }

    let ssl_policy = fail_on_error!(SecPolicy::for_ssl(ProtocolSide::Client, hostname));
    let trust = fail_on_error!(SecTrust::create_with_certificates(&certs[..], &[ssl_policy]));

    // Deliberately swallow errors here: any error is likely to do with the
    // hostname or cert chain, and if those are invalid then by definition the
    // cert does not validate.
    match trust.evaluate() {
        Ok(result) => result.success(),
        Err(_) => false
    }
}

#[cfg(test)]
mod test {
    use osx::validate_cert_chain;
    use test::{expired_chain, certifi_chain, self_signed_chain};

    #[test]
    fn can_validate_good_chain() {
        let chain = certifi_chain();
        let valid = validate_cert_chain(chain, "certifi.io");
        assert_eq!(valid, true);
    }

    #[test]
    fn fails_on_bad_hostname() {
        let chain = certifi_chain();
        let valid = validate_cert_chain(chain, "lukasa.co.uk");
        assert_eq!(valid, false);
    }

    #[test]
    fn fails_on_bad_cert() {
        let mut good_chain = certifi_chain();
        let originals = good_chain.split_first_mut().unwrap();
        let leaf = originals.0;
        let intermediates = originals.1;

        // Deliberately truncate the leaf cert.
        let mut certs = vec![&leaf[1..50]];
        certs.extend(intermediates.iter());
        let valid = validate_cert_chain(certs, "certifi.io");
        assert_eq!(valid, false);
    }

    #[test]
    fn fails_on_expired_cert() {
        let chain = expired_chain();
        let valid = validate_cert_chain(chain, "expired.badssl.com");
        assert_eq!(valid, false);
    }

    #[test]
    fn test_fails_on_self_signed() {
        let chain = self_signed_chain();
        let valid = validate_cert_chain(chain, "self-signed.badssl.com");
        assert_eq!(valid, false);
    }
}
